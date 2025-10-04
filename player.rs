// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: MIT

use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use futures::{future::OptionFuture, FutureExt};

mod audio;
mod video;

#[derive(Clone, Copy)]
pub enum ControlCommand {
    Play,
    Pause,
}

// Shared clock for audio-video synchronization
#[derive(Clone)]
pub struct SharedClock {
    start_time: Arc<Mutex<Option<Instant>>>,
    is_playing: Arc<Mutex<bool>>,
}

impl SharedClock {
    pub fn new() -> Self {
        Self {
            start_time: Arc::new(Mutex::new(None)),
            is_playing: Arc::new(Mutex::new(false)),
        }
    }
    
    pub fn start(&self) {
        let mut start_time = self.start_time.lock().unwrap();
        *start_time = Some(Instant::now());
        let mut is_playing = self.is_playing.lock().unwrap();
        *is_playing = true;
    }
    
    pub fn pause(&self) {
        let mut is_playing = self.is_playing.lock().unwrap();
        *is_playing = false;
    }
    
    pub fn resume(&self) {
        let mut is_playing = self.is_playing.lock().unwrap();
        *is_playing = true;
    }
    
    pub fn get_elapsed_time(&self) -> Option<Duration> {
        let start_time = self.start_time.lock().unwrap();
        let is_playing = self.is_playing.lock().unwrap();
        
        if *is_playing {
            start_time.map(|start| start.elapsed())
        } else {
            None
        }
    }
}

pub struct Player {
    control_sender: smol::channel::Sender<ControlCommand>,
    demuxer_thread: Option<std::thread::JoinHandle<()>>,
    playing: bool,
    playing_changed_callback: Box<dyn Fn(bool)>,
    shared_clock: SharedClock,
}

impl Player {
    pub fn start(
        path: PathBuf,
        video_frame_callback: impl FnMut(&ffmpeg_next::util::frame::Video) + Send + 'static,
        playing_changed_callback: impl Fn(bool) + 'static,
    ) -> Result<Self, anyhow::Error> {
        let (control_sender, control_receiver) = smol::channel::unbounded();
        let shared_clock = SharedClock::new();
        let shared_clock_for_thread = shared_clock.clone();

        let demuxer_thread =
            std::thread::Builder::new().name("demuxer thread".into()).spawn(move || {
                smol::block_on(async move {
                    let mut input_context = match ffmpeg_next::format::input(&path) {
                        Ok(ctx) => ctx,
                        Err(e) => {
                            eprintln!("Failed to open input file '{}': {}", path.display(), e);
                            return;
                        }
                    };

                    let video_stream = match input_context.streams().best(ffmpeg_next::media::Type::Video) {
                        Some(stream) => stream,
                        None => {
                            eprintln!("No video stream found in input file");
                            return;
                        }
                    };
                    let video_stream_index = video_stream.index();
                    let video_playback_thread = match video::VideoPlaybackThread::start(
                        &video_stream,
                        Box::new(video_frame_callback),
                        shared_clock_for_thread.clone(),
                    ) {
                        Ok(thread) => thread,
                        Err(e) => {
                            eprintln!("Failed to start video playback thread: {}", e);
                            return;
                        }
                    };

                    let audio_stream = match input_context.streams().best(ffmpeg_next::media::Type::Audio) {
                        Some(stream) => stream,
                        None => {
                            eprintln!("No audio stream found in input file");
                            return;
                        }
                    };
                    let audio_stream_index = audio_stream.index();
                    let audio_playback_thread = match audio::AudioPlaybackThread::start(&audio_stream, shared_clock_for_thread.clone()) {
                        Ok(thread) => thread,
                        Err(e) => {
                            eprintln!("Failed to start audio playback thread: {}", e);
                            return;
                        }
                    };

                    let mut playing = true;

                    // This is sub-optimal, as reading the packets from ffmpeg might be blocking
                    // and the future won't yield for that. So while ffmpeg sits on some blocking
                    // I/O operation, the caller here will also block and we won't end up polling
                    // the control_receiver future further down.
                    let packet_forwarder_impl = async {
                        for (stream, packet) in input_context.packets() {
                            if stream.index() == audio_stream_index {
                                audio_playback_thread.receive_packet(packet).await;
                            } else if stream.index() == video_stream_index {
                                video_playback_thread.receive_packet(packet).await;
                            }
                        }
                    }
                    .fuse()
                    .shared();

                    loop {
                        // This is sub-optimal, as reading the packets from ffmpeg might be blocking
                        // and the future won't yield for that. So while ffmpeg sits on some blocking
                        // I/O operation, the caller here will also block and we won't end up polling
                        // the control_receiver future further down.
                        let packet_forwarder: OptionFuture<_> =
                            if playing { Some(packet_forwarder_impl.clone()) } else { None }.into();

                        smol::pin!(packet_forwarder);

                        futures::select! {
                            _ = packet_forwarder => {}, // playback finished
                            received_command = control_receiver.recv().fuse() => {
                                match received_command {
                                    Ok(command) => {
                                        video_playback_thread.send_control_message(command).await;
                                        audio_playback_thread.send_control_message(command).await;
                                        match command {
                                            ControlCommand::Play => {
                                                // Continue in the loop, polling the packet forwarder future to forward
                                                // packets
                                                playing = true;
                                                shared_clock_for_thread.resume();
                                            },
                                            ControlCommand::Pause => {
                                                playing = false;
                                                shared_clock_for_thread.pause();
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        // Channel closed -> quit
                                        return;
                                    }
                                }
                            }
                        }
                    }
                })
            })?;

        let playing = true;
        playing_changed_callback(playing);

        Ok(Self {
            control_sender,
            demuxer_thread: Some(demuxer_thread),
            playing,
            playing_changed_callback: Box::new(playing_changed_callback),
            shared_clock,
        })
    }

    pub fn toggle_pause_playing(&mut self) {
        if self.playing {
            self.playing = false;
            self.shared_clock.pause();
            if let Err(e) = self.control_sender.send_blocking(ControlCommand::Pause) {
                eprintln!("Error sending pause command: {}", e);
            }
        } else {
            self.playing = true;
            self.shared_clock.resume();
            if let Err(e) = self.control_sender.send_blocking(ControlCommand::Play) {
                eprintln!("Error sending play command: {}", e);
            }
        }
        (self.playing_changed_callback)(self.playing);
    }
    
    pub fn stop(&mut self) {
        self.playing = false;
        self.shared_clock.pause();
        self.control_sender.close();
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.control_sender.close();
        if let Some(decoder_thread) = self.demuxer_thread.take() {
            if let Err(e) = decoder_thread.join() {
                eprintln!("Error joining demuxer thread: {:?}", e);
            }
        }
    }
}

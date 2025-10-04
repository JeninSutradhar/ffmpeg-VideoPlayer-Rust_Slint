// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: MIT

slint::include_modules!();

use ffmpeg_next::format::Pixel;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use ringbuf::{HeapRb, Rb};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

mod player;

#[derive(Clone)]
struct PlaybackState {
    playing: Arc<Mutex<bool>>,
    volume: Arc<Mutex<f32>>,
    seek_requested: Arc<Mutex<Option<f32>>>,
    should_stop: Arc<Mutex<bool>>,
}

impl PlaybackState {
    fn new() -> Self {
        Self {
            playing: Arc::new(Mutex::new(false)),
            volume: Arc::new(Mutex::new(0.7)),
            seek_requested: Arc::new(Mutex::new(None)),
            should_stop: Arc::new(Mutex::new(false)),
        }
    }
    
    fn is_playing(&self) -> bool {
        *self.playing.lock().unwrap()
    }
    
    fn set_playing(&self, playing: bool) {
        *self.playing.lock().unwrap() = playing;
    }
    
    fn get_volume(&self) -> f32 {
        *self.volume.lock().unwrap()
    }
    
    fn set_volume(&self, volume: f32) {
        *self.volume.lock().unwrap() = volume;
    }
    
    fn check_seek(&self) -> Option<f32> {
        self.seek_requested.lock().unwrap().take()
    }
    
    fn request_seek(&self, position: f32) {
        *self.seek_requested.lock().unwrap() = Some(position);
    }
    
    fn should_stop(&self) -> bool {
        *self.should_stop.lock().unwrap()
    }
    
    fn set_should_stop(&self, stop: bool) {
        *self.should_stop.lock().unwrap() = stop;
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize FFmpeg
    ffmpeg_next::init().map_err(|e| format!("Failed to initialize FFmpeg: {}", e))?;
    
    let app = App::new().map_err(|e| format!("Failed to create app: {}", e))?;
    
    // Set initial status
    app.set_status_text("Ready - Load a video to start".into());
    app.set_url_text("http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/TearsOfSteel.mp4".into());
    app.set_volume(0.7);

    // Shared playback state
    let playback_state = Arc::new(Mutex::new(Option::<PlaybackState>::None));
    
    // Set up UI callbacks
    let app_weak = app.as_weak();
    
    // Toggle play/pause
    {
        let playback_state = playback_state.clone();
        app.on_toggle_pause_play(move || {
            if let Some(state) = playback_state.lock().unwrap().as_ref() {
                let is_playing = state.is_playing();
                state.set_playing(!is_playing);
                println!("Playback {}", if !is_playing { "resumed" } else { "paused" });
            }
        });
    }
    
    // Load URL
    {
        let app_weak = app_weak.clone();
        let playback_state = playback_state.clone();
        
        app.on_load_url(move || {
            println!("Load URL clicked");
            
            // Get URL from UI before spawning thread
            let url = if let Some(app_strong) = app_weak.upgrade() {
                app_strong.get_url_text().to_string()
            } else {
                return;
            };
            
            if url.trim().is_empty() {
                println!("Error: URL is empty");
                return;
            }
            
            // Stop any existing playback
            if let Some(state) = playback_state.lock().unwrap().as_ref() {
                state.set_should_stop(true);
            }
            
            // Create new playback state
            let new_state = PlaybackState::new();
            *playback_state.lock().unwrap() = Some(new_state.clone());
            
            // Start video playback
            let app_weak_clone = app_weak.clone();
            std::thread::spawn(move || {
                if let Err(e) = load_and_play_video(app_weak_clone, url, new_state) {
                    let error_msg = format!("Error: {}", e);
                    eprintln!("{}", error_msg);
                }
            });
        });
    }
    
    // Select file
    {
        let app_weak = app_weak.clone();
        let playback_state = playback_state.clone();
        
        app.on_select_file(move || {
            println!("Select file clicked");
            
            // Open file dialog
            let file_path = rfd::FileDialog::new()
                .add_filter("Video Files", &["mp4", "mkv", "avi", "mov", "webm", "flv", "wmv", "m4v"])
                .add_filter("All Files", &["*"])
                .set_title("Select Video File")
                .pick_file();
            
            if let Some(path) = file_path {
                println!("Selected file: {:?}", path);
                let path_str = path.to_string_lossy().to_string();
                let path_str_clone = path_str.clone();
                
                // Update UI with selected file path
                let _ = app_weak.upgrade_in_event_loop(move |app| {
                    app.set_file_path(path_str_clone.into());
                });
                
                // Stop any existing playback
                if let Some(state) = playback_state.lock().unwrap().as_ref() {
                    state.set_should_stop(true);
                }
                
                // Create new playback state
                let new_state = PlaybackState::new();
                *playback_state.lock().unwrap() = Some(new_state.clone());
                
                // Start video playback
                let app_weak_clone = app_weak.clone();
                std::thread::spawn(move || {
                    if let Err(e) = load_and_play_video(app_weak_clone, path_str, new_state) {
                        let error_msg = format!("Error: {}", e);
                        eprintln!("{}", error_msg);
                    }
                });
            }
        });
    }
    
    // Stop video
    {
        let playback_state = playback_state.clone();
        app.on_stop_video(move || {
            println!("Stop video clicked");
            if let Some(state) = playback_state.lock().unwrap().as_ref() {
                state.set_should_stop(true);
                state.set_playing(false);
            }
        });
    }
    
    // Seek
    {
        let playback_state = playback_state.clone();
        app.on_seek_to(move |position| {
            if let Some(state) = playback_state.lock().unwrap().as_ref() {
                state.request_seek(position);
                println!("Seek requested to: {}", position);
            }
        });
    }
    
    // Volume changed
    {
        let playback_state = playback_state.clone();
        app.on_volume_changed(move |volume| {
            if let Some(state) = playback_state.lock().unwrap().as_ref() {
                state.set_volume(volume);
            }
        });
    }

    app.run().map_err(|e| format!("Failed to run app: {}", e))?;
    
    Ok(())
}

fn load_and_play_video(
    app_weak: slint::Weak<App>, 
    video_source: String,
    playback_state: PlaybackState
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading video from: {}", video_source);
    
    // Update UI status
    app_weak.upgrade_in_event_loop(|app| {
        app.set_status_text("Loading...".into());
        app.set_loading(true);
        app.set_playing(false);
    })?;
    
    // Open input
    let video_path = PathBuf::from(&video_source);
    let mut input_context = ffmpeg_next::format::input(&video_path)
        .map_err(|e| format!("Failed to open video: {}", e))?;
    
    // Find video stream
    let video_stream = input_context.streams()
        .best(ffmpeg_next::media::Type::Video)
        .ok_or("No video stream found")?;
    let video_stream_index = video_stream.index();
    
    // Get video duration
    let duration_secs = if video_stream.duration() > 0 {
        video_stream.duration() as f64 * f64::from(video_stream.time_base())
    } else if input_context.duration() > 0 {
        input_context.duration() as f64 / f64::from(ffmpeg_next::ffi::AV_TIME_BASE)
    } else {
        0.0
    };
    
    // Find audio stream
    let audio_stream = input_context.streams()
        .best(ffmpeg_next::media::Type::Audio);
    let audio_stream_index = audio_stream.as_ref().map(|s| s.index());
    
    // Setup video decoder
    let decoder_context = ffmpeg_next::codec::Context::from_parameters(video_stream.parameters())?;
    let mut video_decoder = decoder_context.decoder().video()?;
    
    // Setup audio decoder if audio stream exists
    let mut audio_decoder = if let Some(stream) = audio_stream {
        let audio_decoder_context = ffmpeg_next::codec::Context::from_parameters(stream.parameters())?;
        Some(audio_decoder_context.decoder().audio()?)
    } else {
        None
    };
    
    // Create scaling context
    let mut scaler = ffmpeg_next::software::scaling::Context::get(
        video_decoder.format(),
        video_decoder.width(),
        video_decoder.height(),
        Pixel::RGB24,
        video_decoder.width(),
        video_decoder.height(),
        ffmpeg_next::software::scaling::Flags::BILINEAR,
    )?;
    
    println!("Video info: {}x{}, duration: {:.2}s", 
        video_decoder.width(), video_decoder.height(), duration_secs);
    
    if audio_decoder.is_some() {
        println!("Audio stream found");
    }
    
    // Setup audio output if audio decoder exists
    let audio_buffer = Arc::new(Mutex::new(HeapRb::<f32>::new(44100 * 2))); // 1 second buffer
    let _audio_output = if let Some(ref decoder) = audio_decoder {
        match setup_audio_output(decoder, audio_buffer.clone()) {
            Ok(output) => Some(output),
            Err(e) => {
                eprintln!("Failed to setup audio: {}", e);
                None
            }
        }
    } else {
        None
    };
    
    // Update UI with duration
    app_weak.upgrade_in_event_loop(move |app| {
        app.set_duration(duration_secs as f32);
        app.set_total_time(format_time(duration_secs as f32).into());
        app.set_status_text("Playing".into());
        app.set_loading(false);
        app.set_playing(true);
    })?;
    
    // Start playback
    playback_state.set_playing(true);
    let start_time = Instant::now();
    let mut frame_count = 0;
    let frame_duration = Duration::from_millis(33); // ~30 FPS
    
    // Process packets
    for (stream, packet) in input_context.packets() {
        // Check if we should stop
        if playback_state.should_stop() {
            break;
        }
        
        // Check for seek request
        if let Some(_seek_pos) = playback_state.check_seek() {
            // TODO: Implement seeking
            println!("Seeking not yet implemented");
        }
        
        if stream.index() == video_stream_index {
            video_decoder.send_packet(&packet)?;
            
            let mut frame = ffmpeg_next::util::frame::Video::empty();
            while video_decoder.receive_frame(&mut frame).is_ok() {
                // Wait if paused
                while !playback_state.is_playing() && !playback_state.should_stop() {
                    std::thread::sleep(Duration::from_millis(100));
                }
                
                if playback_state.should_stop() {
                    break;
                }
                
                // Scale frame
                let mut rgb_frame = ffmpeg_next::util::frame::Video::empty();
                scaler.run(&frame, &mut rgb_frame)?;
                
                // Convert to pixel buffer
                let pixel_buffer = video_frame_to_pixel_buffer(&rgb_frame);
                
                // Update current time
                let elapsed = start_time.elapsed().as_secs_f32();
                frame_count += 1;
                
                // Update UI
                let _ = app_weak.upgrade_in_event_loop(move |app| {
                    app.set_video_frame(slint::Image::from_rgb8(pixel_buffer));
                    app.set_seek_position(elapsed);
                    app.set_current_time(format_time(elapsed).into());
                });
                
                // Frame rate control
                std::thread::sleep(frame_duration);
            }
        } else if Some(stream.index()) == audio_stream_index {
            if let Some(ref mut decoder) = audio_decoder {
                decoder.send_packet(&packet).ok();
                
                let mut audio_frame = ffmpeg_next::util::frame::Audio::empty();
                while decoder.receive_frame(&mut audio_frame).is_ok() {
                    // Process audio frame
                    process_audio_frame(&audio_frame, &audio_buffer, &playback_state);
                }
            }
        }
    }
    
    println!("Video playback finished (frames: {})", frame_count);
    
    // Update UI
    let _ = app_weak.upgrade_in_event_loop(|app| {
        app.set_status_text("Finished".into());
        app.set_playing(false);
    });
    
    playback_state.set_playing(false);
    Ok(())
}

fn setup_audio_output(
    _decoder: &ffmpeg_next::decoder::Audio,
    audio_buffer: Arc<Mutex<HeapRb<f32>>>
) -> Result<Arc<Mutex<cpal::Stream>>, Box<dyn std::error::Error>> {
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or("No output device available")?;
    
    let config = device.default_output_config()?;
    println!("Audio config: {:?}", config);
    
    let stream = device.build_output_stream(
        &config.into(),
        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
            // Read from audio buffer
            if let Ok(mut buffer) = audio_buffer.lock() {
                for sample in data.iter_mut() {
                    *sample = buffer.pop().unwrap_or(0.0);
                }
            }
        },
        |err| eprintln!("Audio stream error: {}", err),
        None,
    )?;
    
    stream.play()?;
    Ok(Arc::new(Mutex::new(stream)))
}

fn process_audio_frame(
    frame: &ffmpeg_next::util::frame::Audio,
    audio_buffer: &Arc<Mutex<HeapRb<f32>>>,
    playback_state: &PlaybackState,
) {
    if !playback_state.is_playing() {
        return;
    }
    
    let volume = playback_state.get_volume();
    
    // Convert audio frame to f32 samples
    let samples = match frame.format() {
        ffmpeg_next::format::Sample::F32(planar) => {
            if planar == ffmpeg_next::format::sample::Type::Planar {
                // Planar format - interleave the channels
                let channel_count = frame.channels() as usize;
                let samples_per_channel = frame.samples() as usize;
                let mut interleaved = Vec::with_capacity(samples_per_channel * channel_count);
                
                for i in 0..samples_per_channel {
                    for ch in 0..channel_count {
                        let sample = unsafe {
                            let ptr = frame.data(ch).as_ptr() as *const f32;
                            *ptr.add(i)
                        };
                        interleaved.push(sample * volume);
                    }
                }
                interleaved
            } else {
                // Interleaved format
                let samples = unsafe {
                    std::slice::from_raw_parts(
                        frame.data(0).as_ptr() as *const f32,
                        frame.samples() as usize * frame.channels() as usize
                    )
                };
                samples.iter().map(|&s| s * volume).collect()
            }
        }
        _ => {
            // For other formats, we'd need conversion
            // For now, just return empty
            return;
        }
    };
    
    // Push samples to buffer
    if let Ok(mut buffer) = audio_buffer.lock() {
        for sample in samples {
            if buffer.push(sample).is_err() {
                // Buffer full, skip this sample
                break;
            }
        }
    }
}

fn format_time(seconds: f32) -> String {
    let total_secs = seconds as u32;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{}:{:02}", mins, secs)
}

fn video_frame_to_pixel_buffer(
    frame: &ffmpeg_next::util::frame::Video,
) -> slint::SharedPixelBuffer<slint::Rgb8Pixel> {
    let mut pixel_buffer =
        slint::SharedPixelBuffer::<slint::Rgb8Pixel>::new(frame.width(), frame.height());

    let ffmpeg_line_iter = frame.data(0).chunks_exact(frame.stride(0));
    let slint_pixel_line_iter = pixel_buffer
        .make_mut_bytes()
        .chunks_mut(frame.width() as usize * core::mem::size_of::<slint::Rgb8Pixel>());

    for (source_line, dest_line) in ffmpeg_line_iter.zip(slint_pixel_line_iter) {
        dest_line.copy_from_slice(&source_line[..dest_line.len()])
    }

    pixel_buffer
}

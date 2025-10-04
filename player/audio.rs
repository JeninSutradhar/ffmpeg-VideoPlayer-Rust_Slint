use std::pin::Pin;

use bytemuck::Pod;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::SizedSample;

use futures::future::OptionFuture;
use futures::FutureExt;
use ringbuf::ring_buffer::RbRef;
use ringbuf::ring_buffer::RbWrite;
use ringbuf::HeapRb;
use std::future::Future;

use super::{ControlCommand, SharedClock};

pub struct AudioPlaybackThread {
    control_sender: smol::channel::Sender<ControlCommand>,
    packet_sender: smol::channel::Sender<ffmpeg_next::codec::packet::packet::Packet>,
    receiver_thread: Option<std::thread::JoinHandle<()>>,
}

impl AudioPlaybackThread {
    pub fn start(stream: &ffmpeg_next::format::stream::Stream, shared_clock: SharedClock) -> Result<Self, anyhow::Error> {
        let (control_sender, control_receiver) = smol::channel::unbounded();

        // Limit buffer size to prevent memory exhaustion
        let (packet_sender, packet_receiver) = smol::channel::bounded(64);

        let decoder_context = ffmpeg_next::codec::Context::from_parameters(stream.parameters())?;
        let packet_decoder = decoder_context.decoder().audio()?;

        let host = cpal::default_host();
        let device = match host.default_output_device() {
            Some(device) => device,
            None => {
                eprintln!("No audio output device available");
                return Err(anyhow::anyhow!("No audio output device available"));
            }
        };

        let config = match device.default_output_config() {
            Ok(config) => config,
            Err(e) => {
                eprintln!("Failed to get default output config: {}", e);
                return Err(anyhow::anyhow!("Failed to get default output config: {}", e));
            }
        };

        let receiver_thread =
            std::thread::Builder::new().name("audio playback thread".into()).spawn(move || {
                smol::block_on(async move {
                    let output_channel_layout = match config.channels() {
                        1 => ffmpeg_next::util::channel_layout::ChannelLayout::MONO,
                        2 => ffmpeg_next::util::channel_layout::ChannelLayout::STEREO,
                        _ => todo!(),
                    };

                    let mut ffmpeg_to_cpal_forwarder = match config.sample_format() {
                        cpal::SampleFormat::U8 => FFmpegToCPalForwarder::new::<u8>(
                            config,
                            &device,
                            packet_receiver,
                            packet_decoder,
                            ffmpeg_next::util::format::sample::Sample::U8(
                                ffmpeg_next::util::format::sample::Type::Packed,
                            ),
                            output_channel_layout,
                            shared_clock.clone(),
                        ),
                        cpal::SampleFormat::F32 => FFmpegToCPalForwarder::new::<f32>(
                            config,
                            &device,
                            packet_receiver,
                            packet_decoder,
                            ffmpeg_next::util::format::sample::Sample::F32(
                                ffmpeg_next::util::format::sample::Type::Packed,
                            ),
                            output_channel_layout,
                            shared_clock.clone(),
                        ),
                        format @ _ => todo!("unsupported cpal output format {:#?}", format),
                    };

                    let packet_receiver_impl =
                        async { ffmpeg_to_cpal_forwarder.stream().await }.fuse().shared();

                    let mut playing = true;

                    loop {
                        let packet_receiver: OptionFuture<_> =
                            if playing { Some(packet_receiver_impl.clone()) } else { None }.into();

                        smol::pin!(packet_receiver);

                        futures::select! {
                            _ = packet_receiver => {},
                            received_command = control_receiver.recv().fuse() => {
                                match received_command {
                                    Ok(ControlCommand::Pause) => {
                                        playing = false;
                                    }
                                    Ok(ControlCommand::Play) => {
                                        playing = true;
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

        Ok(Self { control_sender, packet_sender, receiver_thread: Some(receiver_thread) })
    }

    pub async fn receive_packet(&self, packet: ffmpeg_next::codec::packet::packet::Packet) -> bool {
        match self.packet_sender.send(packet).await {
            Ok(_) => return true,
            Err(smol::channel::SendError(_)) => return false,
        }
    }

    pub async fn send_control_message(&self, message: ControlCommand) {
        self.control_sender.send(message).await.unwrap();
    }
}

impl Drop for AudioPlaybackThread {
    fn drop(&mut self) {
        self.control_sender.close();
        if let Some(receiver_join_handle) = self.receiver_thread.take() {
            if let Err(e) = receiver_join_handle.join() {
                eprintln!("Error joining audio receiver thread: {:?}", e);
            }
        }
    }
}

trait FFMpegToCPalSampleForwarder {
    fn forward(
        &mut self,
        audio_frame: ffmpeg_next::frame::Audio,
    ) -> Pin<Box<dyn Future<Output = ()> + '_>>;
}

impl<T: Pod, R: RbRef> FFMpegToCPalSampleForwarder for ringbuf::Producer<T, R>
where
    <R as RbRef>::Rb: RbWrite<T>,
{
    fn forward(
        &mut self,
        audio_frame: ffmpeg_next::frame::Audio,
    ) -> Pin<Box<dyn Future<Output = ()> + '_>> {
        Box::pin(async move {
            // Audio::plane() returns the wrong slice size, so correct it by hand. See also
            // for a fix https://github.com/zmwangx/rust-ffmpeg/pull/104.
            let expected_bytes =
                audio_frame.samples() * audio_frame.channels() as usize * core::mem::size_of::<T>();
            let cpal_sample_data: &[T] =
                bytemuck::cast_slice(&audio_frame.data(0)[..expected_bytes]);

            while self.free_len() < cpal_sample_data.len() {
                smol::Timer::after(std::time::Duration::from_millis(16)).await;
            }

            // Buffer the samples for playback
            self.push_slice(cpal_sample_data);
        })
    }
}

struct FFmpegToCPalForwarder {
    _cpal_stream: cpal::Stream,
    ffmpeg_to_cpal_pipe: Box<dyn FFMpegToCPalSampleForwarder>,
    packet_receiver: smol::channel::Receiver<ffmpeg_next::codec::packet::packet::Packet>,
    packet_decoder: ffmpeg_next::decoder::Audio,
    resampler: ffmpeg_next::software::resampling::Context,
    shared_clock: SharedClock,
}

impl FFmpegToCPalForwarder {
    fn new<T: Send + Pod + SizedSample + 'static>(
        config: cpal::SupportedStreamConfig,
        device: &cpal::Device,
        packet_receiver: smol::channel::Receiver<ffmpeg_next::codec::packet::packet::Packet>,
        packet_decoder: ffmpeg_next::decoder::Audio,
        output_format: ffmpeg_next::util::format::sample::Sample,
        output_channel_layout: ffmpeg_next::util::channel_layout::ChannelLayout,
        shared_clock: SharedClock,
    ) -> Self {
        let buffer = HeapRb::new(4096);
        let (sample_producer, mut sample_consumer) = buffer.split();

        let cpal_stream = match device
            .build_output_stream(
                &config.config(),
                move |data, _| {
                    let filled = sample_consumer.pop_slice(data);
                    data[filled..].fill(T::EQUILIBRIUM);
                },
                move |err| {
                    eprintln!("error feeding audio stream to cpal: {}", err);
                },
                None,
            ) {
            Ok(stream) => stream,
            Err(e) => {
                eprintln!("Failed to build output stream: {}", e);
                panic!("Failed to build output stream: {}", e);
            }
        };

        if let Err(e) = cpal_stream.play() {
            eprintln!("Failed to play audio stream: {}", e);
            panic!("Failed to play audio stream: {}", e);
        }

        let resampler = match ffmpeg_next::software::resampling::Context::get(
            packet_decoder.format(),
            packet_decoder.channel_layout(),
            packet_decoder.rate(),
            output_format,
            output_channel_layout,
            config.sample_rate().0,
        ) {
            Ok(resampler) => resampler,
            Err(e) => {
                eprintln!("Failed to create resampler: {}", e);
                panic!("Failed to create resampler: {}", e);
            }
        };

        Self {
            _cpal_stream: cpal_stream,
            ffmpeg_to_cpal_pipe: Box::new(sample_producer),
            packet_receiver,
            packet_decoder,
            resampler,
            shared_clock,
        }
    }

    async fn stream(&mut self) {
    loop {
        // Receive the next packet from the packet receiver channel.
        let Ok(packet) = self.packet_receiver.recv().await else { break };

        // Send the packet to the decoder.
        if let Err(e) = self.packet_decoder.send_packet(&packet) {
            eprintln!("Error sending packet to decoder: {}", e);
            return;
        }

        // Create an empty frame to hold the decoded audio data.
        let mut decoded_frame = ffmpeg_next::util::frame::Audio::empty();

        // Continue receiving decoded frames until there are no more available.
        while self.packet_decoder.receive_frame(&mut decoded_frame).is_ok() {
            // Create an empty frame to hold the resampled audio data.
            let mut resampled_frame = ffmpeg_next::util::frame::Audio::empty();

            // Resample the decoded audio frame to match the output format and channel layout.
            if let Err(e) = self.resampler.run(&decoded_frame, &mut resampled_frame) {
                eprintln!("Error resampling audio frame: {}", e);
                continue;
            }

            // Forward the resampled audio frame to the CPAL audio output.
            // Use shared clock to ensure audio-video sync
            if let Some(_elapsed) = self.shared_clock.get_elapsed_time() {
                self.ffmpeg_to_cpal_pipe.forward(resampled_frame).await;
            }
        }
    }
}
}

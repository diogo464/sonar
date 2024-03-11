use std::{
    collections::VecDeque,
    sync::{
        atomic::{AtomicBool, AtomicU32},
        Arc, Mutex,
    },
    time::Duration,
};

use symphonia::core::{codecs::Decoder, formats::FormatReader, io::MediaSourceStream};
use tokio::{
    io::{AsyncReadExt, AsyncSeekExt},
    task::AbortHandle,
};

use super::AudioSource;
use crate::{Error, ErrorKind, Result};

pub type EventSender<T = AudioPlayerEvent> = tokio::sync::mpsc::UnboundedSender<T>;
pub type EventReceiver<T = AudioPlayerEvent> = tokio::sync::mpsc::UnboundedReceiver<T>;
type CSender = crossbeam::channel::Sender<Message>;
type CReceiver = crossbeam::channel::Receiver<Message>;
type OSender<T> = tokio::sync::oneshot::Sender<T>;

pub const CHANNELS: u32 = 2;
pub const SAMPLE_SIZE: u32 = 4;
pub const SAMPLE_RATE: u32 = 44100;

const BUFFER_SIZE_SAMPLES_LO: u32 = 8192;
const BUFFER_SIZE_SAMPLES_HI: u32 = 16 * BUFFER_SIZE_SAMPLES_LO;

#[derive(Debug, Clone)]
pub enum AudioPlayerEvent {
    PlaybackStarted,
    PlaybackPaused,
    PlaybackFinished,
    PlaybackError,
}

pub struct SymphoniaMediaSourceWrapper(Box<dyn AudioSource>);

impl std::io::Read for SymphoniaMediaSourceWrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        tracing::debug!("symphonia read {} bytes", buf.len());
        futures::executor::block_on(self.0.read(buf))
    }
}

impl std::io::Seek for SymphoniaMediaSourceWrapper {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        tracing::debug!("symphonia seek {:?}", pos);
        futures::executor::block_on(self.0.seek(pos))
    }
}

impl symphonia::core::io::MediaSource for SymphoniaMediaSourceWrapper {
    fn is_seekable(&self) -> bool {
        true
    }

    fn byte_len(&self) -> Option<u64> {
        Some(self.0.size())
    }
}

enum Message {
    Source(Box<dyn AudioSource>),
    Play,
    Pause,
    Toggle,
    Stop,
    Seek(Duration),
    Position(OSender<Duration>),
    Close,
}

pub struct AudioPlayer {
    sender: CSender,
}

impl Drop for AudioPlayer {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Close);
    }
}

impl AudioPlayer {
    pub fn new() -> (Self, EventReceiver) {
        let (message_sender, message_receiver) = crossbeam::channel::unbounded();
        let (event_sender, event_receiver) = tokio::sync::mpsc::unbounded_channel();
        tokio::task::spawn_blocking(move || {
            audio_thread(message_receiver, event_sender).unwrap();
        });
        (
            Self {
                sender: message_sender,
            },
            event_receiver,
        )
    }
    pub fn source(&self, source: impl AudioSource) {
        self.sender.send(Message::Source(Box::new(source))).unwrap();
    }
    pub fn play(&self) {
        self.sender.send(Message::Play).unwrap();
    }
    pub fn pause(&self) {
        self.sender.send(Message::Pause).unwrap();
    }
    pub fn toggle(&self) {
        self.sender.send(Message::Toggle).unwrap();
    }
    pub fn stop(&self) {
        self.sender.send(Message::Stop).unwrap();
    }
    pub fn seek(&self, position: Duration) {
        self.sender.send(Message::Seek(position)).unwrap();
    }
    pub async fn position(&self) -> Duration {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender.send(Message::Position(tx)).unwrap();
        rx.await.unwrap()
    }
}

#[derive(Default)]
struct AudioShared {
    buffer: Mutex<VecDeque<f32>>,
    playing: AtomicBool,
    required: AtomicU32,
    copied: AtomicU32,
}

impl AudioShared {
    fn get_playing(&self) -> bool {
        self.playing.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn set_playing(&self, playing: bool) {
        self.playing
            .store(playing, std::sync::atomic::Ordering::Relaxed);
    }

    fn get_required(&self) -> u32 {
        self.required.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn take_copied_duration(&self) -> Duration {
        let copied = self.copied.swap(0, std::sync::atomic::Ordering::Relaxed);
        let duration =
            Duration::from_secs_f64(copied as f64 / (SAMPLE_RATE as f64 * CHANNELS as f64));
        duration
    }

    fn copy_samples(&self, samples: &[f32]) {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend(samples);
        self.required
            .fetch_sub(samples.len() as u32, std::sync::atomic::Ordering::Relaxed);
    }

    fn fill(&self, samples: &mut [f32]) {
        use cpal::Sample;
        let mut buffer = self.buffer.lock().unwrap();
        let mut copied = 0;
        for sample in samples {
            *sample = match buffer.pop_front() {
                Some(v) => {
                    copied += 1;
                    v
                }
                None => f32::EQUILIBRIUM,
            };
        }
        let required = BUFFER_SIZE_SAMPLES_HI.saturating_sub(buffer.len() as u32);
        self.copied
            .fetch_add(copied, std::sync::atomic::Ordering::Relaxed);
        self.required
            .fetch_add(required, std::sync::atomic::Ordering::Relaxed);
    }

    fn clear(&self) {
        self.buffer.lock().unwrap().clear();
        self.required.store(0, std::sync::atomic::Ordering::Relaxed);
        self.copied.store(0, std::sync::atomic::Ordering::Relaxed);
    }
}

struct PreparedSource {
    decoder: Box<dyn Decoder>,
    reader: Box<dyn FormatReader>,
    sample_rate: u32,
    channels: u32,
    base: symphonia::core::units::TimeBase,
}

impl PreparedSource {
    fn seek(&mut self, position: Duration) -> Result<Duration> {
        let result = self.reader.seek(
            symphonia::core::formats::SeekMode::Coarse,
            symphonia::core::formats::SeekTo::Time {
                time: symphonia::core::units::Time {
                    seconds: position.as_secs(),
                    frac: position.as_secs_f64().fract(),
                },
                track_id: None,
            },
        );
        let position = match result {
            Ok(to) => {
                let time = self.base.calc_time(to.actual_ts);
                let position = Duration::new(time.seconds, (time.frac * 1_000_000_000.0) as u32);
                position
            }
            Err(err) => {
                tracing::error!("error seeking audio source: {}", err);
                return Err(Error::wrap(err));
            }
        };
        self.decoder.reset();
        Ok(position)
    }
}

fn audio_thread(receiver: CReceiver, events: EventSender) -> Result<()> {
    use cpal::traits::StreamTrait;

    let shared = Arc::new(AudioShared::default());

    let mut _stream = None;
    let mut sink = None;

    let timeout = Duration::from_millis(100);
    let mut samples = audio_sample_buffer();
    let mut prepared = None;
    let mut eof = false;
    let mut current_position = Duration::default();

    loop {
        if let Ok(message) = receiver.recv_timeout(timeout) {
            match message {
                Message::Source(source) => match audio_prepare(source) {
                    Ok(v) => {
                        // TODO: this is a bit of a hack to allow disabling audio for testing
                        if std::env::var("MADS_NO_AUDIO").is_err() {
                            let astream = audio_stream(shared.clone(), v.channels, v.sample_rate)?;
                            astream.play().map_err(Error::wrap)?;
                            _stream = Some(astream);
                            sink = None;
                        } else {
                            _stream = None;
                            sink = Some(audio_null_sink(shared.clone()));
                        };

                        tracing::info!("prepared audio source");
                        shared.clear();
                        current_position = Duration::ZERO;
                        prepared = Some(v);
                        eof = false;
                    }
                    Err(err) => tracing::error!("error preparing audio source: {}", err),
                },
                Message::Play => {
                    tracing::debug!("play");
                    shared.set_playing(true);
                    let _ = events.send(AudioPlayerEvent::PlaybackStarted);
                }
                Message::Pause => {
                    tracing::debug!("pause");
                    shared.set_playing(false);
                    let _ = events.send(AudioPlayerEvent::PlaybackPaused);
                }
                Message::Toggle => {
                    tracing::debug!("toggle");
                    if shared.get_playing() {
                        shared.set_playing(false);
                        let _ = events.send(AudioPlayerEvent::PlaybackPaused);
                    } else {
                        shared.set_playing(true);
                        let _ = events.send(AudioPlayerEvent::PlaybackStarted);
                    }
                }
                Message::Stop => {
                    tracing::debug!("stop");
                    current_position = Duration::ZERO;
                    shared.set_playing(false);
                    shared.clear();
                    prepared = None;
                    let _ = events.send(AudioPlayerEvent::PlaybackFinished);
                }
                Message::Seek(position) => {
                    if let Some(ref mut prepared) = prepared {
                        tracing::debug!("seek to {:?}", position);
                        if let Ok(new_position) = prepared.seek(position) {
                            tracing::debug!("seeked to {:?}", new_position);
                            shared.clear();
                            eof = false;
                            current_position = new_position;
                        } else {
                            tracing::warn!("invalid seek position");
                        }
                    } else {
                        tracing::debug!(
                            "seek to {:?} failed because there is no active track",
                            position
                        );
                    }
                }
                Message::Position(tx) => {
                    let _ = tx.send(current_position);
                }
                Message::Close => break,
            }
        }

        if let Some(ref mut prepared) = prepared {
            while shared.get_required() > BUFFER_SIZE_SAMPLES_LO && !eof {
                let packet = match prepared.reader.next_packet() {
                    Ok(packet) => packet,
                    Err(symphonia::core::errors::Error::IoError(e))
                        if e.kind() == std::io::ErrorKind::UnexpectedEof =>
                    {
                        eof = true;
                        tracing::debug!("end of file");
                        break;
                    }
                    Err(e) => {
                        tracing::error!("error reading packet: {}", e);
                        break;
                    }
                };
                let buffer = match prepared.decoder.decode(&packet) {
                    Ok(buffer) => buffer,
                    Err(e) => {
                        tracing::error!("error decoding packet: {}", e);
                        break;
                    }
                };
                //let buffer_spec = buffer.spec();
                //assert_eq!(buffer_spec.rate, SAMPLE_RATE);
                //assert_eq!(
                //    buffer_spec.channels,
                //    symphonia::core::audio::Channels::FRONT_LEFT
                //        | symphonia::core::audio::Channels::FRONT_RIGHT
                //);

                samples.copy_interleaved_ref(buffer);
                if samples.is_empty() {
                    break;
                }
                // tracing::trace!("buffering {} samples", samples.samples().len());
                shared.copy_samples(samples.samples());
            }
        }

        current_position += shared.take_copied_duration();
    }

    if let Some(sink) = sink {
        sink.abort();
    }

    tracing::info!("audio thread exiting");
    Ok(())
}

fn audio_prepare(source: Box<dyn AudioSource>) -> Result<PreparedSource> {
    let source = SymphoniaMediaSourceWrapper(source);
    let stream = MediaSourceStream::new(Box::new(source), Default::default());

    let codecs = symphonia::default::get_codecs();
    let probe = symphonia::default::get_probe();
    let result = probe
        .format(
            &Default::default(),
            stream,
            &Default::default(),
            &Default::default(),
        )
        .unwrap();
    let codec_params = result.format.tracks()[0].codec_params.clone();
    let decoder = codecs.make(&codec_params, &Default::default()).unwrap();

    Ok(PreparedSource {
        decoder,
        reader: result.format,
        sample_rate: codec_params.sample_rate.unwrap(),
        channels: codec_params.channels.unwrap().count() as u32,
        base: codec_params.time_base.unwrap(),
    })
}

fn audio_null_sink(shared: Arc<AudioShared>) -> AbortHandle {
    tokio::spawn(async move {
        let mut buffer = [0f32; 4096];
        loop {
            tokio::time::sleep(Duration::MILLISECOND * 50).await;
            if shared.get_playing() {
                shared.fill(&mut buffer);
            }
        }
    })
    .abort_handle()
}

fn audio_stream(shared: Arc<AudioShared>, channels: u32, sample_rate: u32) -> Result<cpal::Stream> {
    use cpal::traits::{DeviceTrait, HostTrait};

    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| {
        Error::new(
            ErrorKind::Internal,
            "failed to obtain default output device",
        )
    })?;
    //.context("failed to obtain default output device")?;
    let config = cpal::StreamConfig {
        channels: channels as u16,
        sample_rate: cpal::SampleRate(sample_rate),
        buffer_size: cpal::BufferSize::Fixed(4096),
    };
    let stream = device
        .build_output_stream(&config, audio_write(shared), audio_error, None)
        .map_err(Error::wrap)?;
    Ok(stream)
}

fn audio_write(
    shared: Arc<AudioShared>,
) -> impl FnMut(&mut [f32], &cpal::OutputCallbackInfo) + Send + 'static {
    use cpal::Sample;
    move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
        if !shared.get_playing() {
            //tracing::trace!("filling with silence");
            for sample in data {
                *sample = f32::EQUILIBRIUM;
            }
            return;
        }
        shared.fill(data);
    }
}

fn audio_error(err: cpal::StreamError) {
    eprintln!("an error occurred on stream: {}", err);
}

fn audio_sample_buffer() -> symphonia::core::audio::SampleBuffer<f32> {
    symphonia::core::audio::SampleBuffer::<f32>::new(
        8192,
        symphonia::core::audio::SignalSpec {
            rate: SAMPLE_RATE,
            channels: symphonia::core::audio::Channels::FRONT_LEFT
                | symphonia::core::audio::Channels::FRONT_RIGHT,
        },
    )
}

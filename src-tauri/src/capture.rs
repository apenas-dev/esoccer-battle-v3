use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Maximum buffer capacity: ~5 seconds at 16kHz mono = 80,000 samples
pub const BUFFER_CAPACITY: usize = 80_000;

#[derive(Debug, Clone)]
pub struct CaptureConfig {
    pub device_name: Option<String>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl Default for CaptureConfig {
    fn default() -> Self {
        Self {
            device_name: None,
            sample_rate: 16000,
            channels: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Handle to a running audio capture.
/// The actual cpal::Stream lives on a dedicated thread (via `stream_handle`).
/// FIX 2: No unsafe impl Send — the stream stays on its owning thread.
pub struct CaptureStream {
    /// Shared buffer written to by the audio callback
    buffer: Arc<Mutex<Vec<f32>>>,
    /// Signals the callback to stop writing
    is_active: Arc<AtomicBool>,
    /// FIX 3: Signals the callback we're draining (it should stop writing)
    draining: Arc<AtomicBool>,
    /// Channel to send the stop signal; None once already stopped
    stop_tx: Option<std::sync::mpsc::Sender<()>>,
    /// Channel to receive the final buffer after stop
    result_rx: std::sync::mpsc::Receiver<StopResult>,
    /// FIX 4: Actual sample rate and channels used by the stream
    sample_rate: u32,
    channels: u16,
}

type StopResult = Result<AudioBuffer, CaptureError>;

#[derive(Debug)]
pub enum CaptureError {
    NoDevice,
    DeviceNotFound(String),
    StreamError(String),
    ConfigError(String),
}

impl std::fmt::Display for CaptureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaptureError::NoDevice => write!(f, "No audio input device available"),
            CaptureError::DeviceNotFound(s) => write!(f, "Device not found: {}", s),
            CaptureError::StreamError(s) => write!(f, "Stream error: {}", s),
            CaptureError::ConfigError(s) => write!(f, "Config error: {}", s),
        }
    }
}

impl CaptureStream {
    /// Start audio capture. The cpal::Stream lives entirely on the spawned thread
    /// so we never need `unsafe impl Send`.
    pub fn start(config: CaptureConfig) -> Result<Self, CaptureError> {
        let host = cpal::default_host();
        let device = match &config.device_name {
            Some(name) => host
                .input_devices()
                .map_err(|_| CaptureError::NoDevice)?
                .find(|d: &cpal::Device| {
                    d.name()
                        .map(|n| n.to_lowercase().contains(&name.to_lowercase()))
                        .unwrap_or(false)
                })
                .ok_or_else(|| CaptureError::DeviceNotFound(name.clone()))?,
            None => host
                .default_input_device()
                .ok_or(CaptureError::NoDevice)?,
        };

        let supported_config = device
            .supported_input_configs()
            .map_err(|e: cpal::SupportedStreamConfigsError| {
                CaptureError::StreamError(e.to_string())
            })?
            .find(|c: &cpal::SupportedStreamConfigRange| {
                c.sample_format() == cpal::SampleFormat::F32
            })
            .ok_or_else(|| {
                CaptureError::ConfigError("No supported F32 input config found".into())
            })?
            .with_max_sample_rate();

        let stream_config: cpal::StreamConfig = supported_config.into();

        // FIX 4: Store actual sample rate and channels from the stream config
        let actual_sample_rate = stream_config.sample_rate.0;
        let actual_channels = stream_config.channels;

        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let is_active = Arc::new(AtomicBool::new(true));
        let draining = Arc::new(AtomicBool::new(false));

        // FIX 2: Stream lives on a dedicated thread — no unsafe Send needed
        let (stop_tx, stop_rx) = std::sync::mpsc::channel::<()>();
        let (result_tx, result_rx) = std::sync::mpsc::channel::<StopResult>();

        let buffer_cb = buffer.clone();
        let is_active_cb = is_active.clone();
        let draining_cb = draining.clone();
        let draining_stop = draining.clone();
        let buffer_read = buffer.clone();

        std::thread::spawn(move || {
            let err_fn = |err: cpal::StreamError| {
                tracing::error!("Audio stream error: {}", err);
            };

            let stream_result = device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    // FIX 3: Don't write if draining or inactive
                    if !is_active_cb.load(Ordering::SeqCst)
                        || draining_cb.load(Ordering::SeqCst)
                    {
                        return;
                    }
                    let mut buf = buffer_cb.lock().unwrap();
                    buf.extend_from_slice(data);
                    // HIGH-2: Ring buffer — discard oldest samples if over capacity
                    if buf.len() > BUFFER_CAPACITY {
                        let excess = buf.len() - BUFFER_CAPACITY;
                        buf.drain(..excess);
                    }
                },
                err_fn,
                None,
            );

            match stream_result {
                Ok(stream) => {
                    if let Err(e) = stream.play() {
                        let _ = result_tx
                            .send(Err(CaptureError::StreamError(e.to_string())));
                        return;
                    }

                    // Wait for stop signal
                    let _ = stop_rx.recv();

                    // FIX 3: Mark draining so callback stops writing, then wait briefly
                    draining_stop.store(true, Ordering::SeqCst);
                    std::thread::sleep(std::time::Duration::from_millis(30));

                    // Drop stream to ensure callback is fully stopped
                    drop(stream);

                    // Read final buffer
                    let samples = buffer_read
                        .lock()
                        .map(|mut b| std::mem::take(&mut *b))
                        .unwrap_or_default();

                    let _ = result_tx.send(Ok(AudioBuffer {
                        samples,
                        sample_rate: actual_sample_rate,
                        channels: actual_channels,
                    }));
                }
                Err(e) => {
                    let _ = result_tx
                        .send(Err(CaptureError::StreamError(e.to_string())));
                }
            }
        });

        Ok(Self {
            buffer,
            is_active,
            draining,
            stop_tx: Some(stop_tx),
            result_rx,
            sample_rate: actual_sample_rate,
            channels: actual_channels,
        })
    }

    /// Stop capture and return the collected audio buffer.
    pub fn stop(mut self) -> Result<AudioBuffer, CaptureError> {
        self.is_active.store(false, Ordering::SeqCst);

        // Signal the stream thread to stop
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }

        // Wait for the result from the stream thread
        self.result_rx
            .recv()
            .map_err(|_| CaptureError::StreamError("Stream thread panicked".into()))?
    }

    pub fn list_devices() -> Result<Vec<String>, CaptureError> {
        let host = cpal::default_host();
        let devices: Vec<String> = host
            .input_devices()
            .map_err(|_| CaptureError::NoDevice)?
            .filter_map(|d: cpal::Device| d.name().ok())
            .collect();

        Ok(devices)
    }
}

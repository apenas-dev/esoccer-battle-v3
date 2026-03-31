use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

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

pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

/// Wrapper that asserts Send — cpal::Stream isn't Send on all platforms,
/// but it's safe to use from a single thread via Mutex in our context.
pub struct CaptureStream {
    inner: Option<CaptureStreamInner>,
}

struct CaptureStreamInner {
    buffer: Arc<Mutex<Vec<f32>>>,
    is_active: Arc<AtomicBool>,
    _stream: cpal::Stream,
}

// SAFETY: Access is serialized through Mutex in VoiceCoordinator/AppState
unsafe impl Send for CaptureStream {}
unsafe impl Send for CaptureStreamInner {}

impl CaptureStream {
    pub fn start(config: CaptureConfig) -> Result<Self, CaptureError> {
        let host = cpal::default_host();
        let device = if let Some(ref name) = config.device_name {
            host.input_devices()
                .map_err(|e| CaptureError::ConfigError(e.to_string()))?
                .find(|d| d.name().map(|n| n == *name).unwrap_or(false))
                .ok_or_else(|| CaptureError::DeviceNotFound(name.clone()))?
        } else {
            host.default_input_device().ok_or(CaptureError::NoDevice)?
        };

        let mut supported_configs = device.supported_input_configs()
            .map_err(|e| CaptureError::ConfigError(e.to_string()))?;

        let supported_config = supported_configs.next()
            .ok_or(CaptureError::NoDevice)?;

        let config: cpal::StreamConfig = supported_config
            .with_sample_rate(cpal::SampleRate(config.sample_rate))
            .into();

        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let is_active = Arc::new(AtomicBool::new(true));

        let buffer_clone = buffer.clone();
        let stream = device.build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if let Ok(mut buf) = buffer_clone.lock() {
                    buf.extend_from_slice(data);
                }
            },
            |err: cpal::StreamError| {
                eprintln!("Capture stream error: {}", err);
            },
            None,
        ).map_err(|e| CaptureError::StreamError(e.to_string()))?;

        stream.play().map_err(|e| CaptureError::StreamError(e.to_string()))?;

        Ok(Self { inner: Some(CaptureStreamInner { _stream: stream, buffer, is_active }) })
    }

    pub fn stop(mut self) -> Result<AudioBuffer, CaptureError> {
        if let Some(inner) = self.inner.take() {
            inner.is_active.store(false, Ordering::SeqCst);
            drop(inner._stream);
            let samples = inner.buffer.lock().map_err(|e| CaptureError::StreamError(e.to_string()))?;
            Ok(AudioBuffer {
                samples: samples.clone(),
                sample_rate: 16000,
                channels: 1,
            })
        } else {
            Err(CaptureError::StreamError("Already stopped".to_string()))
        }
    }

    pub fn list_devices() -> Result<Vec<String>, CaptureError> {
        let host = cpal::default_host();
        let mut devices = Vec::new();
        if let Ok(input_devices) = host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    devices.push(name);
                }
            }
        }
        Ok(devices)
    }
}

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
            CaptureError::NoDevice => write!(f, "No audio input device found"),
            CaptureError::DeviceNotFound(n) => write!(f, "Device not found: {}", n),
            CaptureError::StreamError(e) => write!(f, "Stream error: {}", e),
            CaptureError::ConfigError(e) => write!(f, "Config error: {}", e),
        }
    }
}

impl std::error::Error for CaptureError {}

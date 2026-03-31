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

pub struct CaptureStream {
    buffer: Arc<Mutex<Vec<f32>>>,
    is_active: Arc<AtomicBool>,
    _stream: Option<cpal::Stream>,
}

impl CaptureStream {
    pub fn start(config: CaptureConfig) -> Result<Self, CaptureError> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or(CaptureError::NoDevice)?;

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

        Ok(Self { _stream: Some(stream), buffer, is_active })
    }

    pub fn stop(self) -> Result<AudioBuffer, CaptureError> {
        self.is_active.store(false, Ordering::SeqCst);
        drop(self._stream);
        let samples = self.buffer.lock().map_err(|e| CaptureError::StreamError(e.to_string()))?;
        Ok(AudioBuffer {
            samples: samples.clone(),
            sample_rate: 16000,
            channels: 1,
        })
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

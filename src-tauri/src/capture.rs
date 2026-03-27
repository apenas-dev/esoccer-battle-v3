use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

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

/// Wrapper to make cpal::Stream Send (cpal's !Send is overly conservative on Linux)
struct SendSafeStream(cpal::Stream);
unsafe impl Send for SendSafeStream {}

pub struct CaptureStream {
    stream: Option<SendSafeStream>,
    buffer: Arc<Mutex<Vec<f32>>>,
    is_active: Arc<AtomicBool>,
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
            CaptureError::NoDevice => write!(f, "No audio input device available"),
            CaptureError::DeviceNotFound(s) => write!(f, "Device not found: {}", s),
            CaptureError::StreamError(s) => write!(f, "Stream error: {}", s),
            CaptureError::ConfigError(s) => write!(f, "Config error: {}", s),
        }
    }
}

impl CaptureStream {
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
            .map_err(|e: cpal::SupportedStreamConfigsError| CaptureError::StreamError(e.to_string()))?
            .find(|c: &cpal::SupportedStreamConfigRange| c.sample_format() == cpal::SampleFormat::F32)
            .ok_or_else(|| {
                CaptureError::ConfigError("No supported F32 input config found".into())
            })?
            .with_max_sample_rate();

        let stream_config: cpal::StreamConfig = supported_config.into();

        let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
        let is_active = Arc::new(AtomicBool::new(true));
        let buffer_clone = buffer.clone();

        let err_fn = |err: cpal::StreamError| {
            tracing::error!("Audio stream error: {}", err);
        };

        let stream = device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    let mut buf = buffer_clone.lock().unwrap();
                    buf.extend_from_slice(data);
                },
                err_fn,
                None,
            )
            .map_err(|e: cpal::BuildStreamError| CaptureError::StreamError(e.to_string()))?;

        stream.play().map_err(|e: cpal::PlayStreamError| CaptureError::StreamError(e.to_string()))?;

        Ok(Self {
            stream: Some(SendSafeStream(stream)),
            buffer,
            is_active,
        })
    }

    pub fn stop(mut self) -> Result<AudioBuffer, CaptureError> {
        self.is_active.store(false, Ordering::SeqCst);
        if let Some(SendSafeStream(stream)) = self.stream.take() {
            drop(stream);
        }

        let samples = self
            .buffer
            .lock()
            .map(|mut b| std::mem::take(&mut *b))
            .unwrap_or_default();

        Ok(AudioBuffer {
            samples,
            sample_rate: 16000,
            channels: 1,
        })
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

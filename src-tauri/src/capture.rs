//! Microphone audio capture module.
//!
//! Captures audio from a mic, resamples to 16 kHz mono F32, and buffers
//! samples in a ring buffer for downstream consumers (e.g. Whisper STT).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use cpal::traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _};

// ── Constants ────────────────────────────────────────────────────────────

/// Target sample rate for Whisper.
const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Ring-buffer capacity: ~5 seconds of audio at 16 kHz.
const BUFFER_CAPACITY: usize = TARGET_SAMPLE_RATE as usize * 5;

// ── Public types ─────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct DeviceResult {
    pub name: String,
}

/// Shared audio buffer backed by a ring buffer.
pub type AudioBuffer = Arc<Mutex<VecDeque<f32>>>;

/// Handle to a running capture stream.
pub struct AudioStream {
    /// Shared PCM samples (16 kHz mono F32, newest at the back).
    pub buffer: AudioBuffer,
    shutdown: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<Result<(), String>>>,
}

impl AudioStream {
    /// Stop capture and join the background thread.
    pub fn stop(mut self) -> Result<(), String> {
        self.shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self.thread_handle.take() {
            handle
                .join()
                .map_err(|_| "Capture thread panicked".to_string())?
        } else {
            Ok(())
        }
    }
}

// ── Public functions ─────────────────────────────────────────────────────

/// List all available microphone device names across audio hosts.
pub fn list_microphone() -> Vec<DeviceResult> {
    all_hosts()
        .into_iter()
        .map(|host| host.input_devices())
        .filter_map(|d| d.ok())
        .flat_map(|d| d.collect::<Vec<_>>())
        .filter_map(|d| d.name().ok())
        .map(|name| DeviceResult { name })
        .collect()
}

/// Start capturing audio from a microphone.
///
/// * `device_name` — if `Some(name)`, picks the first mic whose name
///   contains the given string (case-insensitive). If `None`, uses the
///   system default input device.
///
/// Returns an [`AudioStream`] whose `.buffer` field can be read from
/// any thread. The caller **must** call `AudioStream::stop()` to shut
/// down gracefully.
pub fn start_capture(device_name: Option<String>) -> Result<AudioStream, String> {
    let host = cpal::default_host();
    let device = match &device_name {
        Some(name) => find_device_by_name(&host, name)?,
        None => host
            .default_input_device()
            .ok_or_else(|| "No default input device available".to_string())?,
    };

    let device_label = device
        .name()
        .unwrap_or_else(|_| "<unnamed>".to_string());

    log::info!("Capturing from: {device_label}");

    let supported = device
        .supported_input_configs()
        .map_err(|e| format!("Failed to query input configs for {device_label}: {e}"))?
        .find(|c| c.sample_format() == cpal::SampleFormat::F32)
        .ok_or_else(|| {
            format!(
                "Device {device_label} does not support F32 sample format"
            )
        })?;

    let source_rate = supported.min_sample_rate().0;
    let channels = supported.channels() as f32;

    let config: cpal::StreamConfig = supported.with_sample_rate(cpal::SampleRate(TARGET_SAMPLE_RATE)).into();

    let buffer: AudioBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(BUFFER_CAPACITY)));
    let shutdown = Arc::new(AtomicBool::new(false));

    let buf_clone = buffer.clone();
    let shutdown_clone = shutdown.clone();

    let stream = device
        .build_input_stream(
            &config,
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                if shutdown_clone.load(Ordering::Relaxed) {
                    return;
                }
                // Mix channels down to mono and resample.
                let mono: Vec<f32> = data
                    .chunks_exact(channels as usize)
                    .map(|frame| frame.iter().sum::<f32>() / channels)
                    .collect();

                let resampled = resample(&mono, source_rate, TARGET_SAMPLE_RATE);

                let mut buf = buf_clone.lock().unwrap_or_else(|e| e.into_inner());
                for s in resampled {
                    if buf.len() >= BUFFER_CAPACITY {
                        buf.pop_front();
                    }
                    buf.push_back(s);
                }
            },
            |err| {
                log::error!("Input stream error: {err}");
            },
            None,
        )
        .map_err(|e| format!("Failed to build input stream: {e}"))?;

    stream
        .play()
        .map_err(|e| format!("Failed to start input stream: {e}"))?;

    // Keep the stream alive in a dedicated thread so it isn't dropped.
    let shutdown_thread = shutdown.clone();
    let handle = thread::Builder::new()
        .name("audio-capture".into())
        .spawn(move || {
            // Spin until shutdown; the stream lives on this stack frame.
            while !shutdown_thread.load(Ordering::Relaxed) {
                thread::sleep(std::time::Duration::from_millis(100));
            }
            drop(stream);
            Ok(())
        })
        .map_err(|e| format!("Failed to spawn capture thread: {e}"))?;

    Ok(AudioStream {
        buffer,
        shutdown,
        thread_handle: Some(handle),
    })
}

// ── Private helpers ──────────────────────────────────────────────────────

fn all_hosts() -> Vec<cpal::Host> {
    cpal::ALL_HOSTS
        .iter()
        .map(|id| cpal::host_from_id(*id))
        .filter_map(|h| h.ok())
        .collect()
}

/// Find an input device whose name contains `needle` (case-insensitive).
fn find_device_by_name(
    host: &cpal::Host,
    needle: &str,
) -> Result<cpal::Device, String> {
    let devices = host
        .input_devices()
        .map_err(|e| format!("Failed to enumerate input devices: {e}"))?;

    let needle_lower = needle.to_lowercase();
    for dev in devices {
        if let Ok(name) = dev.name() {
            if name.to_lowercase().contains(&needle_lower) {
                return Ok(dev);
            }
        }
    }
    Err(format!("No input device matching '{needle}'"))
}

/// Linear-interpolation resample from `from_rate` to `to_rate`.
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    if from_rate == to_rate {
        return samples.to_vec();
    }
    let ratio = from_rate as f64 / to_rate as f64;
    let out_len = ((samples.len() as f64) / ratio).round() as usize;
    let mut out = Vec::with_capacity(out_len);
    for i in 0..out_len {
        let src_idx = i as f64 * ratio;
        let lo = src_idx.floor() as usize;
        let hi = (lo + 1).min(samples.len() - 1);
        let frac = src_idx - lo as f64;
        out.push(samples[lo] * (1.0 - frac as f32) + samples[hi] * frac as f32);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resample_same_rate() {
        let data = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(resample(&data, 16000, 16000), data);
    }

    #[test]
    fn resample_downsample() {
        let data = vec![0.0, 1.0, 2.0, 3.0]; // 4 samples @ 48 kHz
        let out = resample(&data, 48000, 16000); // expect ~1.33 samples → rounded
        assert!(out.len() >= 1);
    }
}

//! Microphone audio capture module.
//!
//! Captures audio from a mic, resamples to 16 kHz mono F32, and buffers
//! samples in a ring buffer for downstream consumers (e.g. Whisper STT).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait as _, HostTrait as _, StreamTrait as _};

// ── Constants ────────────────────────────────────────────────────────────

/// Target sample rate for Whisper.
const TARGET_SAMPLE_RATE: u32 = 16_000;

/// Ring-buffer capacity: ~5 seconds of audio at 16 kHz.
const BUFFER_CAPACITY: usize = TARGET_SAMPLE_RATE as usize * 5;

/// Timeout for stream creation inside the capture thread.
const STREAM_READY_TIMEOUT_SECS: u64 = 5;

// ── Public types ─────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
pub struct DeviceResult {
    pub name: String,
}

/// Shared audio buffer backed by a ring buffer.
pub type AudioBuffer = Arc<Mutex<VecDeque<f32>>>;

/// Handle to a running capture stream.
///
/// Dropping this value will signal the background thread to stop and join it.
pub struct AudioStream {
    /// Shared PCM samples (16 kHz mono F32, newest at the back).
    pub buffer: AudioBuffer,
    shutdown: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<Result<(), String>>>,
}

impl AudioStream {
    /// Drain all samples from the buffer and return them as a `Vec<f32>`.
    /// The buffer is left empty after this call.
    pub fn drain_buffer(&self) -> Vec<f32> {
        let mut buf = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        buf.drain(..).collect()
    }
}

impl Drop for AudioStream {
    fn drop(&mut self) {
        tracing::info!("[capture] Dropping AudioStream — signalling shutdown");
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            match handle.join() {
                Ok(Ok(())) => tracing::info!("[capture] Capture thread joined cleanly"),
                Ok(Err(e)) => tracing::warn!("[capture] Capture thread exited with error: {e}"),
                Err(_) => tracing::warn!("[capture] Capture thread panicked or was already joined"),
            }
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
/// any thread. The stream is automatically stopped when the `AudioStream`
/// is dropped.
///
/// This function now waits for the capture thread to confirm that the
/// audio stream was successfully created before returning Ok. It also
/// validates sample rate support and resamples when necessary.
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

    tracing::info!("[capture] Device selected: {device_label}");

    let supported = device
        .supported_input_configs()
        .map_err(|e| format!("Failed to query input configs for {device_label}: {e}"))?
        .find(|c| c.sample_format() == cpal::SampleFormat::F32)
        .ok_or_else(|| {
            format!(
                "Device {device_label} does not support F32 sample format"
            )
        })?;

    let channels = supported.channels();
    tracing::info!(
        "[capture] Supported config: F32, channels={channels}, rate={}-{}",
        supported.min_sample_rate().0,
        supported.max_sample_rate().0,
    );

    // Determine actual sample rate: use 16kHz if supported, otherwise use nearest
    let actual_rate = if supported.min_sample_rate().0 <= TARGET_SAMPLE_RATE
        && supported.max_sample_rate().0 >= TARGET_SAMPLE_RATE
    {
        TARGET_SAMPLE_RATE
    } else {
        // Use closest rate to target
        let min = supported.min_sample_rate().0;
        let max = supported.max_sample_rate().0;
        if TARGET_SAMPLE_RATE < min {
            min
        } else {
            max
        }
    };

    let needs_resample = actual_rate != TARGET_SAMPLE_RATE;
    tracing::info!(
        "[capture] Using sample rate: {actual_rate} (resample={needs_resample})"
    );

    let config: cpal::StreamConfig = supported
        .with_sample_rate(cpal::SampleRate(actual_rate))
        .into();

    let buffer: AudioBuffer = Arc::new(Mutex::new(VecDeque::with_capacity(BUFFER_CAPACITY)));
    let shutdown = Arc::new(AtomicBool::new(false));

    // Channel for the capture thread to signal stream creation success/failure
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

    let buf_clone = buffer.clone();
    let shutdown_for_cb = shutdown.clone();
    let shutdown_for_thread = shutdown.clone();
    let device_label_clone = device_label.clone();
    let actual_rate_for_cb = actual_rate;

    // Stream must be created and held inside the thread (cpal::Stream is !Send).
    let handle = thread::Builder::new()
        .name("audio-capture".into())
        .spawn(move || -> Result<(), String> {
            let shutdown_for_cb = shutdown_for_cb;
            let stream = device
                .build_input_stream(
                    &config,
                    move |data: &[f32], _: &cpal::InputCallbackInfo| {
                        if shutdown_for_cb.load(Ordering::SeqCst) {
                            return;
                        }
                        // Mix channels down to mono and resample.
                        let mono: Vec<f32> = data
                            .chunks_exact(channels as usize)
                            .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                            .collect();

                        let resampled = if needs_resample {
                            resample(&mono, actual_rate_for_cb, TARGET_SAMPLE_RATE)
                        } else {
                            mono
                        };

                        let mut buf = match buf_clone.lock() {
                            Ok(b) => b,
                            Err(e) => {
                                tracing::warn!("[capture] Audio buffer lock poisoned: {e}");
                                return;
                            }
                        };

                        let buf_len = buf.len();
                        tracing::trace!(
                            "[capture] Audio frame: {} samples, buffer={buf_len}",
                            resampled.len()
                        );
                        if buf_len > (BUFFER_CAPACITY * 80 / 100) {
                            tracing::warn!(
                                "[capture] ⚠️ Buffer near capacity: {buf_len}/{BUFFER_CAPACITY}"
                            );
                        }

                        for s in resampled {
                            if buf.len() >= BUFFER_CAPACITY {
                                buf.pop_front();
                            }
                            buf.push_back(s);
                        }
                    },
                    move |err| {
                        tracing::warn!("[capture] Stream error on {device_label_clone}: {err}");
                    },
                    None,
                )
                .map_err(|e| format!("Failed to build input stream: {e}"))?;

            tracing::info!("[capture] Stream created successfully");

            stream
                .play()
                .map_err(|e| format!("Failed to start input stream: {e}"))?;

            // Signal success to the caller
            let _ = ready_tx.send(Ok(()));
            tracing::info!("[capture] ✅ Ready — capturing audio");

            // Keep stream alive until shutdown.
            while !shutdown_for_thread.load(Ordering::SeqCst) {
                thread::sleep(Duration::from_millis(100));
            }
            drop(stream);
            Ok(())
        })
        .map_err(|e| format!("Failed to spawn capture thread: {e}"))?;

    // Wait for the stream to be created successfully (with timeout)
    match ready_rx.recv_timeout(Duration::from_secs(STREAM_READY_TIMEOUT_SECS)) {
        Ok(Ok(())) => {
            tracing::info!("[capture] ✅ Capture thread confirmed stream is running");
        }
        Ok(Err(e)) => {
            // Thread sent an error — join to clean up
            let _ = handle.join();
            return Err(format!("Stream creation failed: {e}"));
        }
        Err(_) => {
            tracing::error!("[capture] ❌ Stream creation failed or timed out ({STREAM_READY_TIMEOUT_SECS}s)");
            return Err(format!(
                "Capture stream did not start within {STREAM_READY_TIMEOUT_SECS}s"
            ));
        }
    }

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

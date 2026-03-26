//! Streaming voice transcription pipeline.
//!
//! Periodically extracts audio chunks from the shared ring buffer,
//! runs Whisper inference, and emits Tauri events with recognised text.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;
use tauri::{AppHandle, Emitter};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::buffer;
use crate::capture::AudioBuffer;

// ── Whisper parameters ───────────────────────────────────────────────────

const CHUNK_SECS: f32 = buffer::DEFAULT_CHUNK_SECS;
const OVERLAP_SECS: f32 = buffer::DEFAULT_OVERLAP_SECS;
const LOOP_INTERVAL_MS: u64 = 500;

// ── Model definitions ────────────────────────────────────────────────────

pub trait ModelDirectory {
    fn transcriber_model_dir(&self) -> PathBuf;
}

impl ModelDirectory for directories::ProjectDirs {
    fn transcriber_model_dir(&self) -> PathBuf {
        let dir = self.cache_dir().join("model");
        std::fs::create_dir_all(&dir)
            .map_err(|e| tracing::warn!("Failed to create model dir: {e}"))
            .ok();
        dir
    }
}

#[derive(Debug, EnumIter, Serialize)]
pub enum Category {
    Recommended,
    Other,
}

impl Category {
    pub fn name(&self) -> &'static str {
        match self {
            Category::Recommended => "Recommended Models",
            Category::Other => "Other Models",
        }
    }
}

pub enum ModelType {
    Whisper,
    Quantized,
}

impl ModelType {
    pub fn name(&self) -> &'static str {
        match self {
            ModelType::Whisper => "Whisper",
            ModelType::Quantized => "Quantized",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, EnumIter, Clone, Copy)]
pub enum Model {
    TinyWhisper,
    TinyEnWhisper,
    TinyQuantized,
    TinyEnQuantized,
    BaseWhisper,
    BaseEnWhisper,
    BaseQuantized,
    BaseEnQuantized,
}

impl Model {
    pub fn default_model() -> Self {
        Self::BaseWhisper
    }

    pub fn recommended_models() -> Vec<Self> {
        vec![Self::TinyWhisper, Self::BaseWhisper]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::TinyWhisper => "Tiny",
            Self::TinyEnWhisper => "Tiny English",
            Self::TinyQuantized => "Tiny (Quantized)",
            Self::TinyEnQuantized => "Tiny English (Quantized)",
            Self::BaseWhisper => "Base",
            Self::BaseEnWhisper => "Base English",
            Self::BaseQuantized => "Base (Quantized)",
            Self::BaseEnQuantized => "Base English (Quantized)",
        }
    }

    pub fn download_url(&self) -> &'static str {
        match self {
            Self::TinyWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin?download=true",
            Self::TinyEnWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin?download=true",
            Self::TinyQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny-q5_1.bin?download=true",
            Self::TinyEnQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en-q5_1.bin?download=true",
            Self::BaseWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin?download=true",
            Self::BaseEnWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true",
            Self::BaseQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-q5_1.bin?download=true",
            Self::BaseEnQuantized => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base-en-q5_1.bin?download=true",
        }
    }

    pub fn file_name(&self) -> &'static str {
        match self {
            Self::TinyWhisper => "tiny.bin",
            Self::TinyEnWhisper => "tiny-en.bin",
            Self::TinyQuantized => "tiny-q.bin",
            Self::TinyEnQuantized => "tiny-en-q.bin",
            Self::BaseWhisper => "base.bin",
            Self::BaseEnWhisper => "base-en.bin",
            Self::BaseQuantized => "base-q.bin",
            Self::BaseEnQuantized => "base-en-q.bin",
        }
    }

    pub fn average_memory_usage(&self) -> usize {
        match self {
            Self::TinyWhisper | Self::TinyEnWhisper | Self::TinyQuantized | Self::TinyEnQuantized => 390,
            Self::BaseWhisper | Self::BaseEnWhisper | Self::BaseQuantized | Self::BaseEnQuantized => 500,
        }
    }

    /// Returns `"en"` for English-only models, `"pt"` for multilingual ones.
    pub fn default_language(&self) -> &'static str {
        match self {
            Self::TinyEnWhisper | Self::TinyEnQuantized
            | Self::BaseEnWhisper | Self::BaseEnQuantized => "en",
            _ => "pt",
        }
    }

    /// Checks whether the system has enough available memory to run this model.
    pub fn can_run(&self) -> bool {
        use sysinfo::{MemoryRefreshKind, RefreshKind, System};
        let system = System::new_with_specifics(
            RefreshKind::new().with_memory(MemoryRefreshKind::everything()),
        );
        let available = system.available_memory() as usize;
        (self.average_memory_usage() * 1_000_000) < available
    }

    pub fn disk_usage(&self) -> usize {
        match self {
            Self::TinyWhisper | Self::TinyEnWhisper => 77,
            Self::TinyQuantized | Self::TinyEnQuantized => 33,
            Self::BaseWhisper | Self::BaseEnWhisper => 148,
            Self::BaseQuantized | Self::BaseEnQuantized => 60,
        }
    }

    pub fn path(&self) -> PathBuf {
        crate::project_directory().transcriber_model_dir().join(self.file_name())
    }

    pub fn is_downloaded(&self) -> bool {
        self.path().exists()
    }

    pub fn category(&self) -> Category {
        match self {
            Self::TinyWhisper | Self::BaseWhisper => Category::Recommended,
            _ => Category::Other,
        }
    }

    pub fn model_type(&self) -> ModelType {
        match self {
            Self::TinyQuantized | Self::TinyEnQuantized
            | Self::BaseQuantized | Self::BaseEnQuantized => ModelType::Quantized,
            _ => ModelType::Whisper,
        }
    }
}

/// Load a WhisperContext from the downloaded model file.
pub fn load_context(model: &Model) -> anyhow::Result<WhisperContext> {
    let path = model.path();
    if !path.exists() {
        anyhow::bail!("Model {:?} is not downloaded at {:?}", model, path);
    }
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Model path contains invalid UTF-8"))?;
    let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
        .map_err(|e| anyhow::anyhow!("Failed to load Whisper model: {e}"))?;
    Ok(ctx)
}

// ── Voice Pipeline ───────────────────────────────────────────────────────

/// Handle to the running transcription loop.
///
/// Dropping this value will signal the background thread to stop and join it.
pub struct VoicePipeline {
    shutdown: Arc<AtomicBool>,
    thread_handle: Option<JoinHandle<Result<(), String>>>,
}

impl VoicePipeline {
    /// Start the transcription loop on a dedicated thread.
    ///
    /// The loop:
    /// 1. Loads the WhisperContext **once**.
    /// 2. Repeatedly extracts chunks from `buffer` and runs inference.
    /// 3. Emits `"voice_text"` Tauri events with recognised text.
    pub fn start(
        app_handle: AppHandle,
        audio_buffer: AudioBuffer,
        model: Model,
    ) -> Result<Self, String> {
        let shutdown = Arc::new(AtomicBool::new(false));

        let shutdown_clone = shutdown.clone();
        let handle = thread::Builder::new()
            .name("whisper-transcriber".into())
            .spawn(move || -> Result<(), String> {
                run_loop(app_handle, audio_buffer, model, &shutdown_clone)
            })
            .map_err(|e| format!("Failed to spawn transcription thread: {e}"))?;

        Ok(Self {
            shutdown,
            thread_handle: Some(handle),
        })
    }

    /// Signal the loop to stop. Waits up to 3 seconds for the thread to join.
    /// If it doesn't join in time, the thread is detached (no deadlock).
    pub fn stop(mut self) -> Result<(), String> {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            // Timeout join to prevent deadlock
            let result = thread::scope(|s| {
                s.spawn(|| handle.join()).join()
            });
            match result {
                Ok(Ok(Ok(()))) => Ok(()),
                Ok(Ok(Err(e))) => Err(e),
                Ok(Err(_)) => Err("Transcription thread panicked".to_string()),
                Err(_) => {
                    tracing::warn!("[transcriber] Thread join timed out — detached");
                    Ok(())
                }
            }
        } else {
            Ok(())
        }
    }
}

impl Drop for VoicePipeline {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            match handle.join() {
                Ok(Ok(())) => {}
                Ok(Err(e)) => tracing::warn!("[transcriber] Thread error on drop: {e}"),
                Err(_) => tracing::warn!("[transcriber] Transcription thread panicked during drop"),
            }
        }
    }
}

/// Core loop: load model once, transcribe chunks until shutdown.
fn run_loop(
    app_handle: AppHandle,
    audio_buffer: AudioBuffer,
    model: Model,
    shutdown: &AtomicBool,
) -> Result<(), String> {
    tracing::info!("[transcriber] Loading model {:?}…", model);
    let ctx = load_context(&model)
        .map_err(|e| format!("Failed to load model: {e}"))?;
    let mut state = ctx
        .create_state()
        .map_err(|e| format!("Failed to create Whisper state: {e}"))?;

    let language = model.default_language();

    tracing::info!("[transcriber] Model loaded (language={language}). Starting transcription loop.");

    while !shutdown.load(Ordering::SeqCst) {
        thread::sleep(std::time::Duration::from_millis(LOOP_INTERVAL_MS));

        let chunk = match buffer::extract_chunk(&audio_buffer, CHUNK_SECS, OVERLAP_SECS) {
            Some(c) => c,
            None => continue,
        };

        let params = build_params(language);

        if let Err(e) = state.full(params, &chunk) {
            tracing::warn!("[transcriber] Inference error: {e}");
            continue;
        }

        let text = match extract_text(&state) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("[transcriber] Failed to extract text: {e}");
                continue;
            }
        };

        let trimmed = text.trim();
        if trimmed.is_empty() {
            continue;
        }

        tracing::info!("[transcriber] Recognised: \"{trimmed}\"");

        if let Err(e) = app_handle.emit("voice_text", trimmed) {
            tracing::warn!("[transcriber] Failed to emit event: {e}");
        }
    }

    tracing::info!("[transcriber] Loop stopped.");
    Ok(())
}

/// Build Whisper inference parameters.
fn build_params(language: &'static str) -> FullParams<'static, 'static> {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(language));
    params.set_translate(false);
    params
}

/// Extract recognised text from the first segment.
fn extract_text(state: &whisper_rs::WhisperState) -> Result<String, String> {
    let n_segments = state
        .full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {e}"))?;
    if n_segments == 0 {
        return Ok(String::new());
    }
    state
        .full_get_segment_text(0)
        .map_err(|e| format!("Failed to get segment text: {e}"))
}

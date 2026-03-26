//! Streaming voice transcription pipeline.
//!
//! Periodically extracts audio chunks from the shared ring buffer,
//! runs Whisper inference, and emits Tauri events with recognised text.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

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

/// Timeout for model loading in the transcription thread.
const MODEL_LOAD_TIMEOUT_SECS: u64 = 30;

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
    MediumWhisper,
    MediumEnWhisper,
    SmallWhisper,
    SmallEnWhisper,
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
    /// Parse a model name from a string, supporting both friendly names ("base", "tiny")
    /// and serde variant names ("BaseWhisper", "TinyWhisper"). Case-insensitive.
    pub fn from_str_friendly(s: &str) -> Option<Self> {
        let lower = s.to_lowercase();
        let model = match lower.as_str() {
            "medium" | "mediumwhisper" => Self::MediumWhisper,
            "mediumenwhisper" | "medium_en" | "medium-en" | "mediumwhisperen" => Self::MediumEnWhisper,
            "small" | "smallwhisper" => Self::SmallWhisper,
            "smallenwhisper" | "small_en" | "small-en" | "smallwhisperen" => Self::SmallEnWhisper,
            "tiny" | "tinywhisper" => Self::TinyWhisper,
            "tinyenwhisper" | "tiny_en" | "tiny-en" | "tinywhisperen" => Self::TinyEnWhisper,
            "tinyquantized" | "tiny_quantized" | "tiny-q" => Self::TinyQuantized,
            "tinyenquantized" | "tiny_en_quantized" | "tiny-en-q" => Self::TinyEnQuantized,
            "base" | "basewhisper" => Self::BaseWhisper,
            "baseenwhisper" | "base_en" | "base-en" | "basewhisperen" => Self::BaseEnWhisper,
            "basequantized" | "base_quantized" | "base-q" => Self::BaseQuantized,
            "baseenquantized" | "base_en_quantized" | "base-en-q" => Self::BaseEnQuantized,
            _ => return None,
        };
        Some(model)
    }

    pub fn default_model() -> Self {
        Self::SmallWhisper
    }

    pub fn recommended_models() -> Vec<Self> {
        vec![Self::MediumWhisper, Self::SmallWhisper, Self::BaseWhisper]
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::MediumWhisper => "Medium",
            Self::MediumEnWhisper => "Medium English",
            Self::SmallWhisper => "Small",
            Self::SmallEnWhisper => "Small English",
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
            Self::MediumWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.bin?download=true",
            Self::MediumEnWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-medium.en.bin?download=true",
            Self::SmallWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.bin?download=true",
            Self::SmallEnWhisper => "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-small.en.bin?download=true",
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
            Self::MediumWhisper => "medium.bin",
            Self::MediumEnWhisper => "medium-en.bin",
            Self::SmallWhisper => "small.bin",
            Self::SmallEnWhisper => "small-en.bin",
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
            Self::MediumWhisper | Self::MediumEnWhisper => 1500,
            Self::SmallWhisper | Self::SmallEnWhisper => 1000,
            Self::TinyWhisper | Self::TinyEnWhisper | Self::TinyQuantized | Self::TinyEnQuantized => 390,
            Self::BaseWhisper | Self::BaseEnWhisper | Self::BaseQuantized | Self::BaseEnQuantized => 500,
        }
    }

    /// Returns `"en"` for English-only models, `"pt"` for multilingual ones.
    pub fn default_language(&self) -> &'static str {
        match self {
            Self::MediumEnWhisper | Self::SmallEnWhisper | Self::TinyEnWhisper | Self::TinyEnQuantized
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
            Self::MediumWhisper | Self::MediumEnWhisper => 1524,
            Self::SmallWhisper | Self::SmallEnWhisper => 491,
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
            Self::MediumWhisper | Self::SmallWhisper | Self::BaseWhisper => Category::Recommended,
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
    tracing::info!(
        "[transcriber] Loading model {:?} from {}",
        model,
        path.display()
    );
    let path_str = path
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Model path contains invalid UTF-8"))?;
    let ctx = WhisperContext::new_with_params(path_str, WhisperContextParameters::default())
        .map_err(|e| anyhow::anyhow!("Failed to load Whisper model: {e}"))?;

    // Report model size
    if let Ok(meta) = std::fs::metadata(&path) {
        let size_mb = meta.len() / (1024 * 1024);
        tracing::info!("[transcriber] ✅ Model loaded ({size_mb}MB)");
    } else {
        tracing::info!("[transcriber] ✅ Model loaded");
    }

    Ok(ctx)
}

// ── Voice Pipeline ───────────────────────────────────────────────────────

/// Handle to the running transcription loop.
///
/// Dropping this value will signal the background thread to stop.
/// Uses detached pattern to avoid blocking Drop.
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
    ///
    /// This now waits for model loading confirmation with a timeout.
    /// Returns Err if the model fails to load.
    pub fn start(
        app_handle: AppHandle,
        audio_buffer: AudioBuffer,
        model: Model,
    ) -> Result<Self, String> {
        let shutdown = Arc::new(AtomicBool::new(false));

        // Channel to get model-load success/failure
        let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();

        let shutdown_clone = shutdown.clone();
        let handle = thread::Builder::new()
            .name("whisper-transcriber".into())
            .spawn(move || -> Result<(), String> {
                run_loop(app_handle, audio_buffer, model, &shutdown_clone, ready_tx)
            })
            .map_err(|e| format!("Failed to spawn transcription thread: {e}"))?;

        // Wait for model to load (with timeout)
        match ready_rx.recv_timeout(Duration::from_secs(MODEL_LOAD_TIMEOUT_SECS)) {
            Ok(Ok(())) => {
                tracing::info!("[transcriber] ✅ Transcriber confirmed ready");
            }
            Ok(Err(e)) => {
                // Model load failed — join thread to clean up
                let _ = handle.join();
                return Err(format!("Model load failed: {e}"));
            }
            Err(_) => {
                tracing::error!(
                    "[transcriber] ❌ Model load timed out or channel disconnected ({}s)",
                    MODEL_LOAD_TIMEOUT_SECS
                );
                // Thread might still be loading — just detach it (shutdown will clean up)
                std::mem::forget(handle);
                return Err(format!(
                    "Model did not load within {MODEL_LOAD_TIMEOUT_SECS}s"
                ));
            }
        }

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
            let (tx, rx) = std::sync::mpsc::channel();
            thread::spawn(move || {
                let _ = handle.join();
                let _ = tx.send(());
            });
            match rx.recv_timeout(Duration::from_secs(3)) {
                Ok(()) => {}
                Err(_) => tracing::warn!("[transcriber] Thread join timed out — detached"),
            }
        }
        Ok(())
    }
}

impl Drop for VoicePipeline {
    fn drop(&mut self) {
        tracing::info!("[transcriber] Dropping VoicePipeline — signalling shutdown");
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            // Detach — stop() should have already been called with timeout before
            std::mem::forget(handle);
        }
    }
}

/// Core loop: load model once, transcribe chunks until shutdown.
fn run_loop(
    app_handle: AppHandle,
    audio_buffer: AudioBuffer,
    model: Model,
    shutdown: &AtomicBool,
    ready_tx: mpsc::Sender<Result<(), String>>,
) -> Result<(), String> {
    let ctx = match load_context(&model) {
        Ok(ctx) => ctx,
        Err(e) => {
            let err_msg = format!("Model load failed: {e}");
            tracing::error!("[transcriber] ❌ {err_msg}");
            let _ = ready_tx.send(Err(err_msg));
            return Err(format!("Failed to load model: {e}"));
        }
    };

    let mut state = ctx
        .create_state()
        .map_err(|e| {
            let err_msg = format!("Failed to create Whisper state: {e}");
            let _ = ready_tx.send(Err(err_msg.clone()));
            err_msg
        })?;

    let language = model.default_language();

    // Signal success — model is loaded and ready
    let _ = ready_tx.send(Ok(()));
    tracing::info!(
        "[transcriber] ✅ Ready — transcription loop running (lang={language})"
    );

    while !shutdown.load(Ordering::SeqCst) {
        thread::sleep(Duration::from_millis(LOOP_INTERVAL_MS));

        // Log buffer status
        let buf_len = audio_buffer
            .lock()
            .map(|b| b.len())
            .unwrap_or(0);
        let min_samples = (CHUNK_SECS * 16_000.0) as usize;
        tracing::trace!(
            "[transcriber] Buffer has {buf_len} samples (need {min_samples} for chunk)"
        );

        let chunk = match buffer::extract_chunk(&audio_buffer, CHUNK_SECS, OVERLAP_SECS) {
            Some(c) => c,
            None => continue,
        };

        tracing::info!(
            "[transcriber] Extracted chunk: {} samples",
            chunk.len()
        );

        let params = build_params(language);

        tracing::info!(
            "[transcriber] Inference started (chunk={} samples, lang={language})",
            chunk.len()
        );
        let start = Instant::now();

        if let Err(e) = state.full(params, &chunk) {
            tracing::warn!("[transcriber] Inference error: {e}");
            continue;
        }

        let elapsed_ms = start.elapsed().as_millis();
        tracing::info!(
            "[transcriber] Inference completed in {elapsed_ms}ms"
        );

        let text = match extract_text(&state) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("[transcriber] Failed to extract text: {e}");
                continue;
            }
        };

        let trimmed = text.trim();
        if trimmed.is_empty() {
            tracing::trace!("[transcriber] Empty result from inference (silence?)");
            continue;
        }

        tracing::info!("[transcriber] ✅ Recognised: \"{trimmed}\"");

        // Emit as JSON object — frontend expects { "text": "..." }
        if let Err(e) = app_handle.emit("voice_text", serde_json::json!({ "text": trimmed })) {
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
    match state.get_segment(0) {
        Some(seg) => Ok(seg.to_str().unwrap_or_default().to_string()),
        None => Ok(String::new()),
    }
}

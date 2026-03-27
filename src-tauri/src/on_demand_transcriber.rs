//! On-demand (push-to-talk) transcription module.
//!
//! Provides a one-shot transcription function backed by a lazily-loaded,
//! swappable singleton `WhisperContext`. Unlike the streaming pipeline in
//! `transcriber.rs`, this runs inference exactly once on a provided audio
//! buffer. If the requested model differs from the currently loaded one,
//! the old context is dropped and a new one is created.

use std::sync::Mutex;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use crate::transcriber;

// ── Singleton model (swappable) ─────────────────────────────────────────

/// Holds the loaded Whisper context and which model it was built from.
struct LoadedContext {
    model: transcriber::Model,
    context: WhisperContext,
}

static WHISPER_CONTEXT: Mutex<Option<LoadedContext>> = Mutex::new(None);

/// Return a reference to the global `WhisperContext`, loading or swapping as needed.
///
/// If the requested `model` differs from the currently cached one, the old
/// context is dropped and a fresh instance is created from disk.
pub fn get_or_load_context(
    model: &transcriber::Model,
) -> Result<std::sync::MutexGuard<'static, Option<LoadedContext>>, String> {
    let mut guard = WHISPER_CONTEXT.lock().map_err(|e| format!("WhisperContext lock poisoned: {e}"))?;

    let needs_reload = match guard.as_ref() {
        None => true,
        Some(loaded) => loaded.model.model_type() != model.model_type(),
    };

    if needs_reload {
        tracing::info!(
            "[on-demand] Loading Whisper model (request={:?}, cached={:?})",
            model,
            guard.as_ref().map(|l| &l.model),
        );
        let ctx = transcriber::load_context(model).map_err(|e| format!("{e:#}"))?;
        *guard = Some(LoadedContext {
            model: model.clone(),
            context: ctx,
        });
    }

    Ok(guard)
}

// ── One-shot transcription ───────────────────────────────────────────────

/// Run Whisper inference **once** on the provided audio samples.
///
/// * `audio` — 16 kHz mono F32 PCM samples.
/// * `model`  — used only on the first call to load the model.
/// * `language` — ISO 639-1 code (e.g. `"pt"` or `"en"`).
///
/// Returns the concatenated text from all recognised segments, trimmed.
pub fn transcribe_once(
    audio: &[f32],
    model: &transcriber::Model,
    language: &str,
) -> Result<String, String> {
    tracing::info!(
        "[on-demand] Starting transcription ({} samples, lang={language})",
        audio.len()
    );

    let loaded = get_or_load_context(model)?;
    let ctx = loaded
        .as_ref()
        .expect("context was just loaded")
        .context
        .as_ref();
    tracing::info!("[on-demand] Whisper context ready");

    let mut state = ctx
        .create_state()
        .map_err(|e| format!("Failed to create Whisper state: {e}"))?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(language));
    params.set_translate(false);

    tracing::info!("[on-demand] Running full inference…");
    state
        .full(params, audio)
        .map_err(|e| format!("Whisper inference failed: {e}"))?;

    // Collect text from every segment
    let mut text = String::new();
    let n_segments = state.full_n_segments();

    for i in 0..n_segments {
        if let Some(seg) = state.get_segment(i) {
            if let Ok(s) = seg.to_str() {
                if !s.is_empty() {
                    if !text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(s.trim());
                }
            }
        }
    }

    let trimmed = text.trim().to_string();
    tracing::info!(
        "[on-demand] Transcription complete — {} chars: \"{trimmed}\"",
        trimmed.len()
    );

    Ok(trimmed)
}

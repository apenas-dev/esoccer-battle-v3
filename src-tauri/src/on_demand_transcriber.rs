//! On-demand (push-to-talk) transcription module.
//!
//! Provides a one-shot transcription function backed by a lazily-loaded
//! singleton `WhisperContext`. Unlike the streaming pipeline in
//! `transcriber.rs`, this runs inference exactly once on a provided audio
//! buffer.

use std::sync::OnceLock;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext};

use crate::transcriber;

// ── Singleton model ──────────────────────────────────────────────────────

static WHISPER_CONTEXT: OnceLock<WhisperContext> = OnceLock::new();

/// Return a reference to the lazily-initialised global `WhisperContext`.
///
/// On the first call the model file is loaded from disk. Subsequent calls
/// return the already-loaded instance (zero-copy).
pub fn get_or_load_context(
    model: &transcriber::Model,
) -> Result<&'static WhisperContext, String> {
    if let Some(ctx) = WHISPER_CONTEXT.get() {
        return Ok(ctx);
    }
    let ctx = transcriber::load_context(model).map_err(|e| format!("{e:#}"))?;
    // Safety: we just checked it's None and we're the only writer
    // (OnceLock.set returns Err only if already set, which can't race here).
    let _ = WHISPER_CONTEXT.set(ctx);
    Ok(WHISPER_CONTEXT.get().unwrap())
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

    let ctx = get_or_load_context(model)?;
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

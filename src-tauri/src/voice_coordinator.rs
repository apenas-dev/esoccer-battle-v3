use crate::capture::{CaptureConfig, CaptureStream};
use crate::config::WhisperModel;
use tauri::{AppHandle, Emitter};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Lazy-loaded Whisper context. Loads once, reuses forever.
static WHISPER_CTX: std::sync::OnceLock<Result<WhisperContext, String>> = std::sync::OnceLock::new();

/// Try to load the Whisper model from known paths.
fn load_whisper_context(model: &WhisperModel) -> Result<WhisperContext, String> {
    let model_filename = match model {
        WhisperModel::Tiny => "ggml-tiny.bin",
        WhisperModel::Base => "ggml-base.bin",
        WhisperModel::Small => "ggml-small.bin",
    };

    let search_paths: Vec<&'static str> = vec![
        "./models",
        "/usr/share/esoccer/models",
        "/usr/local/share/esoccer/models",
    ];

    for base in &search_paths {
        let model_path = std::path::Path::new(base).join(model_filename);
        if model_path.exists() {
            tracing::info!("Loading Whisper model from: {}", model_path.display());
            return WhisperContext::new_with_params(
                &model_path,
                WhisperContextParameters::default(),
            ).map_err(|e| format!("Failed to load Whisper model: {}", e));
        }
    }

    Err(format!(
        "Whisper model '{}' not found. Searched: {:?}",
        model_filename, search_paths
    ))
}

/// Get or initialize the Whisper context (lazy, once).
fn get_whisper_ctx(model: &WhisperModel) -> Result<&'static WhisperContext, String> {
    WHISPER_CTX
        .get_or_init(|| {
            tracing::info!("Initializing Whisper (lazy load)...");
            load_whisper_context(model)
        })
        .as_ref()
        .map_err(|e| e.clone())
}

/// Transcribe audio samples using Whisper (single-shot inference).
fn transcribe(samples: &[f32], model: &WhisperModel) -> Result<String, String> {
    let ctx = get_whisper_ctx(model)?;

    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some("pt"));
    params.set_translate(false);
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(true);
    params.set_suppress_nst(true);

    // Create state for inference
    let mut state = ctx
        .create_state()
        .map_err(|e| format!("Failed to create Whisper state: {}", e))?;

    // Run inference
    state
        .full(params, samples)
        .map_err(|e| format!("Whisper inference failed: {}", e))?;

    // Extract text from all segments
    let num_segments = state.full_n_segments();
    let mut text = String::new();
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            if let Ok(s) = segment.to_str() {
                text.push_str(s);
            }
        }
    }

    Ok(text.trim().to_string())
}

#[derive(Debug)]
pub enum VoiceError {
    Capture(String),
    Transcription(String),
    NotListening,
    ModelNotLoaded(String),
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::Capture(s) => write!(f, "Capture error: {}", s),
            VoiceError::Transcription(s) => write!(f, "Transcription error: {}", s),
            VoiceError::NotListening => write!(f, "Not currently listening"),
            VoiceError::ModelNotLoaded(s) => write!(f, "Model error: {}", s),
        }
    }
}

/// Gerencia o pipeline de voz (PTT)
pub struct VoiceCoordinator {
    is_listening: bool,
    capture: Option<CaptureStream>,
    whisper_model: WhisperModel,
}

impl VoiceCoordinator {
    pub fn new() -> Self {
        Self {
            is_listening: false,
            capture: None,
            whisper_model: WhisperModel::Base,
        }
    }

    pub fn with_model(mut self, model: WhisperModel) -> Self {
        self.whisper_model = model;
        self
    }

    /// Início PTT: começa a capturar do microfone
    pub fn start_listening(&mut self, device_name: Option<String>) -> Result<(), VoiceError> {
        if self.is_listening {
            return Err(VoiceError::Capture("Already listening".into()));
        }

        let config = CaptureConfig {
            device_name,
            ..CaptureConfig::default()
        };

        let stream =
            CaptureStream::start(config).map_err(|e| VoiceError::Capture(e.to_string()))?;

        self.capture = Some(stream);
        self.is_listening = true;

        tracing::info!("Voice capture started");
        Ok(())
    }

    /// Fim PTT: para captura, transcreve com Whisper, emite resultado.
    /// LOW FIX: Now returns transcript from Whisper.
    pub fn stop_listening(&mut self, app: &AppHandle) -> Result<Option<String>, VoiceError> {
        if !self.is_listening {
            return Err(VoiceError::NotListening);
        }

        let capture = self.capture.take().ok_or(VoiceError::NotListening)?;
        self.is_listening = false;

        let audio_buffer = capture.stop().map_err(|e| VoiceError::Capture(e.to_string()))?;

        if audio_buffer.samples.is_empty() {
            let _ = app.emit(
                "voice-status",
                serde_json::json!({
                    "status": "silence",
                    "transcript": null,
                    "error": null,
                }),
            );
            return Ok(None);
        }

        tracing::info!(
            "Voice capture stopped, {} samples, running Whisper...",
            audio_buffer.samples.len()
        );

        // CRITICAL-1 FIX: Transcribe with Whisper
        let transcript = match transcribe(&audio_buffer.samples, &self.whisper_model) {
            Ok(text) => {
                if text.is_empty() {
                    tracing::info!("Whisper returned empty transcript");
                    let _ = app.emit(
                        "voice-status",
                        serde_json::json!({
                            "status": "silence",
                            "transcript": null,
                            "error": null,
                        }),
                    );
                    return Ok(None);
                }
                tracing::info!("Whisper transcript: {}", text);
                text
            }
            Err(e) => {
                tracing::warn!("Whisper transcription failed: {}", e);
                let _ = app.emit(
                    "voice-status",
                    serde_json::json!({
                        "status": "error",
                        "transcript": null,
                        "error": e,
                    }),
                );
                return Err(VoiceError::Transcription(e));
            }
        };

        // Emit voice_text event (frontend compatibility)
        let _ = app.emit(
            "voice-text",
            serde_json::json!({ "text": transcript }),
        );

        let _ = app.emit(
            "voice-status",
            serde_json::json!({
                "status": "done",
                "transcript": transcript,
                "error": null,
            }),
        );

        Ok(Some(transcript))
    }

    /// Cancel listening without transcribing
    pub fn cancel_listening(&mut self) -> Result<(), VoiceError> {
        if !self.is_listening {
            return Err(VoiceError::NotListening);
        }

        if let Some(capture) = self.capture.take() {
            let _ = capture.stop();
        }
        self.is_listening = false;

        tracing::info!("Voice capture cancelled");
        Ok(())
    }

    /// Verifica se está ouvindo
    pub fn is_listening(&self) -> bool {
        self.is_listening
    }
}

impl Default for VoiceCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_coordinator_new_not_listening() {
        let vc = VoiceCoordinator::new();
        assert!(!vc.is_listening());
    }

    #[test]
    fn cancel_when_not_listening() {
        let mut vc = VoiceCoordinator::new();
        let result = vc.cancel_listening();
        assert!(result.is_err());
    }
}

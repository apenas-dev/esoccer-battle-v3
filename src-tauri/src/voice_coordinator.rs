//! Unified voice pipeline coordinator.
//!
//! Provides a single entry point for both continuous and push-to-talk
//! voice recognition. Wraps [`capture`], [`buffer`], [`transcriber`],
//! [`on_demand_transcriber`] and [`parser`] into a coherent API.
//!
//! **SRP:** This module only coordinates the voice pipeline. It does NOT
//! depend on `match_service` or emit game events. The caller is
//! responsible for processing [`TranscriptionResult`].

use std::sync::mpsc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};

use crate::capture;
use crate::on_demand_transcriber;
use crate::parser;
use crate::transcriber;

// ── Public types ─────────────────────────────────────────────────────────

/// Voice recognition mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoiceMode {
    Continuous,
    PushToTalk,
}

/// Holds the running continuous pipeline and its audio stream.
/// Both must be kept alive together — dropping the stream stops capture.
struct ContinuousHandle {
    _stream: capture::AudioStream,
    _pipeline: transcriber::VoicePipeline,
}

/// Result of a voice transcription: a parsed game command or unknown text.
#[derive(Debug, Clone)]
pub enum TranscriptionResult {
    Command(parser::GameCommand),
    Unknown(String),
}

/// Unified handle for the voice pipeline.
///
/// Manages either a continuous transcription loop or a push-to-talk
/// recording session. Only one mode can be active at a time.
pub struct VoiceCoordinatorHandle {
    mode: VoiceMode,
    continuous: Option<ContinuousHandle>,
    ptt_stream: Option<capture::AudioStream>,
}

impl Default for VoiceCoordinatorHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceCoordinatorHandle {
    /// Create a new inactive voice coordinator.
    pub fn new() -> Self {
        Self {
            mode: VoiceMode::Continuous,
            continuous: None,
            ptt_stream: None,
        }
    }

    /// Start continuous voice recognition.
    ///
    /// Pipeline: `capture → buffer → VoicePipeline → parser → TranscriptionResult`
    ///
    /// The `VoicePipeline` emits `"voice_text"` Tauri events with recognised
    /// text. The callback chain parses each transcription and emits
    /// `"voice_command"` events with the [`TranscriptionResult`].
    pub fn start_continuous(
        &mut self,
        app: AppHandle,
        device: Option<String>,
        model: transcriber::Model,
    ) -> Result<(), String> {
        // Stop any existing pipeline
        self.stop_continuous();

        let stream = capture::start_capture(device)?;
        let buffer = stream.buffer.clone();
        let pipeline = transcriber::VoicePipeline::start(app.clone(), buffer, model)?;

        // Spawn a listener that reads voice_text events and parses commands.
        // This bridges the VoicePipeline fire-and-forget emission with the parser.
        let app_clone = app.clone();
        let (tx, rx) = mpsc::channel::<()>();
        app.listen("voice_text", move |_event| {
            let _ = tx.send(());
        });

        // Spawn a thread that polls for transcribed text and parses commands.
        // Since VoicePipeline emits "voice_text" with { "text": "..." },
        // we use a dedicated transcription channel for the parsed results.
        let (result_tx, result_rx) = mpsc::channel::<TranscriptionResult>();
        let app_for_parser = app.clone();

        std::thread::Builder::new()
            .name("voice-coordinator-parser".into())
            .spawn(move || {
                // This listener receives voice_text events and parses them.
                // The VoicePipeline emits { "text": "..." } as payload.
                let (text_tx, text_rx) = mpsc::channel::<String>();

                let text_tx_clone = text_tx.clone();
                app_for_parser.listen("voice_text", move |event| {
                    if let Ok(payload) = serde_json::from_str::<serde_json::Value>(event.payload()) {
                        if let Some(text) = payload.get("text").and_then(|t| t.as_str()) {
                            let _ = text_tx_clone.send(text.to_string());
                        }
                    }
                });

                drop(tx); // Drop the initial dummy channel

                while let Ok(text) = text_rx.recv() {
                    let trimmed = text.trim().to_string();
                    if trimmed.is_empty() {
                        continue;
                    }

                    let result = match parser::parse_command(&trimmed) {
                        Some(cmd) => TranscriptionResult::Command(cmd),
                        None => TranscriptionResult::Unknown(trimmed.clone()),
                    };

                    tracing::info!(
                        "[voice-coordinator] Parsed: {:?} (from: \"{}\")",
                        result,
                        trimmed
                    );

                    // Emit parsed result for callers to process
                    let _ = result_tx.send(result.clone());
                    let _ = app_for_parser.emit("voice_command", &result);
                }
            })
            .map_err(|e| format!("Failed to spawn parser thread: {e}"))?;

        // Drop the result_rx — callers listen to "voice_command" events.
        // We keep the channel alive via the thread closure.
        drop(result_rx);

        self.continuous = Some(ContinuousHandle {
            _stream: stream,
            _pipeline: pipeline,
        });
        self.mode = VoiceMode::Continuous;

        tracing::info!("[voice-coordinator] ✅ Continuous pipeline started");
        Ok(())
    }

    /// Stop continuous voice recognition.
    ///
    /// Drops the pipeline and stream, which stops capture and transcription.
    pub fn stop_continuous(&mut self) {
        if self.continuous.take().is_some() {
            tracing::info!("[voice-coordinator] Continuous pipeline stopped");
        }
    }

    /// Start push-to-talk recording.
    ///
    /// Opens the microphone and begins buffering audio. Call [`stop_ptt`]
    /// to transcribe the recorded audio.
    pub fn start_ptt(&mut self, device: Option<String>) -> Result<(), String> {
        // Stop any existing PTT recording
        self.ptt_stream.take();

        let stream = capture::start_capture(device)?;
        self.ptt_stream = Some(stream);
        self.mode = VoiceMode::PushToTalk;

        tracing::info!("[voice-coordinator] ✅ PTT recording started");
        Ok(())
    }

    /// Stop push-to-talk recording, transcribe the audio, and return the result.
    ///
    /// Pipeline: `drain buffer → transcribe_once → parser → TranscriptionResult`
    pub fn stop_ptt(
        &mut self,
        model: transcriber::Model,
    ) -> Result<TranscriptionResult, String> {
        let stream = self
            .ptt_stream
            .take()
            .ok_or_else(|| "No PTT recording in progress".to_string())?;

        tracing::info!("[voice-coordinator] Stopping PTT — draining buffer");

        // Drain accumulated audio from the buffer.
        // Wait briefly for any in-flight audio to arrive.
        std::thread::sleep(Duration::from_millis(200));

        let audio = stream.drain_buffer();
        drop(stream); // Stop capture immediately

        let sample_count = audio.len();
        if sample_count < 1000 {
            return Err(format!(
                "Not enough audio captured ({} samples). Speak closer to the mic.",
                sample_count
            ));
        }

        tracing::info!(
            "[voice-coordinator] Transcribing {} samples via on-demand pipeline",
            sample_count
        );

        let language = model.default_language();
        let text =
            on_demand_transcriber::transcribe_once(&audio, &model, language)?;

        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err("Transcription returned empty text (silence?)".to_string());
        }

        tracing::info!("[voice-coordinator] PTT transcription: \"{}\"", trimmed);

        let result = match parser::parse_command(trimmed) {
            Some(cmd) => {
                tracing::info!("[voice-coordinator] ✅ Recognised command: {:?}", cmd);
                TranscriptionResult::Command(cmd)
            }
            None => {
                tracing::info!("[voice-coordinator] Unknown command: \"{}\"", trimmed);
                TranscriptionResult::Unknown(trimmed.to_string())
            }
        };

        Ok(result)
    }

    /// Check if any voice pipeline is currently active.
    pub fn is_active(&self) -> bool {
        self.continuous.is_some() || self.ptt_stream.is_some()
    }

    /// Get the current voice mode.
    pub fn mode(&self) -> VoiceMode {
        self.mode
    }
}

impl Drop for VoiceCoordinatorHandle {
    fn drop(&mut self) {
        self.stop_continuous();
        self.ptt_stream.take(); // Drop PTT stream if active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_handle_is_inactive() {
        let handle = VoiceCoordinatorHandle::new();
        assert!(!handle.is_active());
        assert_eq!(handle.mode(), VoiceMode::Continuous);
    }

    #[test]
    fn default_handle_is_inactive() {
        let handle = VoiceCoordinatorHandle::default();
        assert!(!handle.is_active());
    }

    #[test]
    fn transcription_result_command_display() {
        let result = TranscriptionResult::Command(parser::GameCommand::StartMatch);
        assert!(matches!(result, TranscriptionResult::Command(_)));

        let result = TranscriptionResult::Unknown("hello".to_string());
        assert!(matches!(result, TranscriptionResult::Unknown(_)));
    }
}

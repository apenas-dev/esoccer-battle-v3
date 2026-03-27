use crate::capture::{CaptureConfig, CaptureStream};
use tauri::{AppHandle, Emitter};

/// Canal de saída do pipeline de voz
#[derive(Debug, Clone)]
pub enum VoiceEvent {
    TranscriptReady(String),
    Listening,
    Silence,
    Error(String),
}

#[derive(Debug)]
pub enum VoiceError {
    Capture(String),
    Transcription(String),
    NotListening,
}

impl std::fmt::Display for VoiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VoiceError::Capture(s) => write!(f, "Capture error: {}", s),
            VoiceError::Transcription(s) => write!(f, "Transcription error: {}", s),
            VoiceError::NotListening => write!(f, "Not currently listening"),
        }
    }
}

/// Gerencia o pipeline de voz (PTT)
pub struct VoiceCoordinator {
    is_listening: bool,
    capture: Option<CaptureStream>,
}

impl VoiceCoordinator {
    pub fn new() -> Self {
        Self {
            is_listening: false,
            capture: None,
        }
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

    /// Fim PTT: para captura, emite voice-status event via Tauri
    /// A transcrição acontece no frontend (WebSpeech API) ou via whisper direto
    pub fn stop_listening(&mut self, app: &AppHandle) -> Result<(), VoiceError> {
        if !self.is_listening {
            return Err(VoiceError::NotListening);
        }

        let capture = self.capture.take().ok_or(VoiceError::NotListening)?;
        self.is_listening = false;

        let audio_buffer = capture.stop().map_err(|e| VoiceError::Capture(e.to_string()))?;

        if audio_buffer.samples.is_empty() {
            // No audio captured — emit silence
            let _ = app.emit(
                "voice-status",
                VoiceStatusPayload {
                    status: "silence".into(),
                    transcript: None,
                    error: None,
                },
            );
            return Ok(());
        }

        // Emit processing status — frontend will handle transcription via WebSpeech/Whisper
        let _ = app.emit(
            "voice-status",
            VoiceStatusPayload {
                status: "processing".into(),
                transcript: None,
                error: None,
            },
        );

        tracing::info!(
            "Voice capture stopped, {} samples captured",
            audio_buffer.samples.len()
        );

        // Note: The actual transcription is handled by the frontend STT provider.
        // The backend just captures and notifies. The frontend then calls
        // execute_command with the transcript.
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

#[derive(Clone, serde::Serialize)]
struct VoiceStatusPayload {
    status: String,
    transcript: Option<String>,
    error: Option<String>,
}

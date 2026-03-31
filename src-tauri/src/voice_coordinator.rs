use std::sync::mpsc;
use tauri::AppHandle;
use crate::capture::{CaptureStream, CaptureConfig};

pub enum VoiceEvent {
    TranscriptReady(String),
    Listening,
    Error(String),
}

pub struct VoiceCoordinator {
    is_listening: bool,
    capture: Option<CaptureStream>,
    event_tx: Option<mpsc::Sender<VoiceEvent>>,
}

// SAFETY: Access is serialized through Mutex in AppState
unsafe impl Send for VoiceCoordinator {}

impl VoiceCoordinator {
    pub fn new(event_tx: mpsc::Sender<VoiceEvent>) -> Self {
        Self { is_listening: false, capture: None, event_tx: Some(event_tx) }
    }

    pub fn start_listening(&mut self, _app: &AppHandle, config: Option<CaptureConfig>) -> Result<(), VoiceError> {
        let capture_config = config.unwrap_or_default();
        let stream = CaptureStream::start(capture_config).map_err(|e| VoiceError::Capture(e.to_string()))?;
        self.capture = Some(stream);
        self.is_listening = true;
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(VoiceEvent::Listening);
        }
        Ok(())
    }

    pub fn stop_listening(&mut self, _app: &AppHandle) -> Result<crate::capture::AudioBuffer, VoiceError> {
        if !self.is_listening {
            return Err(VoiceError::NotListening);
        }
        self.is_listening = false;
        let stream = self.capture.take().ok_or(VoiceError::Capture("No capture stream".to_string()))?;
        let buffer = stream.stop().map_err(|e| VoiceError::Capture(e.to_string()))?;
        // Transcription will be handled by frontend (OpenAI Whisper API or WebSpeech)
        Ok(buffer)
    }

    pub fn is_listening(&self) -> bool {
        self.is_listening
    }
}

#[derive(Debug)]
pub enum VoiceError {
    Capture(String),
    Transcription(String),
    NotListening,
}

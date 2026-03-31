use std::sync::mpsc;
use tauri::AppHandle;

pub enum VoiceEvent {
    TranscriptReady(String),
    Listening,
    Error(String),
}

pub struct VoiceCoordinator {
    is_listening: bool,
    event_tx: Option<mpsc::Sender<VoiceEvent>>,
}

impl VoiceCoordinator {
    pub fn new(event_tx: mpsc::Sender<VoiceEvent>) -> Self {
        Self { is_listening: false, event_tx: Some(event_tx) }
    }

    pub async fn start_listening(&mut self, _app: &AppHandle) -> Result<(), VoiceError> {
        // TODO: integrate with capture.rs for mic capture
        self.is_listening = true;
        if let Some(tx) = &self.event_tx {
            let _ = tx.send(VoiceEvent::Listening);
        }
        Ok(())
    }

    pub async fn stop_listening(&mut self, _app: &AppHandle) -> Result<(), VoiceError> {
        if !self.is_listening {
            return Err(VoiceError::NotListening);
        }
        self.is_listening = false;
        // TODO: stop capture, get audio buffer, transcribe
        Ok(())
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

use crate::audio;
use crate::history;
use crate::match_service::Action;
use tauri::Emitter;

#[derive(Debug)]
pub enum DispatchError {
    Audio(String),
    History(String),
    Emit(String),
}

impl std::fmt::Display for DispatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DispatchError::Audio(s) => write!(f, "Audio error: {}", s),
            DispatchError::History(s) => write!(f, "History error: {}", s),
            DispatchError::Emit(s) => write!(f, "Emit error: {}", s),
        }
    }
}

/// Executa todas as ações em sequência
pub fn dispatch(actions: Vec<Action>, app_handle: &tauri::AppHandle) -> Result<(), DispatchError> {
    for action in actions {
        execute_action(action, app_handle)?;
    }
    Ok(())
}

/// Executa uma única ação
fn execute_action(action: Action, app_handle: &tauri::AppHandle) -> Result<(), DispatchError> {
    match action {
        Action::PlaySound(sound) => {
            // FIX 8: Use SoundFile::from_name instead of inline mapping
            let name = match sound {
                crate::match_service::SoundName::Goal => "goal",
                crate::match_service::SoundName::Whistle => "whistle",
                crate::match_service::SoundName::SixMeters => "six_meters",
                crate::match_service::SoundName::Challenge => "challenge",
            };
            let sound_file = audio::SoundFile::from_name(name)
                .ok_or_else(|| DispatchError::Audio(format!("Unknown sound: {}", name)))?;
            audio::play(sound_file).map_err(|e| DispatchError::Audio(e.to_string()))
        }

        // FIX 1: EmitPhaseChanged now carries sub_phase
        Action::EmitPhaseChanged { phase, sub_phase } => {
            let payload = PhasePayload {
                phase: phase.to_string(),
                sub_phase: sub_phase.to_string(),
            };
            emit_event(app_handle, "phase-changed", &payload)
        }

        Action::EmitScoreChanged { score_a, score_b } => {
            let payload = ScorePayload { score_a, score_b };
            emit_event(app_handle, "score-changed", &payload)
        }

        Action::EmitTimeUpdated {
            elapsed_secs,
            display,
        } => {
            let payload = TimePayload {
                elapsed_secs,
                display,
            };
            emit_event(app_handle, "time-updated", &payload)
        }

        Action::EmitMatchFinished { score_a, score_b } => {
            let payload = ScorePayload { score_a, score_b };
            emit_event(app_handle, "match-finished", &payload)
        }

        // FIX 6: Spawn history::save in background thread to avoid blocking main
        Action::SaveMatch(snapshot) => {
            let app = app_handle.clone();
            std::thread::spawn(move || {
                if let Err(e) = history::save(snapshot) {
                    tracing::error!("Failed to save match history: {}", e);
                }
                let _ = app.emit("history-updated", ());
            });
            Ok(())
        }

        Action::StartTimer => {
            // Timer is managed by main.rs via TimerManager, not here
            Ok(())
        }

        Action::StopTimer => {
            // Timer is managed by main.rs via TimerManager, not here
            Ok(())
        }

        Action::NoOp => Ok(()),
    }
}

fn emit_event<T: serde::Serialize>(
    app_handle: &tauri::AppHandle,
    name: &str,
    payload: &T,
) -> Result<(), DispatchError> {
    app_handle
        .emit(name, payload)
        .map_err(|e| DispatchError::Emit(e.to_string()))
}

#[derive(serde::Serialize)]
struct PhasePayload {
    phase: String,
    sub_phase: String,
}

#[derive(serde::Serialize)]
struct ScorePayload {
    score_a: u32,
    score_b: u32,
}

#[derive(serde::Serialize)]
struct TimePayload {
    elapsed_secs: u64,
    display: String,
}

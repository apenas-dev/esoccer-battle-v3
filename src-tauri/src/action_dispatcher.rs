use crate::audio;
use crate::game::GamePhase;
use crate::history;
use crate::match_service::{Action, SoundName};
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
            let sound_file = match sound {
                SoundName::Goal => audio::SoundFile::Goal,
                SoundName::Whistle => audio::SoundFile::Whistle,
                SoundName::SixMeters => audio::SoundFile::SixMeters,
                SoundName::Challenge => audio::SoundFile::Challenge,
            };
            audio::play(sound_file).map_err(|e| DispatchError::Audio(e.to_string()))
        }

        Action::EmitPhaseChanged(phase) => {
            let payload = PhasePayload {
                phase: phase.to_string(),
                sub_phase: if phase == GamePhase::Playing {
                    "normal".to_string()
                } else {
                    "normal".to_string()
                },
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

        Action::SaveMatch(snapshot) => {
            history::save(snapshot).map_err(|e| DispatchError::History(e.to_string()))
        }

        Action::StartTimer => emit_event(app_handle, "timer-control", &"start"),

        Action::StopTimer => emit_event(app_handle, "timer-control", &"stop"),

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

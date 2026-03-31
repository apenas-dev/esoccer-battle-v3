use tauri::{AppHandle, Emitter};
use crate::match_service::{Action, SoundName, MatchSnapshot};
use crate::audio;
use crate::history;

pub async fn dispatch(
    actions: Vec<Action>,
    app_handle: &AppHandle,
) -> Result<(), DispatchError> {
    for action in actions {
        match action {
            Action::PlaySound(sound) => {
                let sound_file = match sound {
                    SoundName::Goal => audio::SoundFile::Goal,
                    SoundName::Whistle => audio::SoundFile::Whistle,
                };
                audio::play(sound_file).await.map_err(|e| DispatchError::Audio(e.to_string()))?;
            }
            Action::EmitPhaseChanged(p) => {
                app_handle.emit("phase-changed", p).map_err(|e| DispatchError::Emit(e.to_string()))?;
            }
            Action::EmitScoreChanged { score_a, score_b } => {
                app_handle.emit("score-changed", serde_json::json!({ "score_a": score_a, "score_b": score_b }))
                    .map_err(|e| DispatchError::Emit(e.to_string()))?;
            }
            Action::EmitMatchFinished { score_a, score_b } => {
                app_handle.emit("match-finished", serde_json::json!({ "score_a": score_a, "score_b": score_b }))
                    .map_err(|e| DispatchError::Emit(e.to_string()))?;
            }
            Action::SaveMatch(snapshot) => {
                history::save(snapshot).await.map_err(|e| DispatchError::History(e.to_string()))?;
            }
            Action::StartTimer => {
                app_handle.emit("timer-control", "start").map_err(|e| DispatchError::Emit(e.to_string()))?;
            }
            Action::StopTimer => {
                app_handle.emit("timer-control", "stop").map_err(|e| DispatchError::Emit(e.to_string()))?;
            }
            Action::NoOp => {}
        }
    }
    Ok(())
}

#[derive(Debug)]
pub enum DispatchError {
    Audio(String),
    History(String),
    Emit(String),
}

use crate::game::{MatchState, GamePhase};
use crate::command::GameCommand;

#[derive(Debug, Clone)]
pub enum Action {
    PlaySound(SoundName),
    EmitPhaseChanged(GamePhase),
    EmitScoreChanged { score_a: u32, score_b: u32 },
    EmitMatchFinished { score_a: u32, score_b: u32 },
    SaveMatch(MatchSnapshot),
    StartTimer,
    StopTimer,
    NoOp,
}

#[derive(Debug, Clone)]
pub enum SoundName {
    Goal,
    Whistle,
}

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub new_state: MatchState,
    pub actions: Vec<Action>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchSnapshot {
    pub match_id: String,
    pub team_a_name: String,
    pub team_b_name: String,
    pub score_a: u32,
    pub score_b: u32,
    pub duration_secs: u64,
    pub finished_at: String,
}

/// ÚNICA função pública. Pura. Determinística.
/// `now` — timestamp UNIX em segundos, injetado pelo caller.
pub fn process(state: &MatchState, command: GameCommand, now: u64) -> MatchResult {
    let noop = || MatchResult { new_state: state.clone(), actions: vec![Action::NoOp] };

    match (&state.phase, command) {
        // Start: Idle → Playing
        (GamePhase::Idle, GameCommand::Start) => {
            let new_state = state.clone()
                .with_phase(GamePhase::Playing)
                .with_started_at(now);
            MatchResult {
                new_state,
                actions: vec![Action::StartTimer, Action::PlaySound(SoundName::Whistle), Action::EmitPhaseChanged(GamePhase::Playing)],
            }
        }

        // GoalA: Playing → score_a += 1  (#15: sem clone desnecessário)
        (GamePhase::Playing, GameCommand::GoalA) => {
            let new_state = state.clone().with_score_a(state.score_a + 1);
            MatchResult {
                actions: vec![Action::PlaySound(SoundName::Goal), Action::EmitScoreChanged { score_a: new_state.score_a, score_b: new_state.score_b }],
                new_state,
            }
        }

        // GoalB: Playing → score_b += 1  (#15: sem clone desnecessário)
        (GamePhase::Playing, GameCommand::GoalB) => {
            let new_state = state.clone().with_score_b(state.score_b + 1);
            MatchResult {
                actions: vec![Action::PlaySound(SoundName::Goal), Action::EmitScoreChanged { score_a: new_state.score_a, score_b: new_state.score_b }],
                new_state,
            }
        }

        // Pause: Playing → Paused
        (GamePhase::Playing, GameCommand::Pause) => {
            let new_state = state.clone()
                .with_phase(GamePhase::Paused);
            MatchResult {
                new_state,
                actions: vec![Action::StopTimer, Action::EmitPhaseChanged(GamePhase::Paused)],
            }
        }

        // Resume: Paused → Playing
        (GamePhase::Paused, GameCommand::Resume) => {
            let new_state = state.clone()
                .with_phase(GamePhase::Playing);
            MatchResult {
                new_state,
                actions: vec![Action::StartTimer, Action::EmitPhaseChanged(GamePhase::Playing)],
            }
        }

        // End: Playing/Paused → Finished
        (GamePhase::Playing | GamePhase::Paused, GameCommand::End) => {
            // Use `now` for finished_at instead of chrono::Utc::now()
            let finished_at = chrono::DateTime::from_timestamp(now as i64, 0)
                .unwrap_or_else(chrono::Utc::now)
                .to_rfc3339();
            let snapshot = MatchSnapshot {
                match_id: state.match_id.clone(),
                team_a_name: state.config.team_a_name.clone(),
                team_b_name: state.config.team_b_name.clone(),
                score_a: state.score_a,
                score_b: state.score_b,
                duration_secs: state.elapsed_secs,
                finished_at,
            };
            let new_state = state.clone()
                .with_phase(GamePhase::Finished);
            MatchResult {
                actions: vec![
                    Action::StopTimer,
                    Action::SaveMatch(snapshot),
                    Action::PlaySound(SoundName::Whistle),
                    Action::EmitMatchFinished { score_a: new_state.score_a, score_b: new_state.score_b },
                ],
                new_state,
            }
        }

        // Reset: Finished → Idle
        (GamePhase::Finished, GameCommand::Reset) => {
            let new_state = MatchState::new(state.config.clone());
            MatchResult {
                new_state,
                actions: vec![Action::EmitPhaseChanged(GamePhase::Idle)],
            }
        }

        // Invalid transitions
        _ => noop(),
    }
}

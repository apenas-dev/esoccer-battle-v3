use crate::command::GameCommand;
use crate::game::{GamePhase, MatchState, PlayingSubPhase};

/// Ações a serem executadas pelo dispatcher
#[derive(Debug, Clone)]
pub enum Action {
    PlaySound(SoundName),
    EmitPhaseChanged { phase: GamePhase, sub_phase: PlayingSubPhase },
    EmitScoreChanged { score_a: u32, score_b: u32 },
    EmitTimeUpdated { elapsed_secs: u64, display: String },
    EmitMatchFinished { score_a: u32, score_b: u32 },
    SaveMatch(MatchSnapshot),
    StartTimer,
    StopTimer,
    NoOp,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum SoundName {
    Goal,
    Whistle,
    SixMeters,
    Challenge,
}

/// Resultado do processamento de um comando
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub new_state: MatchState,
    pub actions: Vec<Action>,
}

/// Snapshot para salvar no histórico
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchSnapshot {
    pub match_id: String,
    pub team_a_name: String,
    pub team_b_name: String,
    pub score_a: u32,
    pub score_b: u32,
    pub duration_secs: u64,
    pub finished_at: String, // ISO 8601
}

/// Processa um comando puro. NÃO executa efeitos colaterais.
pub fn process(state: &MatchState, command: GameCommand) -> MatchResult {
    match command {
        GameCommand::Start => process_start(state),
        GameCommand::GoalA => process_goal_a(state),
        GameCommand::GoalB => process_goal_b(state),
        GameCommand::Pause => process_pause(state),
        GameCommand::Resume => process_resume(state),
        GameCommand::Doubt => process_doubt(state),
        GameCommand::Resolve => process_resolve(state),
        GameCommand::VoltaSeis => process_volta_seis(state),
        GameCommand::End => process_end(state),
        GameCommand::Reset => process_reset(state),
    }
}

fn noop(state: &MatchState) -> MatchResult {
    MatchResult {
        new_state: state.clone(),
        actions: vec![Action::NoOp],
    }
}

fn process_start(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Idle {
        return noop(state);
    }

    let new_state = state
        .clone()
        .with_phase(GamePhase::Playing)
        .with_sub_phase(PlayingSubPhase::Normal)
        .with_started_at(Some(chrono::Utc::now().timestamp_millis() as u64))
        .with_elapsed(0);

    MatchResult {
        new_state,
        actions: vec![
            Action::PlaySound(SoundName::Whistle),
            Action::EmitPhaseChanged { phase: GamePhase::Playing, sub_phase: PlayingSubPhase::Normal },
            Action::StartTimer,
        ],
    }
}

fn process_goal_a(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Normal {
        return noop(state);
    }

    let new_score = state.score_a + 1;
    let new_state = state.clone().with_score_a(new_score);

    MatchResult {
        new_state,
        actions: vec![
            Action::PlaySound(SoundName::Goal),
            Action::EmitScoreChanged {
                score_a: new_score,
                score_b: state.score_b,
            },
        ],
    }
}

fn process_goal_b(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Normal {
        return noop(state);
    }

    let new_score = state.score_b + 1;
    let new_state = state.clone().with_score_b(new_score);

    MatchResult {
        new_state,
        actions: vec![
            Action::PlaySound(SoundName::Goal),
            Action::EmitScoreChanged {
                score_a: state.score_a,
                score_b: new_score,
            },
        ],
    }
}

fn process_pause(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing {
        return noop(state);
    }

    let sub_phase = state.sub_phase.clone();
    let new_state = state
        .clone()
        .with_phase(GamePhase::Paused)
        .with_paused_elapsed(state.elapsed_secs);

    MatchResult {
        new_state,
        actions: vec![
            Action::StopTimer,
            Action::EmitPhaseChanged { phase: GamePhase::Paused, sub_phase },
        ],
    }
}

fn process_resume(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Paused {
        return noop(state);
    }

    // Explicitly restore elapsed from paused value — do not rely on implicit equality
    let sub_phase = state.sub_phase.clone();
    let new_state = state
        .clone()
        .with_phase(GamePhase::Playing)
        .with_elapsed(state.paused_elapsed_secs)
        .with_started_at(Some(chrono::Utc::now().timestamp_millis() as u64));

    MatchResult {
        new_state,
        actions: vec![
            Action::StartTimer,
            Action::EmitPhaseChanged { phase: GamePhase::Playing, sub_phase },
        ],
    }
}

fn process_doubt(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Normal {
        return noop(state);
    }

    let new_state = state.clone().with_sub_phase(PlayingSubPhase::Challenge);

    MatchResult {
        new_state,
        actions: vec![
            Action::PlaySound(SoundName::Challenge),
            Action::EmitPhaseChanged { phase: GamePhase::Playing, sub_phase: PlayingSubPhase::Challenge },
        ],
    }
}

fn process_resolve(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Challenge {
        return noop(state);
    }

    let new_state = state.clone().with_sub_phase(PlayingSubPhase::Normal);

    MatchResult {
        new_state,
        actions: vec![Action::EmitPhaseChanged { phase: GamePhase::Playing, sub_phase: PlayingSubPhase::Normal }],
    }
}

fn process_volta_seis(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Challenge {
        return noop(state);
    }

    let new_state = state.clone().with_sub_phase(PlayingSubPhase::Normal);

    MatchResult {
        new_state,
        actions: vec![
            Action::PlaySound(SoundName::SixMeters),
            Action::EmitPhaseChanged { phase: GamePhase::Playing, sub_phase: PlayingSubPhase::Normal },
        ],
    }
}

fn process_end(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing && state.phase != GamePhase::Paused {
        return noop(state);
    }

    let snapshot = MatchSnapshot {
        match_id: state.match_id.clone(),
        team_a_name: state.config.team_a_name.clone(),
        team_b_name: state.config.team_b_name.clone(),
        score_a: state.score_a,
        score_b: state.score_b,
        duration_secs: state.elapsed_secs,
        finished_at: chrono::Utc::now().to_rfc3339(),
    };

    let new_state = state.clone().with_phase(GamePhase::Finished);

    MatchResult {
        new_state,
        actions: vec![
            Action::StopTimer,
            Action::PlaySound(SoundName::Whistle),
            Action::EmitMatchFinished {
                score_a: state.score_a,
                score_b: state.score_b,
            },
            Action::EmitPhaseChanged { phase: GamePhase::Finished, sub_phase: PlayingSubPhase::Normal },
            Action::SaveMatch(snapshot),
        ],
    }
}

fn process_reset(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Finished {
        return noop(state);
    }

    let new_state = MatchState::new(state.config.clone());

    MatchResult {
        new_state,
        actions: vec![Action::EmitPhaseChanged { phase: GamePhase::Idle, sub_phase: PlayingSubPhase::Normal }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::MatchConfig;

    fn idle_state() -> MatchState {
        MatchState::new(MatchConfig::default())
    }

    fn playing_state() -> MatchState {
        idle_state()
            .with_phase(GamePhase::Playing)
            .with_started_at(Some(1000))
    }

    #[test]
    fn start_from_idle() {
        let state = idle_state();
        let result = process(&state, GameCommand::Start);
        assert_eq!(result.new_state.phase, GamePhase::Playing);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Normal);
        assert!(result.new_state.started_at.is_some());
        assert!(result.actions.iter().any(|a| matches!(a, Action::StartTimer)));
    }

    #[test]
    fn start_from_playing_is_noop() {
        let state = playing_state();
        let result = process(&state, GameCommand::Start);
        assert_eq!(result.new_state.phase, GamePhase::Playing);
        assert!(result.actions.iter().any(|a| matches!(a, Action::NoOp)));
    }

    #[test]
    fn goal_a_from_playing() {
        let state = playing_state();
        let result = process(&state, GameCommand::GoalA);
        assert_eq!(result.new_state.score_a, 1);
        assert!(result.actions.iter().any(|a| matches!(a, Action::PlaySound(SoundName::Goal))));
    }

    #[test]
    fn goal_b_from_playing() {
        let state = playing_state();
        let result = process(&state, GameCommand::GoalB);
        assert_eq!(result.new_state.score_b, 1);
    }

    #[test]
    fn goal_during_challenge_is_noop() {
        let state = playing_state().with_sub_phase(PlayingSubPhase::Challenge);
        let result = process(&state, GameCommand::GoalA);
        assert_eq!(result.new_state.score_a, 0);
        assert!(result.actions.iter().any(|a| matches!(a, Action::NoOp)));
    }

    #[test]
    fn pause_from_playing() {
        let state = playing_state().with_elapsed(60);
        let result = process(&state, GameCommand::Pause);
        assert_eq!(result.new_state.phase, GamePhase::Paused);
        assert_eq!(result.new_state.paused_elapsed_secs, 60);
    }

    #[test]
    fn resume_from_paused() {
        let state = playing_state()
            .with_phase(GamePhase::Paused)
            .with_elapsed(60)
            .with_paused_elapsed(60);
        let result = process(&state, GameCommand::Resume);
        assert_eq!(result.new_state.phase, GamePhase::Playing);
    }

    #[test]
    fn doubt_from_normal() {
        let state = playing_state();
        let result = process(&state, GameCommand::Doubt);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Challenge);
    }

    #[test]
    fn resolve_from_challenge() {
        let state = playing_state().with_sub_phase(PlayingSubPhase::Challenge);
        let result = process(&state, GameCommand::Resolve);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Normal);
    }

    #[test]
    fn volta_seis_from_challenge() {
        let state = playing_state().with_sub_phase(PlayingSubPhase::Challenge);
        let result = process(&state, GameCommand::VoltaSeis);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Normal);
        assert!(result.actions.iter().any(|a| matches!(a, Action::PlaySound(SoundName::SixMeters))));
    }

    #[test]
    fn end_from_playing() {
        let state = playing_state().with_score_a(3).with_score_b(2);
        let result = process(&state, GameCommand::End);
        assert_eq!(result.new_state.phase, GamePhase::Finished);
        assert!(result.actions.iter().any(|a| matches!(a, Action::SaveMatch(_))));
    }

    #[test]
    fn end_from_paused() {
        let state = playing_state()
            .with_phase(GamePhase::Paused)
            .with_paused_elapsed(300);
        let result = process(&state, GameCommand::End);
        assert_eq!(result.new_state.phase, GamePhase::Finished);
    }

    #[test]
    fn reset_from_finished() {
        let state = playing_state().with_phase(GamePhase::Finished).with_score_a(5);
        let result = process(&state, GameCommand::Reset);
        assert_eq!(result.new_state.phase, GamePhase::Idle);
        assert_eq!(result.new_state.score_a, 0);
        // New match ID
        assert_ne!(result.new_state.match_id, state.match_id);
    }

    #[test]
    fn reset_from_playing_is_noop() {
        let state = playing_state();
        let result = process(&state, GameCommand::Reset);
        assert!(result.actions.iter().any(|a| matches!(a, Action::NoOp)));
    }

    #[test]
    fn doubt_from_challenge_is_noop() {
        let state = playing_state().with_sub_phase(PlayingSubPhase::Challenge);
        let result = process(&state, GameCommand::Doubt);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Challenge);
        assert!(result.actions.iter().any(|a| matches!(a, Action::NoOp)));
    }

    #[test]
    fn end_generates_correct_snapshot() {
        let state = playing_state()
            .with_score_a(2)
            .with_score_b(1)
            .with_elapsed(300);
        let result = process(&state, GameCommand::End);
        let snapshot_action = result.actions.iter().find(|a| matches!(a, Action::SaveMatch(_)));
        assert!(snapshot_action.is_some());
        if let Action::SaveMatch(snap) = snapshot_action.unwrap() {
            assert_eq!(snap.score_a, 2);
            assert_eq!(snap.score_b, 1);
            assert_eq!(snap.duration_secs, 300);
            assert_eq!(snap.match_id, state.match_id);
            assert_eq!(snap.team_a_name, state.config.team_a_name);
            assert_eq!(snap.team_b_name, state.config.team_b_name);
            assert!(!snap.finished_at.is_empty());
        }
    }

    #[test]
    fn resolve_from_normal_is_noop() {
        let state = playing_state();
        let result = process(&state, GameCommand::Resolve);
        assert!(result.actions.iter().any(|a| matches!(a, Action::NoOp)));
    }

    #[test]
    fn volta_seis_from_normal_is_noop() {
        let state = playing_state();
        let result = process(&state, GameCommand::VoltaSeis);
        assert!(result.actions.iter().any(|a| matches!(a, Action::NoOp)));
    }

    #[test]
    fn all_commands_in_idle_produce_noop_except_start() {
        let state = idle_state();
        let noop_cmds = [
            GameCommand::GoalA, GameCommand::GoalB, GameCommand::Pause,
            GameCommand::Resume, GameCommand::Doubt, GameCommand::Resolve,
            GameCommand::VoltaSeis, GameCommand::End, GameCommand::Reset,
        ];
        for cmd in &noop_cmds {
            let result = process(&state, cmd.clone());
            assert!(
                result.actions.iter().any(|a| matches!(a, Action::NoOp)),
                "Expected NoOp for {:?} in Idle",
                cmd
            );
        }
    }
}

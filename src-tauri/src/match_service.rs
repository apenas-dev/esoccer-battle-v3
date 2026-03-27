//! `match_service.rs` — Pure business logic for match state transitions.
//!
//! **Responsabilidade ÚNICA:** Receive state + command, return new state + actions.
//! Zero I/O, zero Tauri, zero threads, zero side-effects.
//!
//! SRP: State transitions + action generation only.
//! OCP: Adding a new command = one `GameCommand` variant + one `match` arm here.

use crate::audio::SoundName;
use crate::command::GameCommand;
use crate::game::{GamePhase, MatchState, PlayingSubPhase};
use serde::{Deserialize, Serialize};

// ── Public types ─────────────────────────────────────────────────────────

/// Side-effects to be executed by the dispatcher (not executed here).
#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    PlaySound(SoundName),
    EmitPhaseChanged(GamePhase),
    EmitScoreChanged { score_a: u32, score_b: u32 },
    EmitTimeUpdated { elapsed_secs: u64, display: String },
    EmitMatchFinished { score_a: u32, score_b: u32 },
    SaveMatch(MatchSnapshot),
    StartTimer,
    StopTimer,
    NoOp,
}

/// Result of processing a command.
#[derive(Debug, Clone)]
pub struct MatchResult {
    pub new_state: MatchState,
    pub actions: Vec<Action>,
}

/// Snapshot for persisting to history.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchSnapshot {
    pub match_id: String,
    pub team_a_name: String,
    pub team_b_name: String,
    pub score_a: u32,
    pub score_b: u32,
    pub duration_secs: u64,
    pub finished_at: String, // ISO 8601
}

// ── process() — the single public entry point ────────────────────────────

/// Processes a command against the current match state.
///
/// Pure function: deterministic, no side-effects, thread-safe.
/// Returns `[NoOp]` for commands issued in invalid phases.
pub fn process(state: &MatchState, command: GameCommand) -> MatchResult {
    let result = match command {
        GameCommand::Start => process_start(state),
        GameCommand::GoalA => process_goal_a(state),
        GameCommand::GoalB => process_goal_b(state),
        GameCommand::Pause => process_pause(state),
        GameCommand::Resume => process_resume(state),
        GameCommand::End => process_end(state),
        GameCommand::Doubt => process_doubt(state),
        GameCommand::Resolve => process_resolve(state),
        GameCommand::VoltaSeis => process_volta_seis(state),
        GameCommand::Reset => process_reset(state),
    };

    for (i, action) in result.actions.iter().enumerate() {
        eprintln!("[SERVICE] action[{}]: {:?}", i, action);
    }
    eprintln!(
        "[SERVICE] process({:?}) → phase={:?}, actions={}",
        command, result.new_state.phase, result.actions.len()
    );

    result
}

// ── Command handlers (each is a pure transition) ─────────────────────────

/// Start: Idle → Playing, Normal sub-phase.
fn process_start(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Idle {
        return noop(state);
    }

    let new_state = state
        .clone()
        .with_phase(GamePhase::Playing)
        .with_sub_phase(PlayingSubPhase::Normal)
        .with_started_at(Some(now_millis()))
        .with_elapsed(0)
        .with_score_a(0)
        .with_score_b(0);

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![
            Action::StartTimer,
            Action::PlaySound(SoundName::Whistle),
            Action::EmitPhaseChanged(GamePhase::Playing),
            Action::EmitTimeUpdated {
                elapsed_secs: 0,
                display: new_state.display_time_string(),
            },
        ],
    }
}

/// GoalA: Playing (Normal) → score_a += 1.
fn process_goal_a(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Normal {
        return noop(state);
    }

    let new_score = state.score_a + 1;
    let new_state = state.clone().with_score_a(new_score);

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![
            Action::PlaySound(SoundName::Goal),
            Action::EmitScoreChanged {
                score_a: new_score,
                score_b: new_state.score_b,
            },
        ],
    }
}

/// GoalB: Playing (Normal) → score_b += 1.
fn process_goal_b(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Normal {
        return noop(state);
    }

    let new_score = state.score_b + 1;
    let new_state = state.clone().with_score_b(new_score);

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![
            Action::PlaySound(SoundName::Goal),
            Action::EmitScoreChanged {
                score_a: new_state.score_a,
                score_b: new_score,
            },
        ],
    }
}

/// Pause: Playing → Paused.
fn process_pause(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing {
        return noop(state);
    }

    // Save elapsed so Resume can restore it
    let new_state = state
        .clone()
        .with_phase(GamePhase::Paused)
        .with_paused_elapsed(state.elapsed_secs);

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![
            Action::StopTimer,
            Action::EmitPhaseChanged(GamePhase::Paused),
        ],
    }
}

/// Resume: Paused → Playing, restore elapsed time.
fn process_resume(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Paused {
        return noop(state);
    }

    let new_state = state
        .clone()
        .with_phase(GamePhase::Playing)
        .with_elapsed(state.paused_elapsed_secs);

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![
            Action::StartTimer,
            Action::EmitPhaseChanged(GamePhase::Playing),
            Action::EmitTimeUpdated {
                elapsed_secs: new_state.elapsed_secs,
                display: new_state.display_time_string(),
            },
        ],
    }
}

/// End: Playing or Paused → Finished.
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
        finished_at: now_iso8601(),
    };

    let new_state = state.clone().with_phase(GamePhase::Finished);

    MatchResult {
        actions: vec![
            Action::StopTimer,
            Action::SaveMatch(snapshot),
            Action::PlaySound(SoundName::Whistle),
            Action::EmitMatchFinished {
                score_a: new_state.score_a,
                score_b: new_state.score_b,
            },
            Action::EmitPhaseChanged(GamePhase::Finished),
        ],
        new_state,
    }
}

/// Doubt: Playing (Normal) → sub=Challenge.
fn process_doubt(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Normal {
        return noop(state);
    }

    let new_state = state
        .clone()
        .with_sub_phase(PlayingSubPhase::Challenge);

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![
            Action::PlaySound(SoundName::Challenge),
            Action::EmitPhaseChanged(GamePhase::Playing),
        ],
    }
}

/// Resolve: Playing (Challenge) → sub=Normal.
fn process_resolve(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Challenge {
        return noop(state);
    }

    let new_state = state
        .clone()
        .with_sub_phase(PlayingSubPhase::Normal);

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![
            Action::EmitPhaseChanged(GamePhase::Playing),
        ],
    }
}

/// VoltaSeis: Playing (Challenge) → sub=Normal + play SixMeters sound.
fn process_volta_seis(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Playing || state.sub_phase != PlayingSubPhase::Challenge {
        return noop(state);
    }

    let new_state = state
        .clone()
        .with_sub_phase(PlayingSubPhase::Normal);

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![
            Action::PlaySound(SoundName::SixMeters),
            Action::EmitPhaseChanged(GamePhase::Playing),
        ],
    }
}

/// Reset: Finished → Idle (new match_id, scores zeroed).
fn process_reset(state: &MatchState) -> MatchResult {
    if state.phase != GamePhase::Finished {
        return noop(state);
    }

    let new_state = state
        .clone()
        .with_phase(GamePhase::Idle)
        .with_score_a(0)
        .with_score_b(0)
        .with_elapsed(0)
        .with_started_at(None)
        .with_paused_elapsed(0)
        .with_new_match_id();

    MatchResult {
        new_state: new_state.clone(),
        actions: vec![Action::EmitPhaseChanged(GamePhase::Idle)],
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

/// Returns an unchanged state with `[NoOp]` — used for invalid-phase commands.
fn noop(state: &MatchState) -> MatchResult {
    MatchResult {
        new_state: state.clone(),
        actions: vec![Action::NoOp],
    }
}

/// Current UTC timestamp as milliseconds since epoch.
fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock before UNIX epoch")
        .as_millis() as u64
}

/// Current UTC timestamp as ISO 8601 string.
fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339()
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::GameCommand;
    use crate::game::{GamePhase, MatchConfig, MatchState, PlayingSubPhase};

    // ── Helpers ──────────────────────────────────────────────────────────

    fn default_config() -> MatchConfig {
        MatchConfig::new("Time A", "Time B")
    }

    fn idle_state() -> MatchState {
        MatchState::new(default_config())
    }

    fn playing_state() -> MatchState {
        idle_state()
            .with_phase(GamePhase::Playing)
            .with_sub_phase(PlayingSubPhase::Normal)
            .with_started_at(Some(0))
    }

    fn playing_challenge() -> MatchState {
        playing_state().with_sub_phase(PlayingSubPhase::Challenge)
    }

    fn paused_state() -> MatchState {
        playing_state()
            .with_phase(GamePhase::Paused)
            .with_elapsed(60)
            .with_paused_elapsed(60)
    }

    fn finished_state() -> MatchState {
        playing_state()
            .with_phase(GamePhase::Finished)
            .with_score_a(3)
            .with_score_b(2)
    }

    /// Helper: assert result has a specific action variant.
    fn has_action(result: &MatchResult, predicate: impl Fn(&Action) -> bool) -> bool {
        result.actions.iter().any(predicate)
    }

    /// Helper: assert result is noop.
    fn assert_noop(result: &MatchResult, original: &MatchState) {
        assert_eq!(result.new_state.phase, original.phase);
        assert!(has_action(result, |a| *a == Action::NoOp));
    }

    // ══════════════════════════════════════════════════════════════════════
    // 1. Start
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn start_from_idle_transitions_to_playing() {
        let result = process(&idle_state(), GameCommand::Start);
        assert_eq!(result.new_state.phase, GamePhase::Playing);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Normal);
        assert_eq!(result.new_state.score_a, 0);
        assert_eq!(result.new_state.score_b, 0);
        assert_eq!(result.new_state.elapsed_secs, 0);
        assert!(result.new_state.started_at.is_some());
    }

    #[test]
    fn start_from_idle_produces_expected_actions() {
        let result = process(&idle_state(), GameCommand::Start);
        assert!(has_action(&result, |a| *a == Action::StartTimer));
        assert!(has_action(&result, |a| *a == Action::PlaySound(SoundName::Whistle)));
        assert!(has_action(&result, |a| *a == Action::EmitPhaseChanged(GamePhase::Playing)));
    }

    #[test]
    fn start_from_playing_is_noop() {
        let result = process(&playing_state(), GameCommand::Start);
        assert_noop(&result, &playing_state());
    }

    #[test]
    fn start_from_paused_is_noop() {
        let result = process(&paused_state(), GameCommand::Start);
        assert_noop(&result, &paused_state());
    }

    #[test]
    fn start_from_finished_is_noop() {
        let result = process(&finished_state(), GameCommand::Start);
        assert_noop(&result, &finished_state());
    }

    // ══════════════════════════════════════════════════════════════════════
    // 2. GoalA
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn goal_a_increments_score() {
        let result = process(&playing_state(), GameCommand::GoalA);
        assert_eq!(result.new_state.score_a, 1);
        assert_eq!(result.new_state.score_b, 0);
    }

    #[test]
    fn goal_a_produces_sound_and_score_action() {
        let result = process(&playing_state(), GameCommand::GoalA);
        assert!(has_action(&result, |a| *a == Action::PlaySound(SoundName::Goal)));
        assert!(has_action(&result, |a| matches!(a, Action::EmitScoreChanged { score_a: 1, score_b: 0 })));
    }

    #[test]
    fn goal_a_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::GoalA);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn goal_a_from_paused_is_noop() {
        let result = process(&paused_state(), GameCommand::GoalA);
        assert_noop(&result, &paused_state());
    }

    #[test]
    fn goal_a_from_challenge_is_noop() {
        let result = process(&playing_challenge(), GameCommand::GoalA);
        assert_noop(&result, &playing_challenge());
    }

    #[test]
    fn goal_a_multiple_goals() {
        let state = playing_state().with_score_a(4);
        let result = process(&state, GameCommand::GoalA);
        assert_eq!(result.new_state.score_a, 5);
    }

    // ══════════════════════════════════════════════════════════════════════
    // 3. GoalB
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn goal_b_increments_score() {
        let result = process(&playing_state(), GameCommand::GoalB);
        assert_eq!(result.new_state.score_b, 1);
        assert_eq!(result.new_state.score_a, 0);
    }

    #[test]
    fn goal_b_produces_sound_and_score_action() {
        let result = process(&playing_state(), GameCommand::GoalB);
        assert!(has_action(&result, |a| *a == Action::PlaySound(SoundName::Goal)));
        assert!(has_action(&result, |a| matches!(a, Action::EmitScoreChanged { score_a: 0, score_b: 1 })));
    }

    #[test]
    fn goal_b_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::GoalB);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn goal_b_from_challenge_is_noop() {
        let result = process(&playing_challenge(), GameCommand::GoalB);
        assert_noop(&result, &playing_challenge());
    }

    // ══════════════════════════════════════════════════════════════════════
    // 4. Pause
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn pause_from_playing_transitions_to_paused() {
        let result = process(&playing_state(), GameCommand::Pause);
        assert_eq!(result.new_state.phase, GamePhase::Paused);
    }

    #[test]
    fn pause_preserves_elapsed() {
        let state = playing_state().with_elapsed(90);
        let result = process(&state, GameCommand::Pause);
        assert_eq!(result.new_state.paused_elapsed_secs, 90);
        assert_eq!(result.new_state.elapsed_secs, 90);
    }

    #[test]
    fn pause_produces_stop_timer_and_phase_change() {
        let result = process(&playing_state(), GameCommand::Pause);
        assert!(has_action(&result, |a| *a == Action::StopTimer));
        assert!(has_action(&result, |a| *a == Action::EmitPhaseChanged(GamePhase::Paused)));
    }

    #[test]
    fn pause_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::Pause);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn pause_from_paused_is_noop() {
        let result = process(&paused_state(), GameCommand::Pause);
        assert_noop(&result, &paused_state());
    }

    #[test]
    fn pause_from_finished_is_noop() {
        let result = process(&finished_state(), GameCommand::Pause);
        assert_noop(&result, &finished_state());
    }

    // ══════════════════════════════════════════════════════════════════════
    // 5. Resume
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn resume_from_paused_transitions_to_playing() {
        let result = process(&paused_state(), GameCommand::Resume);
        assert_eq!(result.new_state.phase, GamePhase::Playing);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Normal);
    }

    #[test]
    fn resume_restores_elapsed_from_paused_elapsed() {
        let result = process(&paused_state(), GameCommand::Resume);
        assert_eq!(result.new_state.elapsed_secs, 60); // paused_elapsed was 60
    }

    #[test]
    fn resume_produces_start_timer_and_phase_change() {
        let result = process(&paused_state(), GameCommand::Resume);
        assert!(has_action(&result, |a| *a == Action::StartTimer));
        assert!(has_action(&result, |a| *a == Action::EmitPhaseChanged(GamePhase::Playing)));
        assert!(has_action(&result, |a| matches!(a, Action::EmitTimeUpdated { .. })));
    }

    #[test]
    fn resume_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::Resume);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn resume_from_playing_is_noop() {
        let result = process(&playing_state(), GameCommand::Resume);
        assert_noop(&result, &playing_state());
    }

    #[test]
    fn resume_from_finished_is_noop() {
        let result = process(&finished_state(), GameCommand::Resume);
        assert_noop(&result, &finished_state());
    }

    // ══════════════════════════════════════════════════════════════════════
    // 6. End
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn end_from_playing_transitions_to_finished() {
        let result = process(&playing_state(), GameCommand::End);
        assert_eq!(result.new_state.phase, GamePhase::Finished);
    }

    #[test]
    fn end_from_paused_transitions_to_finished() {
        let result = process(&paused_state(), GameCommand::End);
        assert_eq!(result.new_state.phase, GamePhase::Finished);
    }

    #[test]
    fn end_preserves_scores() {
        let state = playing_state().with_score_a(3).with_score_b(2);
        let result = process(&state, GameCommand::End);
        assert_eq!(result.new_state.score_a, 3);
        assert_eq!(result.new_state.score_b, 2);
    }

    #[test]
    fn end_produces_expected_actions() {
        let state = playing_state().with_score_a(1).with_score_b(1);
        let result = process(&state, GameCommand::End);
        assert!(has_action(&result, |a| *a == Action::StopTimer));
        assert!(has_action(&result, |a| *a == Action::PlaySound(SoundName::Whistle)));
        assert!(has_action(&result, |a| matches!(a, Action::SaveMatch(_))));
        assert!(has_action(&result, |a| matches!(a, Action::EmitMatchFinished { score_a: 1, score_b: 1 })));
        assert!(has_action(&result, |a| *a == Action::EmitPhaseChanged(GamePhase::Finished)));
    }

    #[test]
    fn end_save_match_snapshot_has_correct_data() {
        let state = playing_state().with_score_a(5).with_score_b(3).with_elapsed(480);
        let result = process(&state, GameCommand::End);
        let snapshot = result.actions.iter().find_map(|a| {
            if let Action::SaveMatch(s) = a { Some(s) } else { None }
        });
        assert!(snapshot.is_some());
        let snap = snapshot.unwrap();
        assert_eq!(snap.score_a, 5);
        assert_eq!(snap.score_b, 3);
        assert_eq!(snap.duration_secs, 480);
        assert_eq!(snap.match_id, state.match_id);
        assert_eq!(snap.team_a_name, "Time A");
        assert_eq!(snap.team_b_name, "Time B");
        assert!(!snap.finished_at.is_empty());
    }

    #[test]
    fn end_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::End);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn end_from_finished_is_noop() {
        let result = process(&finished_state(), GameCommand::End);
        assert_noop(&result, &finished_state());
    }

    // ══════════════════════════════════════════════════════════════════════
    // 7. Doubt
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn doubt_from_normal_transitions_to_challenge() {
        let result = process(&playing_state(), GameCommand::Doubt);
        assert_eq!(result.new_state.phase, GamePhase::Playing);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Challenge);
    }

    #[test]
    fn doubt_preserves_scores() {
        let state = playing_state().with_score_a(2).with_score_b(1);
        let result = process(&state, GameCommand::Doubt);
        assert_eq!(result.new_state.score_a, 2);
        assert_eq!(result.new_state.score_b, 1);
    }

    #[test]
    fn doubt_produces_challenge_sound() {
        let result = process(&playing_state(), GameCommand::Doubt);
        assert!(has_action(&result, |a| *a == Action::PlaySound(SoundName::Challenge)));
        assert!(has_action(&result, |a| *a == Action::EmitPhaseChanged(GamePhase::Playing)));
    }

    #[test]
    fn doubt_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::Doubt);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn doubt_from_paused_is_noop() {
        let result = process(&paused_state(), GameCommand::Doubt);
        assert_noop(&result, &paused_state());
    }

    #[test]
    fn doubt_from_challenge_is_noop() {
        let result = process(&playing_challenge(), GameCommand::Doubt);
        assert_noop(&result, &playing_challenge());
    }

    // ══════════════════════════════════════════════════════════════════════
    // 8. Resolve
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn resolve_from_challenge_transitions_to_normal() {
        let result = process(&playing_challenge(), GameCommand::Resolve);
        assert_eq!(result.new_state.phase, GamePhase::Playing);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Normal);
    }

    #[test]
    fn resolve_produces_phase_changed() {
        let result = process(&playing_challenge(), GameCommand::Resolve);
        assert!(has_action(&result, |a| *a == Action::EmitPhaseChanged(GamePhase::Playing)));
        // No sound for resolve
        assert!(!has_action(&result, |a| matches!(a, Action::PlaySound(_))));
    }

    #[test]
    fn resolve_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::Resolve);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn resolve_from_normal_is_noop() {
        let result = process(&playing_state(), GameCommand::Resolve);
        assert_noop(&result, &playing_state());
    }

    #[test]
    fn resolve_from_paused_is_noop() {
        let result = process(&paused_state(), GameCommand::Resolve);
        assert_noop(&result, &paused_state());
    }

    // ══════════════════════════════════════════════════════════════════════
    // 9. VoltaSeis
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn volta_seis_from_challenge_transitions_to_normal() {
        let result = process(&playing_challenge(), GameCommand::VoltaSeis);
        assert_eq!(result.new_state.phase, GamePhase::Playing);
        assert_eq!(result.new_state.sub_phase, PlayingSubPhase::Normal);
    }

    #[test]
    fn volta_seis_produces_six_meters_sound() {
        let result = process(&playing_challenge(), GameCommand::VoltaSeis);
        assert!(has_action(&result, |a| *a == Action::PlaySound(SoundName::SixMeters)));
        assert!(has_action(&result, |a| *a == Action::EmitPhaseChanged(GamePhase::Playing)));
    }

    #[test]
    fn volta_seis_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::VoltaSeis);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn volta_seis_from_normal_is_noop() {
        let result = process(&playing_state(), GameCommand::VoltaSeis);
        assert_noop(&result, &playing_state());
    }

    #[test]
    fn volta_seis_from_paused_is_noop() {
        let result = process(&paused_state(), GameCommand::VoltaSeis);
        assert_noop(&result, &paused_state());
    }

    // ══════════════════════════════════════════════════════════════════════
    // 10. Reset
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn reset_from_finished_transitions_to_idle() {
        let result = process(&finished_state(), GameCommand::Reset);
        assert_eq!(result.new_state.phase, GamePhase::Idle);
        assert_eq!(result.new_state.score_a, 0);
        assert_eq!(result.new_state.score_b, 0);
        assert_eq!(result.new_state.elapsed_secs, 0);
        assert_eq!(result.new_state.started_at, None);
    }

    #[test]
    fn reset_generates_new_match_id() {
        let result = process(&finished_state(), GameCommand::Reset);
        assert_ne!(result.new_state.match_id, finished_state().match_id);
    }

    #[test]
    fn reset_preserves_config() {
        let result = process(&finished_state(), GameCommand::Reset);
        assert_eq!(result.new_state.config.team_a_name, "Time A");
        assert_eq!(result.new_state.config.team_b_name, "Time B");
    }

    #[test]
    fn reset_produces_phase_changed() {
        let result = process(&finished_state(), GameCommand::Reset);
        assert!(has_action(&result, |a| *a == Action::EmitPhaseChanged(GamePhase::Idle)));
    }

    #[test]
    fn reset_from_idle_is_noop() {
        let result = process(&idle_state(), GameCommand::Reset);
        assert_noop(&result, &idle_state());
    }

    #[test]
    fn reset_from_playing_is_noop() {
        let result = process(&playing_state(), GameCommand::Reset);
        assert_noop(&result, &playing_state());
    }

    #[test]
    fn reset_from_paused_is_noop() {
        let result = process(&paused_state(), GameCommand::Reset);
        assert_noop(&result, &paused_state());
    }

    // ══════════════════════════════════════════════════════════════════════
    // Full lifecycle
    // ══════════════════════════════════════════════════════════════════════

    #[test]
    fn full_match_lifecycle() {
        let state = idle_state();

        // Start
        let r = process(&state, GameCommand::Start);
        assert_eq!(r.new_state.phase, GamePhase::Playing);

        // Goal A
        let r = process(&r.new_state, GameCommand::GoalA);
        assert_eq!(r.new_state.score_a, 1);

        // Goal B
        let r = process(&r.new_state, GameCommand::GoalB);
        assert_eq!(r.new_state.score_b, 1);

        // Pause
        let r = process(&r.new_state, GameCommand::Pause);
        assert_eq!(r.new_state.phase, GamePhase::Paused);

        // Resume
        let r = process(&r.new_state, GameCommand::Resume);
        assert_eq!(r.new_state.phase, GamePhase::Playing);

        // Doubt
        let r = process(&r.new_state, GameCommand::Doubt);
        assert_eq!(r.new_state.sub_phase, PlayingSubPhase::Challenge);

        // VoltaSeis (resolve via 6 meters)
        let r = process(&r.new_state, GameCommand::VoltaSeis);
        assert_eq!(r.new_state.sub_phase, PlayingSubPhase::Normal);

        // Another doubt + resolve
        let r = process(&r.new_state, GameCommand::Doubt);
        assert_eq!(r.new_state.sub_phase, PlayingSubPhase::Challenge);
        let r = process(&r.new_state, GameCommand::Resolve);
        assert_eq!(r.new_state.sub_phase, PlayingSubPhase::Normal);

        // End
        let r = process(&r.new_state, GameCommand::End);
        assert_eq!(r.new_state.phase, GamePhase::Finished);

        // Reset
        let r = process(&r.new_state, GameCommand::Reset);
        assert_eq!(r.new_state.phase, GamePhase::Idle);
        assert_eq!(r.new_state.score_a, 0);
        assert_eq!(r.new_state.score_b, 0);
    }

    #[test]
    fn immutability_original_state_unchanged() {
        let original = playing_state().with_score_a(2);
        let _result = process(&original, GameCommand::GoalA);
        assert_eq!(original.score_a, 2); // unchanged
    }

    #[test]
    fn determinism_same_input_same_output() {
        let state = playing_state();
        let r1 = process(&state, GameCommand::GoalA);
        let r2 = process(&state, GameCommand::GoalA);
        assert_eq!(r1.new_state.score_a, r2.new_state.score_a);
        assert_eq!(r1.actions.len(), r2.actions.len());
    }
}

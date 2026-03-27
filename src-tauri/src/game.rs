//! game.rs — Match State (pure domain, zero I/O)
//!
//! SRP: Define and manage the immutable data structure that represents
//! the complete state of a football match. No Tauri, no cpal, no rodio.

use serde::{Deserialize, Serialize};
use std::time::Duration;

// ── Enums ────────────────────────────────────────────────────────────────

/// Phase of the match (state machine).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    Idle,
    Playing,
    Paused,
    Finished,
}

/// Sub-state active while in Playing phase.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlayingSubPhase {
    Normal,
    Challenge,
}

/// Timer display mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Countdown,
    CountUp,
}

// ── Config ───────────────────────────────────────────────────────────────

/// Immutable match configuration (set before start, never changes during match).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchConfig {
    pub team_a_name: String,
    pub team_b_name: String,
    pub duration_secs: u64,
    pub timer_mode: TimerMode,
}

impl MatchConfig {
    /// Create a new match config with sensible defaults.
    pub fn new(team_a_name: &str, team_b_name: &str) -> Self {
        Self {
            team_a_name: team_a_name.to_owned(),
            team_b_name: team_b_name.to_owned(),
            duration_secs: 600, // 10 minutes
            timer_mode: TimerMode::Countdown,
        }
    }

    /// Builder: set match duration in seconds.
    pub fn with_duration(mut self, secs: u64) -> Self {
        self.duration_secs = secs;
        self
    }

    /// Builder: set timer mode.
    pub fn with_timer_mode(mut self, mode: TimerMode) -> Self {
        self.timer_mode = mode;
        self
    }
}

// ── MatchState ───────────────────────────────────────────────────────────

/// Complete immutable state of a match.
///
/// All mutations go through `with_*` methods that consume self and return
/// a new instance (persistent data structure pattern).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    pub phase: GamePhase,
    pub sub_phase: PlayingSubPhase,
    pub config: MatchConfig,
    pub score_a: u32,
    pub score_b: u32,
    pub elapsed_secs: u64,
    pub started_at: Option<u64>,       // timestamp millis
    pub paused_elapsed_secs: u64,      // accumulated elapsed when paused
    pub match_id: String,              // UUID
}

impl MatchState {
    /// Create a new Idle match state.
    pub fn new(config: MatchConfig) -> Self {
        Self {
            phase: GamePhase::Idle,
            sub_phase: PlayingSubPhase::Normal,
            config,
            score_a: 0,
            score_b: 0,
            elapsed_secs: 0,
            started_at: None,
            paused_elapsed_secs: 0,
            match_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    // ── Query methods ────────────────────────────────────────────────────

    /// Returns the duration to display:
    /// - Countdown: remaining time (duration - elapsed), 0 if time up
    /// - CountUp: elapsed time
    pub fn display_time(&self) -> Duration {
        match self.config.timer_mode {
            TimerMode::Countdown => {
                let remaining = self.config.duration_secs.saturating_sub(self.elapsed_secs);
                Duration::from_secs(remaining)
            }
            TimerMode::CountUp => Duration::from_secs(self.elapsed_secs),
        }
    }

    /// Returns true if the match has run out of time (countdown only).
    /// For CountUp mode, always returns false.
    pub fn is_time_up(&self) -> bool {
        self.config.timer_mode == TimerMode::Countdown
            && self.elapsed_secs >= self.config.duration_secs
    }

    /// Returns true if the match is currently active (Playing phase, any sub-phase).
    pub fn is_playing(&self) -> bool {
        self.phase == GamePhase::Playing
    }

    // ── Builder methods (immutable transitions) ──────────────────────────

    /// Set score for team A.
    pub fn with_score_a(mut self, score: u32) -> Self {
        self.score_a = score;
        self
    }

    /// Set score for team B.
    pub fn with_score_b(mut self, score: u32) -> Self {
        self.score_b = score;
        self
    }

    /// Set game phase. Also resets sub_phase to Normal when leaving Playing.
    pub fn with_phase(mut self, phase: GamePhase) -> Self {
        let old_phase = self.phase.clone();
        let is_playing = phase == GamePhase::Playing;
        self.phase = phase.clone();
        if !is_playing {
            self.sub_phase = PlayingSubPhase::Normal;
        }
        if old_phase != phase {
            eprintln!("[GAME] phase: {:?} → {:?}", old_phase, phase);
        }
        self
    }

    /// Set playing sub-phase.
    pub fn with_sub_phase(mut self, sub: PlayingSubPhase) -> Self {
        self.sub_phase = sub;
        self
    }

    /// Set elapsed seconds.
    pub fn with_elapsed(mut self, elapsed: u64) -> Self {
        self.elapsed_secs = elapsed;
        self
    }

    /// Set the started_at timestamp (millis).
    pub fn with_started_at(mut self, ts: Option<u64>) -> Self {
        self.started_at = ts;
        self
    }

    /// Set the paused_elapsed_secs accumulator.
    pub fn with_paused_elapsed(mut self, secs: u64) -> Self {
        self.paused_elapsed_secs = secs;
        self
    }

    /// Set a new match_id (useful for Reset → new match).
    pub fn with_new_match_id(mut self) -> Self {
        self.match_id = uuid::Uuid::new_v4().to_string();
        self
    }
}

// ── Display formatting ───────────────────────────────────────────────────

/// Format a Duration as MM:SS.
fn format_duration(d: Duration) -> String {
    let total_secs = d.as_secs();
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{mins:02}:{secs:02}")
}

impl MatchState {
    /// Returns the display time formatted as MM:SS.
    pub fn display_time_string(&self) -> String {
        format_duration(self.display_time())
    }
}

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> MatchConfig {
        MatchConfig::new("Time A", "Time B")
    }

    fn countdown_config(duration_secs: u64) -> MatchConfig {
        MatchConfig::new("Time A", "Time B").with_duration(duration_secs)
    }

    fn countup_config() -> MatchConfig {
        MatchConfig::new("Time A", "Time B").with_timer_mode(TimerMode::CountUp)
    }

    // ── new() ────────────────────────────────────────────────────────────

    #[test]
    fn new_creates_idle_state() {
        let state = MatchState::new(default_config());
        assert_eq!(state.phase, GamePhase::Idle);
        assert_eq!(state.sub_phase, PlayingSubPhase::Normal);
        assert_eq!(state.score_a, 0);
        assert_eq!(state.score_b, 0);
        assert_eq!(state.elapsed_secs, 0);
        assert_eq!(state.started_at, None);
        assert_eq!(state.paused_elapsed_secs, 0);
        assert!(!state.match_id.is_empty());
    }

    #[test]
    fn new_generates_unique_ids() {
        let a = MatchState::new(default_config());
        let b = MatchState::new(default_config());
        assert_ne!(a.match_id, b.match_id);
    }

    // ── with_score_* ─────────────────────────────────────────────────────

    #[test]
    fn with_score_a() {
        let state = MatchState::new(default_config()).with_score_a(3);
        assert_eq!(state.score_a, 3);
        assert_eq!(state.score_b, 0); // unchanged
    }

    #[test]
    fn with_score_b() {
        let state = MatchState::new(default_config()).with_score_b(5);
        assert_eq!(state.score_b, 5);
    }

    // ── with_phase ───────────────────────────────────────────────────────

    #[test]
    fn with_phase_playing() {
        let state = MatchState::new(default_config()).with_phase(GamePhase::Playing);
        assert_eq!(state.phase, GamePhase::Playing);
        assert_eq!(state.sub_phase, PlayingSubPhase::Normal);
    }

    #[test]
    fn with_phase_resets_sub_phase_when_not_playing() {
        let state = MatchState::new(default_config())
            .with_phase(GamePhase::Playing)
            .with_sub_phase(PlayingSubPhase::Challenge)
            .with_phase(GamePhase::Paused);
        assert_eq!(state.sub_phase, PlayingSubPhase::Normal);
    }

    #[test]
    fn with_phase_preserves_sub_phase_when_playing() {
        let state = MatchState::new(default_config())
            .with_phase(GamePhase::Playing)
            .with_sub_phase(PlayingSubPhase::Challenge);
        // Re-setting Playing should NOT reset sub_phase
        let state2 = state.clone().with_phase(GamePhase::Playing);
        assert_eq!(state2.sub_phase, PlayingSubPhase::Challenge);
    }

    // ── with_sub_phase ───────────────────────────────────────────────────

    #[test]
    fn with_sub_phase_challenge() {
        let state = MatchState::new(default_config())
            .with_phase(GamePhase::Playing)
            .with_sub_phase(PlayingSubPhase::Challenge);
        assert_eq!(state.sub_phase, PlayingSubPhase::Challenge);
    }

    // ── with_elapsed ─────────────────────────────────────────────────────

    #[test]
    fn with_elapsed() {
        let state = MatchState::new(default_config()).with_elapsed(120);
        assert_eq!(state.elapsed_secs, 120);
    }

    // ── with_started_at ──────────────────────────────────────────────────

    #[test]
    fn with_started_at() {
        let state = MatchState::new(default_config()).with_started_at(Some(1000));
        assert_eq!(state.started_at, Some(1000));
    }

    #[test]
    fn with_started_at_none() {
        let state = MatchState::new(default_config())
            .with_started_at(Some(1000))
            .with_started_at(None);
        assert_eq!(state.started_at, None);
    }

    // ── with_paused_elapsed ──────────────────────────────────────────────

    #[test]
    fn with_paused_elapsed() {
        let state = MatchState::new(default_config()).with_paused_elapsed(45);
        assert_eq!(state.paused_elapsed_secs, 45);
    }

    // ── with_new_match_id ────────────────────────────────────────────────

    #[test]
    fn with_new_match_id() {
        let state = MatchState::new(default_config());
        let original_id = state.match_id.clone();
        let state2 = state.with_new_match_id();
        assert_ne!(state2.match_id, original_id);
    }

    // ── display_time (Countdown) ─────────────────────────────────────────

    #[test]
    fn display_time_countdown_remaining() {
        let state = MatchState::new(countdown_config(600)).with_elapsed(120);
        assert_eq!(state.display_time(), Duration::from_secs(480));
    }

    #[test]
    fn display_time_countdown_zero_when_time_up() {
        let state = MatchState::new(countdown_config(300)).with_elapsed(300);
        assert_eq!(state.display_time(), Duration::from_secs(0));
    }

    #[test]
    fn display_time_countdown_clamps_at_zero() {
        let state = MatchState::new(countdown_config(300)).with_elapsed(500);
        assert_eq!(state.display_time(), Duration::from_secs(0));
    }

    // ── display_time (CountUp) ───────────────────────────────────────────

    #[test]
    fn display_time_countup_elapsed() {
        let state = MatchState::new(countup_config()).with_elapsed(90);
        assert_eq!(state.display_time(), Duration::from_secs(90));
    }

    // ── is_time_up ───────────────────────────────────────────────────────

    #[test]
    fn is_time_up_false_before_duration() {
        let state = MatchState::new(countdown_config(600)).with_elapsed(300);
        assert!(!state.is_time_up());
    }

    #[test]
    fn is_time_up_true_at_duration() {
        let state = MatchState::new(countdown_config(600)).with_elapsed(600);
        assert!(state.is_time_up());
    }

    #[test]
    fn is_time_up_true_after_duration() {
        let state = MatchState::new(countdown_config(600)).with_elapsed(601);
        assert!(state.is_time_up());
    }

    #[test]
    fn is_time_up_false_for_countup() {
        let state = MatchState::new(countup_config()).with_elapsed(99999);
        assert!(!state.is_time_up());
    }

    // ── is_playing ───────────────────────────────────────────────────────

    #[test]
    fn is_playing_true() {
        let state = MatchState::new(default_config()).with_phase(GamePhase::Playing);
        assert!(state.is_playing());
    }

    #[test]
    fn is_playing_false_for_other_phases() {
        for phase in [GamePhase::Idle, GamePhase::Paused, GamePhase::Finished] {
            let state = MatchState::new(default_config()).with_phase(phase);
            assert!(!state.is_playing());
        }
    }

    // ── display_time_string ──────────────────────────────────────────────

    #[test]
    fn display_time_string_format() {
        let state = MatchState::new(countdown_config(600)).with_elapsed(330);
        assert_eq!(state.display_time_string(), "04:30");
    }

    #[test]
    fn display_time_string_under_one_min() {
        let state = MatchState::new(countdown_config(600)).with_elapsed(570);
        assert_eq!(state.display_time_string(), "00:30");
    }

    #[test]
    fn display_time_string_zero() {
        let state = MatchState::new(countdown_config(600)).with_elapsed(600);
        assert_eq!(state.display_time_string(), "00:00");
    }

    // ── MatchConfig builder ──────────────────────────────────────────────

    #[test]
    fn config_new_defaults() {
        let cfg = MatchConfig::new("Alpha", "Beta");
        assert_eq!(cfg.team_a_name, "Alpha");
        assert_eq!(cfg.team_b_name, "Beta");
        assert_eq!(cfg.duration_secs, 600);
        assert_eq!(cfg.timer_mode, TimerMode::Countdown);
    }

    #[test]
    fn config_builder_chaining() {
        let cfg = MatchConfig::new("A", "B")
            .with_duration(900)
            .with_timer_mode(TimerMode::CountUp);
        assert_eq!(cfg.duration_secs, 900);
        assert_eq!(cfg.timer_mode, TimerMode::CountUp);
    }

    // ── Serialization round-trip ─────────────────────────────────────────

    #[test]
    fn serde_roundtrip() {
        let state = MatchState::new(
            MatchConfig::new("Team X", "Team Y")
                .with_duration(480)
                .with_timer_mode(TimerMode::CountUp),
        )
        .with_phase(GamePhase::Playing)
        .with_score_a(2)
        .with_score_b(1)
        .with_elapsed(180)
        .with_started_at(Some(1711545600000));

        let json = serde_json::to_string(&state).unwrap();
        let deserialized: MatchState = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.phase, state.phase);
        assert_eq!(deserialized.score_a, state.score_a);
        assert_eq!(deserialized.score_b, state.score_b);
        assert_eq!(deserialized.elapsed_secs, state.elapsed_secs);
        assert_eq!(deserialized.started_at, state.started_at);
        assert_eq!(deserialized.config.duration_secs, 480);
        assert_eq!(deserialized.config.timer_mode, TimerMode::CountUp);
    }

    #[test]
    fn serde_phase_is_snake_case() {
        assert_eq!(
            serde_json::to_string(&GamePhase::Playing).unwrap(),
            "\"playing\""
        );
    }

    #[test]
    fn serde_sub_phase_is_snake_case() {
        assert_eq!(
            serde_json::to_string(&PlayingSubPhase::Challenge).unwrap(),
            "\"challenge\""
        );
    }

    #[test]
    fn serde_timer_mode_is_snake_case() {
        assert_eq!(
            serde_json::to_string(&TimerMode::CountUp).unwrap(),
            "\"count_up\""
        );
    }

    // ── Full lifecycle simulation ────────────────────────────────────────

    #[test]
    fn full_match_lifecycle() {
        let idle = MatchState::new(countdown_config(120));

        // Start
        let playing = idle
            .clone()
            .with_phase(GamePhase::Playing)
            .with_started_at(Some(0));

        // Goal A
        let goal_a = playing.clone().with_score_a(1);
        assert_eq!(goal_a.score_a, 1);

        // Goal B
        let goal_b = goal_a.clone().with_score_b(1);
        assert_eq!(goal_b.score_b, 1);

        // Elapse time
        let ticked = goal_b.clone().with_elapsed(60);
        assert_eq!(ticked.display_time_string(), "01:00");
        assert!(!ticked.is_time_up());

        // Challenge
        let challenge = ticked
            .clone()
            .with_sub_phase(PlayingSubPhase::Challenge);
        assert_eq!(challenge.sub_phase, PlayingSubPhase::Challenge);

        // Resolve
        let resolved = challenge.with_sub_phase(PlayingSubPhase::Normal);
        assert_eq!(resolved.sub_phase, PlayingSubPhase::Normal);

        // Time up
        let time_up = resolved.clone().with_elapsed(120);
        assert!(time_up.is_time_up());
        assert_eq!(time_up.display_time_string(), "00:00");

        // Finish
        let finished = time_up.with_phase(GamePhase::Finished);
        assert_eq!(finished.phase, GamePhase::Finished);
        assert_eq!(finished.score_a, 1);
        assert_eq!(finished.score_b, 1);

        // Reset (clone finished before consuming it)
        let finished_id = finished.match_id.clone();
        let reset = finished
            .with_phase(GamePhase::Idle)
            .with_score_a(0)
            .with_score_b(0)
            .with_elapsed(0)
            .with_started_at(None)
            .with_new_match_id();
        assert_eq!(reset.phase, GamePhase::Idle);
        assert_ne!(reset.match_id, finished_id);
    }

    // ── Immutability: original unchanged ─────────────────────────────────

    #[test]
    fn with_methods_do_not_mutate_original() {
        let original = MatchState::new(default_config());
        let _modified = original
            .clone()
            .with_phase(GamePhase::Playing)
            .with_score_a(5)
            .with_elapsed(300);

        assert_eq!(original.phase, GamePhase::Idle);
        assert_eq!(original.score_a, 0);
        assert_eq!(original.elapsed_secs, 0);
    }

    // ── Pause / Resume with elapsed accumulation ─────────────────────────

    #[test]
    fn pause_accumulates_elapsed() {
        // Play 60s, pause, resume
        let playing = MatchState::new(countdown_config(600))
            .with_phase(GamePhase::Playing)
            .with_elapsed(60);

        let paused = playing.clone().with_phase(GamePhase::Paused);
        assert_eq!(paused.paused_elapsed_secs, playing.paused_elapsed_secs); // 0

        // Simulate resume with accumulated elapsed
        let resumed = paused
            .with_phase(GamePhase::Playing)
            .with_paused_elapsed(60); // remember where we were
        assert_eq!(resumed.elapsed_secs, 60); // still 60
        assert_eq!(resumed.paused_elapsed_secs, 60);

        // After 30 more seconds of play
        let later = resumed.with_elapsed(90);
        assert_eq!(later.display_time_string(), "08:30");
    }
}

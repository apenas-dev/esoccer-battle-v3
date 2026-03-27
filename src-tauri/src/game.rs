use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ── Enums ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    Idle,
    Playing,
    Paused,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayingSubPhase {
    Normal,
    Challenge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Countdown,
    CountUp,
}

// ── Config ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchConfig {
    pub team_a_name: String,
    pub team_b_name: String,
    pub duration_secs: u64,
    pub timer_mode: TimerMode,
}

impl Default for MatchConfig {
    fn default() -> Self {
        Self {
            team_a_name: "Time A".into(),
            team_b_name: "Time B".into(),
            duration_secs: 600,
            timer_mode: TimerMode::Countdown,
        }
    }
}

// ── State ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    pub phase: GamePhase,
    pub sub_phase: PlayingSubPhase,
    pub config: MatchConfig,
    pub score_a: u32,
    pub score_b: u32,
    pub elapsed_secs: u64,
    pub started_at: Option<u64>,
    pub paused_elapsed_secs: u64,
    pub match_id: String,
}

impl MatchState {
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
            match_id: Uuid::new_v4().to_string(),
        }
    }

    // ── Builders ──

    pub fn with_phase(mut self, phase: GamePhase) -> Self {
        self.phase = phase;
        self
    }

    pub fn with_sub_phase(mut self, sub_phase: PlayingSubPhase) -> Self {
        self.sub_phase = sub_phase;
        self
    }

    pub fn with_score_a(mut self, score_a: u32) -> Self {
        self.score_a = score_a;
        self
    }

    pub fn with_score_b(mut self, score_b: u32) -> Self {
        self.score_b = score_b;
        self
    }

    pub fn with_elapsed_secs(mut self, elapsed_secs: u64) -> Self {
        self.elapsed_secs = elapsed_secs;
        self
    }

    // ── Logic ──

    pub fn display_time(&self) -> String {
        let secs = match self.config.timer_mode {
            TimerMode::Countdown => {
                self.config
                    .duration_secs
                    .saturating_sub(self.elapsed_secs)
            }
            TimerMode::CountUp => self.elapsed_secs,
        };
        format!("{:02}:{:02}", secs / 60, secs % 60)
    }

    pub fn is_time_up(&self) -> bool {
        if self.config.timer_mode != TimerMode::Countdown {
            return false;
        }
        self.elapsed_secs >= self.config.duration_secs
    }
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_defaults() {
        let state = MatchState::new(MatchConfig::default());
        assert_eq!(state.phase, GamePhase::Idle);
        assert_eq!(state.score_a, 0);
        assert!(!state.match_id.is_empty());
    }

    #[test]
    fn builders_chain() {
        let state = MatchState::new(MatchConfig::default())
            .with_phase(GamePhase::Playing)
            .with_score_a(3)
            .with_score_b(1)
            .with_elapsed_secs(120);
        assert_eq!(state.score_a, 3);
        assert_eq!(state.score_b, 1);
        assert_eq!(state.elapsed_secs, 120);
    }

    #[test]
    fn display_time_countdown() {
        let state = MatchState::new(MatchConfig::default())
            .with_elapsed_secs(90);
        assert_eq!(state.display_time(), "08:30");
    }

    #[test]
    fn display_time_countup() {
        let state = MatchState::new(MatchConfig {
            timer_mode: TimerMode::CountUp,
            ..MatchConfig::default()
        })
        .with_elapsed_secs(65);
        assert_eq!(state.display_time(), "01:05");
    }

    #[test]
    fn is_time_up() {
        let state = MatchState::new(MatchConfig::default())
            .with_elapsed_secs(600);
        assert!(state.is_time_up());
    }

    #[test]
    fn is_time_up_countup_never() {
        let state = MatchState::new(MatchConfig {
            timer_mode: TimerMode::CountUp,
            ..MatchConfig::default()
        })
        .with_elapsed_secs(9999);
        assert!(!state.is_time_up());
    }

    #[test]
    fn serialization_roundtrip() {
        let state = MatchState::new(MatchConfig::default()).with_score_a(2);
        let json = serde_json::to_string(&state).unwrap();
        let back: MatchState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.score_a, 2);
        assert_eq!(back.match_id, state.match_id);
    }
}

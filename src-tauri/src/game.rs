use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Fase da partida (máquina de estados)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    Idle,
    Playing,
    Paused,
    Finished,
}

impl std::fmt::Display for GamePhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GamePhase::Idle => write!(f, "idle"),
            GamePhase::Playing => write!(f, "playing"),
            GamePhase::Paused => write!(f, "paused"),
            GamePhase::Finished => write!(f, "finished"),
        }
    }
}

/// Sub-estado ativo durante Playing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PlayingSubPhase {
    Normal,
    Challenge,
}

/// Modo do cronômetro
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Countdown,
    CountUp,
}

/// Configuração da partida (imutável após início)
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
            team_a_name: "Time A".to_string(),
            team_b_name: "Time B".to_string(),
            duration_secs: 600,
            timer_mode: TimerMode::Countdown,
        }
    }
}

/// Estado completo e imutável da partida
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
    /// Cria estado inicial Idle
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

    /// Retorna duração formatada baseada no TimerMode
    pub fn display_time(&self) -> Duration {
        match self.config.timer_mode {
            TimerMode::CountUp => Duration::from_secs(self.elapsed_secs),
            TimerMode::Countdown => {
                let remaining = self.config.duration_secs.saturating_sub(self.elapsed_secs);
                Duration::from_secs(remaining)
            }
        }
    }

    /// Retorna true se a partida acabou por tempo
    pub fn is_time_up(&self) -> bool {
        match self.config.timer_mode {
            TimerMode::Countdown => self.elapsed_secs >= self.config.duration_secs,
            TimerMode::CountUp => false,
        }
    }

    pub fn with_score_a(mut self, score: u32) -> Self { self.score_a = score; self }
    pub fn with_score_b(mut self, score: u32) -> Self { self.score_b = score; self }
    pub fn with_phase(mut self, phase: GamePhase) -> Self { self.phase = phase; self }
    pub fn with_sub_phase(mut self, sub: PlayingSubPhase) -> Self { self.sub_phase = sub; self }
    pub fn with_elapsed(mut self, elapsed: u64) -> Self { self.elapsed_secs = elapsed; self }
    pub fn with_started_at(mut self, ts: Option<u64>) -> Self { self.started_at = ts; self }
    pub fn with_paused_elapsed(mut self, elapsed: u64) -> Self { self.paused_elapsed_secs = elapsed; self }
    pub fn with_match_id(mut self, id: String) -> Self { self.match_id = id; self }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_state() -> MatchState {
        MatchState::new(MatchConfig::default())
    }

    #[test]
    fn new_creates_idle_state() {
        let state = default_state();
        assert_eq!(state.phase, GamePhase::Idle);
        assert_eq!(state.score_a, 0);
        assert!(state.started_at.is_none());
    }

    #[test]
    fn with_score_a_increments() {
        let state = default_state().with_score_a(3);
        assert_eq!(state.score_a, 3);
    }

    #[test]
    fn display_time_countdown() {
        let state = MatchState::new(MatchConfig {
            timer_mode: TimerMode::Countdown,
            duration_secs: 600,
            ..Default::default()
        }).with_elapsed(120);
        assert_eq!(state.display_time(), Duration::from_secs(480));
    }

    #[test]
    fn display_time_countup() {
        let state = MatchState::new(MatchConfig {
            timer_mode: TimerMode::CountUp,
            duration_secs: 600,
            ..Default::default()
        }).with_elapsed(120);
        assert_eq!(state.display_time(), Duration::from_secs(120));
    }

    #[test]
    fn is_time_up_countdown() {
        let state = MatchState::new(MatchConfig {
            timer_mode: TimerMode::Countdown,
            duration_secs: 600,
            ..Default::default()
        }).with_elapsed(600);
        assert!(state.is_time_up());
    }

    #[test]
    fn is_time_up_countup_never() {
        let state = MatchState::new(MatchConfig {
            timer_mode: TimerMode::CountUp,
            duration_secs: 600,
            ..Default::default()
        }).with_elapsed(9999);
        assert!(!state.is_time_up());
    }

    #[test]
    fn serialization_roundtrip() {
        let state = default_state();
        let json = serde_json::to_string(&state).unwrap();
        let deserialized: MatchState = serde_json::from_str(&json).unwrap();
        assert_eq!(state.match_id, deserialized.match_id);
    }
}

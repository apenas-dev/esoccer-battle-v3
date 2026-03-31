use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Fase da partida — 4 estados, zero sub-fase
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GamePhase {
    Idle,
    Playing,
    Paused,
    Finished,
}

/// Configuração da partida (imutável após início)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchConfig {
    pub team_a_name: String,
    pub team_b_name: String,
    pub duration_secs: u64,
    pub timer_mode: TimerMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TimerMode {
    Countdown,
    CountUp,
}

/// Estado completo e imutável da partida
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    pub phase: GamePhase,
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
            config,
            score_a: 0,
            score_b: 0,
            elapsed_secs: 0,
            started_at: None,
            paused_elapsed_secs: 0,
            match_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    pub fn display_time(&self) -> Duration {
        Duration::from_secs(self.elapsed_secs)
    }

    pub fn is_time_up(&self) -> bool {
        match self.config.timer_mode {
            TimerMode::Countdown => self.elapsed_secs >= self.config.duration_secs,
            TimerMode::CountUp => false,
        }
    }

    pub fn with_score_a(self, score: u32) -> Self {
        Self { score_a: score, ..self }
    }

    pub fn with_score_b(self, score: u32) -> Self {
        Self { score_b: score, ..self }
    }

    pub fn with_phase(self, phase: GamePhase) -> Self {
        Self { phase, ..self }
    }

    pub fn with_elapsed(self, elapsed: u64) -> Self {
        Self { elapsed_secs: elapsed, ..self }
    }

    pub fn with_started_at(self, started_at: u64) -> Self {
        Self { started_at: Some(started_at), ..self }
    }
}

impl From<crate::config::AppConfig> for MatchConfig {
    fn from(config: crate::config::AppConfig) -> Self {
        Self {
            team_a_name: config.team_a_name,
            team_b_name: config.team_b_name,
            duration_secs: config.match_duration_secs,
            timer_mode: config.timer_mode,
        }
    }
}

use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use crate::game::TimerMode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mic_device: Option<String>,
    pub whisper_model: WhisperModel,
    pub language: Language,
    pub voice_threshold: f32,
    pub team_a_name: String,
    pub team_b_name: String,
    pub theme: Theme,
    pub match_duration_secs: u64,
    pub timer_mode: TimerMode,
    pub volume: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum WhisperModel { Tiny, Base, Small }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language { PtBr, En, Es }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme { Dark, Light }

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mic_device: None,
            whisper_model: WhisperModel::Base,
            language: Language::PtBr,
            voice_threshold: 0.3,
            team_a_name: "Time A".to_string(),
            team_b_name: "Time B".to_string(),
            theme: Theme::Dark,
            match_duration_secs: 600,
            timer_mode: TimerMode::Countdown,
            volume: 0.7,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let path = config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = std::fs::read_to_string(&path).map_err(|e| ConfigError::Io(e.to_string()))?;
        serde_json::from_str(&contents).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io(e.to_string()))?;
        }
        let contents = serde_json::to_string_pretty(self).map_err(|e| ConfigError::Parse(e.to_string()))?;
        std::fs::write(&path, contents).map_err(|e| ConfigError::Io(e.to_string()))
    }
}

pub fn config_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("esoccer-battle")
        .join("config.json")
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

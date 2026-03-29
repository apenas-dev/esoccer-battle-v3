use crate::game::TimerMode;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
pub enum WhisperModel {
    Tiny,
    Base,
    Small,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    PtBr,
    En,
    Es,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    Validation(String),
}

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
            let default = Self::default();
            default.save()?;
            return Ok(default);
        }

        let content = std::fs::read_to_string(&path).map_err(|e| ConfigError::Io(e.to_string()))?;
        serde_json::from_str(&content).map_err(|e| ConfigError::Parse(e.to_string()))
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        self.validate()?;

        let path = config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::Io(e.to_string()))?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| ConfigError::Parse(e.to_string()))?;
        std::fs::write(&path, content).map_err(|e| ConfigError::Io(e.to_string()))
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if !(0.0..=1.0).contains(&self.voice_threshold) {
            return Err(ConfigError::Validation(
                "voice_threshold must be between 0.0 and 1.0".into(),
            ));
        }
        if !(0.0..=1.0).contains(&self.volume) {
            return Err(ConfigError::Validation(
                "volume must be between 0.0 and 1.0".into(),
            ));
        }
        if !(60..=7200).contains(&self.match_duration_secs) {
            return Err(ConfigError::Validation(
                "match_duration_secs must be between 60 and 7200".into(),
            ));
        }
        Ok(())
    }
}

pub fn config_path() -> PathBuf {
    let base = directories::ProjectDirs::from("com", "esoccer", "battle")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("config.json")
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(s) => write!(f, "IO error: {}", s),
            ConfigError::Parse(s) => write!(f, "Parse error: {}", s),
            ConfigError::Validation(s) => write!(f, "Validation error: {}", s),
        }
    }
}

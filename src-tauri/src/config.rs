//! Persistent application configuration — load/save JSON with validation.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::game::TimerMode;

// ── Enums ────────────────────────────────────────────────────────────────

/// Maximum allowed match duration in seconds (90 minutes).
pub const MAX_MATCH_DURATION_SECS: u64 = 5400;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WhisperModel {
    Tiny,
    Base,
    Small,
}

impl WhisperModel {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Tiny => "tiny",
            Self::Base => "base",
            Self::Small => "small",
        }
    }

    pub fn from_str_lossy(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "tiny" => Some(Self::Tiny),
            "base" => Some(Self::Base),
            "small" => Some(Self::Small),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    PtBr,
    En,
    Es,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Dark,
    Light,
}

// ── AppConfig ─────────────────────────────────────────────────────────────

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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mic_device: None,
            whisper_model: WhisperModel::Small,
            language: Language::PtBr,
            voice_threshold: 0.3,
            team_a_name: String::from("Time A"),
            team_b_name: String::from("Time B"),
            theme: Theme::Dark,
            match_duration_secs: 600, // 10 minutes
            timer_mode: TimerMode::Countdown,
            volume: 0.7,
        }
    }
}

// ── Validation ───────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum ConfigError {
    Io(String),
    Parse(String),
    Validation(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(msg) => write!(f, "IO error: {msg}"),
            Self::Parse(msg) => write!(f, "Parse error: {msg}"),
            Self::Validation(msg) => write!(f, "Validation error: {msg}"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl AppConfig {
    /// Validate all fields. Returns a list of errors (empty = valid).
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if !(0.0..=1.0).contains(&self.voice_threshold) {
            errors.push(format!(
                "voice_threshold must be between 0.0 and 1.0, got {}",
                self.voice_threshold
            ));
        }

        if !(0.0..=1.0).contains(&self.volume) {
            errors.push(format!(
                "volume must be between 0.0 and 1.0, got {}",
                self.volume
            ));
        }

        if self.match_duration_secs == 0 {
            errors.push("match_duration_secs must be greater than 0".to_string());
        }

        if self.match_duration_secs > MAX_MATCH_DURATION_SECS {
            errors.push(format!(
                "match_duration_secs must be <= {} ({} minutes), got {}",
                MAX_MATCH_DURATION_SECS,
                MAX_MATCH_DURATION_SECS / 60,
                self.match_duration_secs
            ));
        }

        if self.team_a_name.trim().is_empty() {
            errors.push("team_a_name must not be empty".to_string());
        }

        if self.team_b_name.trim().is_empty() {
            errors.push("team_b_name must not be empty".to_string());
        }

        errors
    }

    /// Load config from disk, creating defaults if file doesn't exist.
    pub fn load() -> Result<Self, ConfigError> {
        let path = config_path();

        if !path.exists() {
            let defaults = Self::default();
            defaults.save()?;
            eprintln!("[CONFIG] created default config at {:?}", path);
            return Ok(defaults);
        }

        let contents = std::fs::read_to_string(&path).map_err(|e| {
            ConfigError::Io(format!("Failed to read config at {:?}: {e}", path))
        })?;

        let config: AppConfig = serde_json::from_str(&contents).map_err(|e| {
            ConfigError::Parse(format!("Failed to parse config JSON: {e}"))
        })?;

        let errors = config.validate();
        if !errors.is_empty() {
            eprintln!("[CONFIG] validation errors: {}", errors.join("; "));
            return Err(ConfigError::Validation(errors.join("; ")));
        }

        eprintln!("[CONFIG] loaded from {:?}", path);
        Ok(config)
    }

    /// Load from disk, falling back to defaults on any error.
    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to load config, using defaults: {e}");
                Self::default()
            }
        }
    }

    /// Save config to disk as formatted JSON.
    pub fn save(&self) -> Result<(), ConfigError> {
        let path = config_path();

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ConfigError::Io(format!("Failed to create config directory: {e}"))
            })?;
        }

        let json = serde_json::to_string_pretty(self).map_err(|e| {
            ConfigError::Parse(format!("Failed to serialize config: {e}"))
        })?;

        std::fs::write(&path, json).map_err(|e| {
            ConfigError::Io(format!("Failed to write config to {:?}: {e}", path))
        })?;

        Ok(())
    }
}

// ── Config file path ─────────────────────────────────────────────────────

/// Returns the path to the JSON config file using the system config directory.
pub fn config_path() -> PathBuf {
    let dir = dirs()
        .map(|d| d.config_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    dir.join("esoccer-battle").join("config.json")
}

/// Platform-agnostic config directory resolution.
fn dirs() -> Option<directories::ProjectDirs> {
    directories::ProjectDirs::from("com", "esoccer", "battle")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_is_valid() {
        let config = AppConfig::default();
        assert!(config.validate().is_empty());
    }

    #[test]
    fn reject_invalid_threshold() {
        let mut config = AppConfig::default();
        config.voice_threshold = 1.5;
        assert!(!config.validate().is_empty());
    }

    #[test]
    fn reject_invalid_volume() {
        let mut config = AppConfig::default();
        config.volume = -0.1;
        assert!(!config.validate().is_empty());
    }

    #[test]
    fn reject_zero_duration() {
        let mut config = AppConfig::default();
        config.match_duration_secs = 0;
        assert!(!config.validate().is_empty());
    }

    #[test]
    fn reject_duration_exceeding_max() {
        let mut config = AppConfig::default();
        config.match_duration_secs = 7200; // 120 min > 90 min max
        assert!(!config.validate().is_empty());
        assert!(config.validate().iter().any(|e| e.contains("5400")));
    }

    #[test]
    fn accept_duration_at_max() {
        let mut config = AppConfig::default();
        config.match_duration_secs = 5400; // exactly 90 min
        assert!(config.validate().is_empty());
    }

    #[test]
    fn reject_empty_team_names() {
        let mut config = AppConfig::default();
        config.team_a_name = String::new();
        let errors = config.validate();
        assert!(errors.iter().any(|e| e.contains("team_a_name")));
    }

    #[test]
    fn serde_roundtrip() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let back: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.whisper_model, back.whisper_model);
        assert_eq!(config.language, back.language);
        assert_eq!(config.theme, back.theme);
        assert_eq!(config.timer_mode, back.timer_mode);
    }
}

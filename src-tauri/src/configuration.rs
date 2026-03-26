use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::warn;

fn config_path<D>() -> PathBuf {
    let dir = crate::project_directory().config_dir().to_path_buf();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!("Failed to create config directory: {e}");
    }
    dir.join("settings.bin")
}

pub fn save<D: Serialize>(data: &D) -> anyhow::Result<()> {
    let bytes = bincode::serialize(data)?;
    std::fs::write(config_path::<D>(), bytes)?;
    Ok(())
}

pub fn load<D: DeserializeOwned>() -> anyhow::Result<D> {
    let path = config_path::<D>();
    let data = std::fs::read(&path)?;
    Ok(bincode::deserialize(&data)?)
}

/// Returns `Ok(true)` if the config file exists, `Ok(false)` if it doesn't (first run).
pub fn exists<D>() -> bool {
    config_path::<D>().exists()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub mic_device: Option<String>,
    pub model: String,
    pub language: String,
    pub voice_threshold: f32,
    pub theme: String,
    pub team_a_name: String,
    pub team_b_name: String,
}

impl AppSettings {
    /// Hard-coded defaults with zero I/O.
    pub fn new() -> Self {
        Self {
            mic_device: None,
            model: "MediumWhisper".to_owned(),
            language: "pt".to_owned(),
            voice_threshold: 0.3,
            theme: "dark".to_owned(),
            team_a_name: "Time A".to_owned(),
            team_b_name: "Time B".to_owned(),
        }
    }

    /// Load from disk, returning error on failure.
    pub fn load() -> anyhow::Result<Self> {
        load::<Self>()
    }

    /// Try to load from disk; fall back to [`Self::new()`].
    pub fn load_or_default() -> Self {
        match load::<Self>() {
            Ok(s) => s,
            Err(e) if e.downcast_ref::<std::io::Error>().map_or(false, |io| io.kind() == std::io::ErrorKind::NotFound) => {
                // First run — no config file yet. Save defaults for next time.
                let defaults = Self::new();
                if let Err(save_err) = save(&defaults) {
                    tracing::warn!("Failed to save default settings: {save_err}");
                }
                defaults
            }
            Err(e) => {
                warn!("Failed to load settings, using defaults: {e}");
                Self::new()
            }
        }
    }
}

impl Default for AppSettings {
    /// Returns hard-coded defaults (no I/O).
    fn default() -> Self {
        Self::new()
    }
}

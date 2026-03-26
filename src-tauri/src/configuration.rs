use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tracing::warn;

fn config_path<D>() -> PathBuf {
    let dir = crate::project_directory().config_dir().to_path_buf();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!("Failed to create config directory: {e}");
    }
    dir.join(format!("{}.bin", std::any::type_name::<D>()).replace("::", "-"))
}

pub fn save<D: Serialize>(data: &D) -> anyhow::Result<()> {
    let bytes = bincode::serialize(data)?;
    std::fs::write(config_path::<D>(), bytes)?;
    Ok(())
}

pub fn load<D: DeserializeOwned>() -> anyhow::Result<D> {
    let data = std::fs::read(config_path::<D>())?;
    Ok(bincode::deserialize(&data)?)
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
            model: "base".to_owned(),
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

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

impl Default for AppSettings {
    fn default() -> Self {
        if let Ok(this) = load::<Self>() {
            return this;
        }
        let this = Self {
            mic_device: None,
            model: "base".to_owned(),
            language: "pt".to_owned(),
            voice_threshold: 0.3,
            theme: "dark".to_owned(),
            team_a_name: "Time A".to_owned(),
            team_b_name: "Time B".to_owned(),
        };
        if let Err(e) = save(&this) {
            warn!("Failed to save default settings: {e}");
        }
        this
    }
}

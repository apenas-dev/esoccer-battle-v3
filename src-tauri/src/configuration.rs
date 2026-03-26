use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize, Serialize};

fn config_path<D>() -> PathBuf {
    let dir = crate::project_directory().config_dir().to_path_buf();
    std::fs::create_dir_all(&dir).expect("Can't create config directory");
    dir.join(format!("{}.bin", std::any::type_name::<D>()).replace("::", "-"))
}

pub fn save<D: Serialize>(data: &D) {
    let data = bincode::serialize(data).expect("Can't serialize config");
    std::fs::write(config_path::<D>(), data).expect("Can't save configuration");
}

fn load<D: DeserializeOwned>() -> anyhow::Result<D> {
    let data = std::fs::read(config_path::<D>())?;
    Ok(bincode::deserialize(&data)?)
}

fn data_dir() -> PathBuf {
    let data_dir = crate::project_directory().data_dir().to_path_buf();
    std::fs::create_dir_all(&data_dir).expect("Can't create data directory");
    data_dir
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavePathConfig {
    pub save_path: PathBuf,
}

impl Default for SavePathConfig {
    fn default() -> Self {
        Self {
            save_path: data_dir(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    pub save_to: SavePathConfig,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        if let Ok(this) = load::<Self>() {
            return this;
        }
        let this = Self {
            save_to: SavePathConfig::default(),
        };
        save(&this);
        this
    }
}

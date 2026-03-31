use std::path::PathBuf;
use std::sync::OnceLock;

static RESOURCE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub enum SoundFile {
    Goal,
    Whistle,
}

impl SoundFile {
    pub fn filename(&self) -> &'static str {
        match self {
            SoundFile::Goal => "goal.wav",
            SoundFile::Whistle => "whistle.wav",
        }
    }
}

pub async fn play(sound: SoundFile) -> Result<(), AudioError> {
    let base_path = RESOURCE_PATH.get().ok_or_else(|| AudioError::FileNotFound("Resource path not set".to_string()))?;
    let sound_path = base_path.join("sounds").join(sound.filename());

    if !sound_path.exists() {
        return Err(AudioError::FileNotFound(sound_path.to_string_lossy().to_string()));
    }

    // TODO: use rodio for actual audio playback
    let _ = (&sound_path, sound.filename());
    Ok(())
}

pub fn preload_sounds(resource_path: PathBuf) -> Result<(), AudioError> {
    let _ = RESOURCE_PATH.set(resource_path);
    Ok(())
}

pub fn volume() -> f32 {
    0.7
}

pub fn set_volume(_vol: f32) {
    // TODO: implement volume control
}

#[derive(Debug)]
pub enum AudioError {
    FileNotFound(String),
    Playback(String),
    Load(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::FileNotFound(p) => write!(f, "Sound file not found: {}", p),
            AudioError::Playback(e) => write!(f, "Playback error: {}", e),
            AudioError::Load(e) => write!(f, "Load error: {}", e),
        }
    }
}

impl std::error::Error for AudioError {}

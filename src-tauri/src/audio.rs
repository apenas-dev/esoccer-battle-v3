use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

static RESOURCE_PATH: OnceLock<PathBuf> = OnceLock::new();
static PRELOADED_SOUNDS: OnceLock<Mutex<PreloadedSounds>> = OnceLock::new();

struct PreloadedSounds {
    goal: Option<Vec<u8>>,
    whistle: Option<Vec<u8>>,
}

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
    let sounds = PRELOADED_SOUNDS.get().ok_or_else(|| AudioError::Load("Sounds not preloaded".to_string()))?;
    let data = {
        let guard = sounds.lock().map_err(|e| AudioError::Playback(e.to_string()))?;
        match sound {
            SoundFile::Goal => guard.goal.clone().ok_or_else(|| AudioError::FileNotFound("goal.wav".to_string()))?,
            SoundFile::Whistle => guard.whistle.clone().ok_or_else(|| AudioError::FileNotFound("whistle.wav".to_string()))?,
        }
    };

    let cursor = std::io::Cursor::new(data);
    let decoder = rodio::Decoder::new(cursor)
        .map_err(|e| AudioError::Load(format!("Decode error: {}", e)))?;

    let (_stream, stream_handle) = rodio::OutputStream::try_default()
        .map_err(|e| AudioError::Playback(format!("Failed to create output stream: {}", e)))?;

    let sink = rodio::Sink::try_new(&stream_handle)
        .map_err(|e| AudioError::Playback(format!("Failed to create sink: {}", e)))?;

    sink.append(decoder);
    sink.sleep_until_end();

    Ok(())
}

pub fn preload_sounds(resource_path: PathBuf) -> Result<(), AudioError> {
    let _ = RESOURCE_PATH.set(resource_path.clone());

    let sounds_dir = resource_path.join("sounds");
    let goal = std::fs::read(sounds_dir.join("goal.wav")).ok();
    let whistle = std::fs::read(sounds_dir.join("whistle.wav")).ok();

    let _ = PRELOADED_SOUNDS.set(Mutex::new(PreloadedSounds { goal, whistle }));
    Ok(())
}

pub fn volume() -> f32 {
    0.7
}

pub fn set_volume(_vol: f32) {
    // Volume control is handled at the sink level when playing
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

use std::path::PathBuf;
use std::sync::Mutex;

/// Nomes de sons disponíveis
#[derive(Debug, Clone, Copy)]
pub enum SoundFile {
    Goal,
    Whistle,
    SixMeters,
    Challenge,
}

impl SoundFile {
    pub fn filename(&self) -> &'static str {
        match self {
            SoundFile::Goal => "goal.wav",
            SoundFile::Whistle => "whistle.wav",
            SoundFile::SixMeters => "six_meters.wav",
            SoundFile::Challenge => "challenge.wav",
        }
    }

    /// FIX 8: Map from action name string to SoundFile
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "goal" => Some(Self::Goal),
            "whistle" => Some(Self::Whistle),
            "six_meters" => Some(Self::SixMeters),
            "challenge" => Some(Self::Challenge),
            _ => None,
        }
    }
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
            AudioError::FileNotFound(s) => write!(f, "File not found: {}", s),
            AudioError::Playback(s) => write!(f, "Playback error: {}", s),
            AudioError::Load(s) => write!(f, "Load error: {}", s),
        }
    }
}

static VOLUME: Mutex<f32> = Mutex::new(0.7);
static SOUND_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Reproduz um som em thread separada
pub fn play(sound: SoundFile) -> Result<(), AudioError> {
    let dir_lock = SOUND_DIR.lock().map_err(|e| AudioError::Load(e.to_string()))?;
    let base = dir_lock.as_ref().ok_or_else(|| {
        AudioError::Load("Sounds not preloaded. Call preload_sounds() first.".into())
    })?;

    let path = base.join(sound.filename());
    if !path.exists() {
        return Err(AudioError::FileNotFound(path.display().to_string()));
    }

    let vol = VOLUME.lock().map_err(|e| AudioError::Load(e.to_string()))?.to_owned();

    // Spawn in separate thread to avoid blocking
    std::thread::spawn(move || {
        if let Err(e) = play_blocking(&path, vol) {
            tracing::error!("Audio playback error: {:?}", e);
        }
    });

    Ok(())
}

fn play_blocking(path: &std::path::Path, vol: f32) -> Result<(), AudioError> {
    let file = std::fs::File::open(path).map_err(|e| AudioError::FileNotFound(e.to_string()))?;

    let stream = rodio::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| AudioError::Load(format!("Failed to decode audio: {}", e)))?;

    let (_stream, handle) = rodio::OutputStream::try_default()
        .map_err(|e| AudioError::Playback(format!("No audio output: {}", e)))?;

    let sink = rodio::Sink::try_new(&handle)
        .map_err(|e| AudioError::Playback(format!("Failed to create sink: {}", e)))?;

    sink.set_volume(vol);
    sink.append(stream);
    sink.sleep_until_end();

    Ok(())
}

/// Pré-carrega sons na memória (valida que existem)
pub fn preload_sounds(resource_path: PathBuf) -> Result<(), AudioError> {
    let sounds_dir = resource_path.join("sounds");
    if !sounds_dir.exists() {
        return Err(AudioError::FileNotFound(
            format!("Sounds directory not found: {}", sounds_dir.display()),
        ));
    }

    for sound in &[SoundFile::Goal, SoundFile::Whistle, SoundFile::SixMeters, SoundFile::Challenge] {
        let path = sounds_dir.join(sound.filename());
        if !path.exists() {
            return Err(AudioError::FileNotFound(
                format!("Sound file not found: {}", path.display()),
            ));
        }
    }

    let mut dir_lock = SOUND_DIR.lock().map_err(|e| AudioError::Load(e.to_string()))?;
    *dir_lock = Some(sounds_dir);

    Ok(())
}

pub fn volume() -> f32 {
    VOLUME.lock().map(|v| *v).unwrap_or(0.7)
}

pub fn set_volume(vol: f32) {
    let clamped = vol.clamp(0.0, 1.0);
    if let Ok(mut v) = VOLUME.lock() {
        *v = clamped;
    }
}

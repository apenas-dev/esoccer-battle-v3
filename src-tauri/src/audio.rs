//! Sound playback — game sound effects via rodio.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::LazyLock;

// ── SoundName ────────────────────────────────────────────────────────────

/// Sound effects used during the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundName {
    Goal,
    Whistle,
    SixMeters,
    Challenge,
}

impl SoundName {
    /// Filename relative to the sounds directory.
    pub fn filename(&self) -> &'static str {
        match self {
            Self::Goal => "goal.wav",
            Self::Whistle => "whistle.wav",
            Self::SixMeters => "six_meters.wav",
            Self::Challenge => "challenge.wav",
        }
    }

    /// All sound variants (useful for bulk preload).
    pub fn all() -> &'static [SoundName] {
        &[Self::Goal, Self::Whistle, Self::SixMeters, Self::Challenge]
    }
}

// ── Errors ───────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum AudioError {
    FileNotFound(String),
    Playback(String),
    Load(String),
}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::FileNotFound(msg) => write!(f, "Sound file not found: {msg}"),
            Self::Playback(msg) => write!(f, "Playback error: {msg}"),
            Self::Load(msg) => write!(f, "Load error: {msg}"),
        }
    }
}

impl std::error::Error for AudioError {}

// ── Volume ───────────────────────────────────────────────────────────────

/// Global volume as a percentage (0–100), stored as atomic for lock-free access.
static VOLUME_PERCENT: AtomicU32 = AtomicU32::new(70);

/// Returns the current volume as a float in 0.0..1.0.
pub fn volume() -> f32 {
    VOLUME_PERCENT.load(Ordering::Relaxed) as f32 / 100.0
}

/// Sets the volume. Clamped to 0.0..1.0.
pub fn set_volume(vol: f32) {
    let pct = (vol.clamp(0.0, 1.0) * 100.0) as u32;
    VOLUME_PERCENT.store(pct, Ordering::Relaxed);
}

// ── Audio output (lazily initialized) ────────────────────────────────────

static OUTPUT_HANDLE: LazyLock<Option<rodio::OutputStreamHandle>> = LazyLock::new(|| {
    let (_stream, handle) = match rodio::OutputStream::try_default() {
        Ok(pair) => pair,
        Err(e) => {
            tracing::warn!("Failed to create global audio output stream: {e}");
            return None;
        }
    };
    // `_stream` must outlive all Sinks — intentionally leaked via LazyLock.
    std::mem::forget(_stream);
    Some(handle)
});

// ── Sounds directory ─────────────────────────────────────────────────────

static SOUNDS_DIR: std::sync::Mutex<Option<PathBuf>> = std::sync::Mutex::new(None);

/// Returns the directory that contains sound files.
fn sounds_dir() -> PathBuf {
    if let Ok(guard) = SOUNDS_DIR.lock() {
        if let Some(dir) = guard.as_ref() {
            return dir.clone();
        }
    }

    // Try next to the executable first, then fallback to crate-relative.
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("sounds")));

    if let Some(ref dir) = exe_dir {
        if dir.is_dir() {
            return dir.clone();
        }
    }

    PathBuf::from("sounds")
}

// ── Preload ──────────────────────────────────────────────────────────────

/// Sets the sounds directory (call once at app startup, e.g. with the Tauri
/// resource resolver path).
pub fn set_sounds_dir(dir: PathBuf) {
    if let Ok(mut guard) = SOUNDS_DIR.lock() {
        *guard = Some(dir);
    }
}

/// Preloads all sound files — verifies they exist and are decodable.
/// Call during app initialization to catch missing assets early.
pub fn preload_sounds() -> Result<(), AudioError> {
    let dir = sounds_dir();

    for sound in SoundName::all() {
        let path = dir.join(sound.filename());
        let file = std::fs::File::open(&path).map_err(|e| {
            AudioError::FileNotFound(format!("{path:?}: {e}"))
        })?;
        let _decoder = rodio::Decoder::new(std::io::BufReader::new(file))
            .map_err(|e| AudioError::Load(format!("{path:?}: {e}")))?;
    }

    tracing::info!("All sounds preloaded successfully from {:?}", dir);
    Ok(())
}

// ── Play ─────────────────────────────────────────────────────────────────

/// Plays a sound effect. Non-blocking — spawns a background thread.
/// Never panics; logs warnings on failure.
pub fn play(sound: SoundName) {
    let path = sounds_dir().join(sound.filename());

    std::thread::spawn(move || {
        let handle = match OUTPUT_HANDLE.as_ref() {
            Some(h) => h,
            None => {
                tracing::warn!(
                    "No audio output available — sound {:?} skipped",
                    sound
                );
                return;
            }
        };

        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!("Audio file not found: {:?} ({e})", path);
                return;
            }
        };

        let source = match rodio::Decoder::new(std::io::BufReader::new(file)) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to decode {:?}: {e}", path);
                return;
            }
        };

        let sink = match rodio::Sink::try_new(handle) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to create audio sink: {e}");
                return;
            }
        };

        sink.set_volume(volume());
        sink.append(source);
        sink.sleep_until_end();
    });
}

/// Plays a sound and returns a `Result`. Blocking variant for use in tests
/// or contexts where you need error propagation.
pub fn play_blocking(sound: SoundName) -> Result<(), AudioError> {
    let path = sounds_dir().join(sound.filename());

    let handle = OUTPUT_HANDLE
        .as_ref()
        .ok_or_else(|| AudioError::Playback("No audio output".into()))?;

    let file = std::fs::File::open(&path)
        .map_err(|e| AudioError::FileNotFound(format!("{path:?}: {e}")))?;

    let source = rodio::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| AudioError::Load(format!("{path:?}: {e}")))?;

    let sink =
        rodio::Sink::try_new(handle).map_err(|e| AudioError::Playback(e.to_string()))?;

    sink.set_volume(volume());
    sink.append(source);
    sink.sleep_until_end();
    Ok(())
}

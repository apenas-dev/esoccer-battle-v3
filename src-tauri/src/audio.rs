//! Sound playback — game sound effects via rodio.
//!
//! Architecture:
//! - `init()` must be called once at startup (not LazyLock).
//! - If init fails, audio is disabled but the app keeps running.
//! - `play()` enqueues work on a bounded channel — no unbounded thread spawn.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver};

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

pub type AudioResult<T> = Result<T, AudioError>;

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

// ── Audio output (explicit init) ────────────────────────────────────────

/// Tracks whether init was attempted successfully.
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// The sender side of the work queue. Send sound play requests here.
static SENDER: std::sync::Mutex<Option<std::sync::mpsc::SyncSender<SoundName>>> = std::sync::Mutex::new(None);

/// Maximum concurrent audio tasks in the work queue.
const WORK_QUEUE_CAPACITY: usize = 8;

/// Initialize the audio subsystem. Must be called once at app startup.
///
/// - Creates the rodio output stream.
/// - Spawns a single worker thread with a bounded channel.
/// - If initialization fails, logs the error and marks audio as unavailable.
///   The app keeps running — `play()` calls will be silently skipped.
pub fn init() {
    if INITIALIZED.load(Ordering::Relaxed) {
        eprintln!("[AUDIO] already initialized, skipping");
        return;
    }

    // Verify that audio output is available before spawning worker.
    // OutputStream is not Send, so we create it inside the worker thread.
    if rodio::OutputStream::try_default().is_err() {
        eprintln!("[AUDIO] ERROR: No audio output device available");
        eprintln!("[AUDIO] Audio will be disabled for this session");
        return;
    }

    let (tx, rx): (std::sync::mpsc::SyncSender<SoundName>, Receiver<SoundName>) =
        mpsc::sync_channel(WORK_QUEUE_CAPACITY);

    std::thread::Builder::new()
        .name("audio-worker".into())
        .spawn(move || {
            // Create OutputStream inside this thread — it's not Send so
            // it cannot cross thread boundaries.
            let (_stream, handle) = match rodio::OutputStream::try_default() {
                Ok(pair) => pair,
                Err(e) => {
                    eprintln!("[AUDIO] ERROR: Failed to create output in worker: {e}");
                    return;
                }
            };
            while let Ok(sound) = rx.recv() {
                play_sync(&handle, sound);
            }
            // Channel closed — worker exits gracefully.
        })
        .expect("failed to spawn audio worker thread");

    if let Ok(mut guard) = SENDER.lock() {
        *guard = Some(tx);
    }

    INITIALIZED.store(true, Ordering::Relaxed);
    eprintln!("[AUDIO] initialized successfully (queue capacity={WORK_QUEUE_CAPACITY})");
}

/// Returns true if audio was successfully initialized.
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::Relaxed)
}

/// Synchronous sound playback (runs inside the worker thread).
fn play_sync(handle: &rodio::OutputStreamHandle, sound: SoundName) {
    let path = sounds_dir().join(sound.filename());

    let file = match std::fs::File::open(&path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[AUDIO] WARN: file not found: {:?} ({e})", path);
            return;
        }
    };

    let source = match rodio::Decoder::new(std::io::BufReader::new(file)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[AUDIO] WARN: failed to decode {:?}: {e}", path);
            return;
        }
    };

    let sink = match rodio::Sink::try_new(handle) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[AUDIO] WARN: failed to create sink: {e}");
            return;
        }
    };

    sink.set_volume(volume());
    sink.append(source);
    sink.sleep_until_end();
}

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
pub fn preload_sounds() -> AudioResult<()> {
    let dir = sounds_dir();

    for sound in SoundName::all() {
        let path = dir.join(sound.filename());
        let file = std::fs::File::open(&path).map_err(|e| {
            AudioError::FileNotFound(format!("{path:?}: {e}"))
        })?;
        let _decoder = rodio::Decoder::new(std::io::BufReader::new(file))
            .map_err(|e| AudioError::Load(format!("{path:?}: {e}")))?;
    }

    eprintln!("[AUDIO] all sounds preloaded successfully from {:?}", dir);
    Ok(())
}

// ── Play ─────────────────────────────────────────────────────────────────

/// Plays a sound effect. Non-blocking — enqueues on the bounded work queue.
/// Never panics; logs warnings on failure or if audio is not initialized.
pub fn play(sound: SoundName) {
    if !INITIALIZED.load(Ordering::Relaxed) {
        eprintln!("[AUDIO] play({:?}) skipped — audio not initialized", sound);
        return;
    }

    let sender = match SENDER.lock() {
        Ok(guard) => guard.as_ref().cloned(),
        Err(_) => None,
    };

    let sender = match sender {
        Some(s) => s,
        None => {
            eprintln!("[AUDIO] play({:?}) skipped — sender not available", sound);
            return;
        }
    };

    match sender.try_send(sound) {
        Ok(()) => {
            eprintln!("[AUDIO] enqueued {:?}", sound);
        }
        Err(mpsc::TrySendError::Full(s)) => {
            eprintln!("[AUDIO] WARN: queue full, dropping {:?}", s);
        }
        Err(mpsc::TrySendError::Disconnected(s)) => {
            eprintln!("[AUDIO] WARN: worker disconnected, dropping {:?}", s);
        }
    }
}

/// Plays a sound and returns a `Result`. Blocking variant for use in tests
/// or contexts where you need error propagation.
pub fn play_blocking(sound: SoundName) -> AudioResult<()> {
    let path = sounds_dir().join(sound.filename());

    let _handle = INITIALIZED
        .load(Ordering::Relaxed)
        .then_some(())
        .ok_or_else(|| AudioError::Playback("Audio not initialized".into()))?;

    // We need the actual OutputStreamHandle — but in init() it's moved
    // into the worker thread. This blocking variant is best-effort;
    // for testing, init() should have been called or we create a temp one.
    // For now, redirect to the same path as play() but blocking.
    // Since the handle is inside the worker, we open a separate stream.
    let (stream, handle) = rodio::OutputStream::try_default()
        .map_err(|e| AudioError::Playback(format!("Failed to create output: {e}")))?;

    let file = std::fs::File::open(&path)
        .map_err(|e| AudioError::FileNotFound(format!("{path:?}: {e}")))?;

    let source = rodio::Decoder::new(std::io::BufReader::new(file))
        .map_err(|e| AudioError::Load(format!("{path:?}: {e}")))?;

    let sink =
        rodio::Sink::try_new(&handle).map_err(|e| AudioError::Playback(e.to_string()))?;

    sink.set_volume(volume());
    sink.append(source);
    sink.sleep_until_end();

    // Keep stream alive until playback finishes
    drop(stream);
    eprintln!("[AUDIO] play_blocking({:?}) completed", sound);
    Ok(())
}

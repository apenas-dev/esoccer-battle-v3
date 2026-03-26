use std::path::PathBuf;

/// Sound effects used during the game.
#[derive(Debug, Clone)]
pub enum GameSound {
    Goal,
    WhistleStart,
    WhistleEnd,
    SixMeters,
    Challenge,
}

impl GameSound {
    /// Returns the filename that corresponds to this sound effect.
    fn filename(&self) -> &'static str {
        match self {
            Self::Goal => "goal.wav",
            Self::WhistleStart => "whistle_start.wav",
            Self::WhistleEnd => "whistle_end.wav",
            Self::SixMeters => "six_meters.wav",
            Self::Challenge => "challenge.wav",
        }
    }
}

/// Returns the directory that contains audio assets.
///
/// Looks for `audio/` next to the executable first, then falls back to the
/// crate manifest directory (useful during development).
fn audio_dir() -> PathBuf {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("audio")));

    if let Some(dir) = exe_dir {
        if dir.is_dir() {
            return dir;
        }
    }

    // Fallback: `src-tauri/audio/` relative to the workspace.
    PathBuf::from("audio")
}

/// Play a game sound effect asynchronously.
///
/// If the audio file cannot be found or loaded a warning is logged and the
/// function returns without error — it will **never** panic. Playback happens
/// on a dedicated `std::thread` so the caller is not blocked.
pub fn play_sound(sound: GameSound) {
    let path = audio_dir().join(sound.filename());

    std::thread::spawn(move || {
        let (stream, handle) = match rodio::OutputStream::try_default() {
            Ok(pair) => pair,
            Err(e) => {
                tracing::warn!(
                    "Failed to open audio output stream for {:?}: {e}",
                    sound
                );
                return;
            }
        };

        let file = match std::fs::File::open(&path) {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(
                    "Audio file not found: {:?} ({e}) — sound {:?} will be skipped",
                    path,
                    sound
                );
                return;
            }
        };

        let source = match rodio::Decoder::new(std::io::BufReader::new(file)) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to decode audio file {:?}: {e}", path);
                return;
            }
        };

        let sink = match rodio::Sink::try_new(&handle) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("Failed to create audio sink: {e}");
                return;
            }
        };

        sink.append(source);
        sink.sleep_until_end();

        // `stream` is dropped here after playback completes.
        drop(stream);
    });
}

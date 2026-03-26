#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tauri::{Emitter, Listener, Manager, State};

mod audio;
mod buffer;
mod capture;
mod configuration;
mod game;
mod parser;
mod transcriber;

// ── Shared state types ───────────────────────────────────────────────────

type Settings = Arc<Mutex<configuration::AppSettings>>;
type MatchState = Arc<Mutex<game::MatchState>>;
type VoicePipelineState = Mutex<Option<transcriber::VoicePipeline>>;
type TimerHandle = Mutex<Option<TimerGuard>>;

/// Handle to the background timer thread.
struct TimerGuard {
    shutdown: Arc<AtomicBool>,
    thread_handle: JoinHandle<()>,
}

impl TimerGuard {
    fn stop(self) {
        self.shutdown.store(true, Ordering::Relaxed);
        let _ = self.thread_handle.join();
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn project_directory() -> directories::ProjectDirs {
    directories::ProjectDirs::from("com.esoccer", "ESoccer", "ESoccerBattle")
        .expect("Cannot use app directory")
}

/// Spawn the 1-second timer thread. Returns when `match_state.status != Playing`.
fn spawn_timer(app: tauri::AppHandle, match_state: MatchState) -> TimerGuard {
    use std::time::Instant;

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    let handle = thread::Builder::new()
        .name("match-timer".into())
        .spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                if shutdown_clone.load(Ordering::Relaxed) {
                    break;
                }

                // Measure how long the previous iteration took (lock + emit)
                // and compensate the sleep so ticks stay near 1 s.
                let processing_elapsed = last_tick.elapsed();
                let sleep_duration = Duration::from_secs(1).saturating_sub(processing_elapsed);
                if !sleep_duration.is_zero() {
                    thread::sleep(sleep_duration);
                }
                last_tick = Instant::now();

                let elapsed = {
                    let mut state = match match_state.lock() {
                        Ok(g) => g,
                        Err(e) => {
                            tracing::warn!("[timer] Lock poisoned: {e}");
                            let _ = app.emit("timer_error", format!("Lock poisoned: {e}"));
                            break;
                        }
                    };
                    game::tick(&mut state);
                    state.elapsed_seconds
                };

                if let Err(e) = app.emit("timer_tick", elapsed) {
                    tracing::warn!("[timer] Failed to emit: {e}");
                }

                // Stop ticking if not playing (check after tick since start_match sets Playing).
                let is_playing = match_state
                    .lock()
                    .map(|s| s.status == game::MatchStatus::Playing)
                    .unwrap_or(false);

                if !is_playing {
                    break;
                }
            }
        })
        .expect("Failed to spawn timer thread");

    TimerGuard {
        shutdown,
        thread_handle: handle,
    }
}

fn stop_timer(timer: &TimerHandle) {
    let mut guard = match timer.lock() {
        Ok(g) => g,
        Err(e) => {
            tracing::warn!("Timer lock poisoned: {e}");
            return;
        }
    };
    if let Some(t) = guard.take() {
        t.stop();
    }
}

fn emit_state(app: &tauri::AppHandle, state: &game::MatchState) {
    if let Err(e) = app.emit("match_state_changed", state.clone()) {
        tracing::warn!("Failed to emit match_state_changed: {e}");
    }
}

fn execute_command(
    app: &tauri::AppHandle,
    match_state: &MatchState,
    cmd: parser::GameCommand,
) {
    let state = {
        let mut guard = match match_state.lock() {
            Ok(g) => g,
            Err(e) => {
                tracing::warn!("MatchState lock poisoned: {e}");
                return;
            }
        };
        match cmd {
            parser::GameCommand::StartMatch => game::start_match(&mut guard),
            parser::GameCommand::Restart => game::restart(&mut guard),
            parser::GameCommand::EndMatch => game::end_match(&mut guard),
            parser::GameCommand::Challenge => game::challenge(&mut guard),
            parser::GameCommand::GoalA => game::goal_a(&mut guard),
            parser::GameCommand::GoalB => game::goal_b(&mut guard),
        }
        guard.clone()
    };

    // Play sound
    match cmd {
        parser::GameCommand::GoalA | parser::GameCommand::GoalB => {
            audio::play_sound(audio::GameSound::Goal);
        }
        parser::GameCommand::StartMatch => {
            audio::play_sound(audio::GameSound::WhistleStart);
        }
        parser::GameCommand::EndMatch => {
            audio::play_sound(audio::GameSound::WhistleEnd);
        }
        parser::GameCommand::Restart => {
            audio::play_sound(audio::GameSound::SixMeters);
        }
        parser::GameCommand::Challenge => {
            audio::play_sound(audio::GameSound::Challenge);
        }
    }

    emit_state(app, &state);
}

// ── Tauri commands ───────────────────────────────────────────────────────

#[tauri::command]
fn start_match_cmd(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
    settings: State<'_, Settings>,
) -> Result<(), String> {
    let (team_a, team_b) = {
        let s = settings
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        (s.team_a_name.clone(), s.team_b_name.clone())
    };

    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        *state = game::new_match(&team_a, &team_b);
        game::start_match(&mut state);
    }

    // Stop any existing timer
    stop_timer(&timer);

    // Spawn new timer
    let guard = spawn_timer(app.clone(), match_state.inner().clone());
    {
        let mut t = timer
            .lock()
            .map_err(|e| format!("Timer lock poisoned: {e}"))?;
        *t = Some(guard);
    }

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn end_match_cmd(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::end_match(&mut state);
    }

    stop_timer(&timer);
    audio::play_sound(audio::GameSound::WhistleEnd);

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn goal_a_cmd(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::goal_a(&mut state);
    }
    audio::play_sound(audio::GameSound::Goal);

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn goal_b_cmd(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::goal_b(&mut state);
    }
    audio::play_sound(audio::GameSound::Goal);

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn restart_cmd(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
) -> Result<(), String> {
    stop_timer(&timer);

    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::restart(&mut state);
    }
    audio::play_sound(audio::GameSound::SixMeters);

    // Re-spawn timer since we're still Playing
    let guard = spawn_timer(app.clone(), match_state.inner().clone());
    {
        let mut t = timer
            .lock()
            .map_err(|e| format!("Timer lock poisoned: {e}"))?;
        *t = Some(guard);
    }

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn challenge_cmd(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::challenge(&mut state);
    }
    audio::play_sound(audio::GameSound::Challenge);

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn resolve_challenge_cmd(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::resolve_challenge(&mut state);
    }

    // Stop any existing timer before re-spawning
    stop_timer(&timer);

    // Re-spawn timer since we're back to Playing
    let guard = spawn_timer(app.clone(), match_state.inner().clone());
    {
        let mut t = timer
            .lock()
            .map_err(|e| format!("Timer lock poisoned: {e}"))?;
        *t = Some(guard);
    }

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn get_match_state(match_state: State<'_, MatchState>) -> Result<game::MatchState, String> {
    match_state
        .lock()
        .map(|s| s.clone())
        .map_err(|e| format!("Lock poisoned: {e}"))
}

// ── Voice pipeline commands ──────────────────────────────────────────────

#[tauri::command]
fn start_listening(
    app: tauri::AppHandle,
    pipeline: State<'_, VoicePipelineState>,
    device_name: Option<String>,
    model: transcriber::Model,
) -> Result<(), String> {
    // Stop any existing pipeline first.
    {
        let mut guard = pipeline
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        if guard.is_some() {
            let old = guard.take().ok_or("Failed to take old pipeline")?;
            old.stop().map_err(|e| format!("Failed to stop previous pipeline: {e}"))?;
        }
    }

    let stream = capture::start_capture(device_name)?;
    let audio_buffer = stream.buffer.clone();

    let voice_pipeline = transcriber::VoicePipeline::start(app, audio_buffer, model)?;

    {
        let mut guard = pipeline
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        *guard = Some(voice_pipeline);
    }

    Ok(())
}

#[tauri::command]
fn stop_listening(pipeline: State<'_, VoicePipelineState>) -> Result<(), String> {
    let mut guard = pipeline
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?;
    if let Some(p) = guard.take() {
        p.stop().map_err(|e| format!("Failed to stop pipeline: {e}"))?;
    }
    Ok(())
}

#[tauri::command]
fn list_microphone() -> Vec<capture::DeviceResult> {
    capture::list_microphone()
}

// ── Settings commands ────────────────────────────────────────────────────

#[tauri::command]
fn get_settings(settings: State<'_, Settings>) -> Result<configuration::AppSettings, String> {
    settings
        .lock()
        .map(|s| s.clone())
        .map_err(|e| format!("Lock poisoned: {e}"))
}

#[tauri::command]
fn set_settings(
    settings_state: State<'_, Settings>,
    settings: configuration::AppSettings,
) -> Result<(), String> {
    if let Err(e) = configuration::save(&settings) {
        tracing::warn!("Failed to save settings: {e}");
    }
    settings_state
        .lock()
        .map(|mut s| *s = settings)
        .map_err(|e| format!("Lock poisoned: {e}"))
}

// ── Model management commands ────────────────────────────────────────────

#[tauri::command]
fn download_model(app: tauri::AppHandle, model: transcriber::Model) -> Result<String, String> {
    let channel_name = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Time error: {e}"))?
        .as_nanos()
        .to_string();

    let ch = channel_name.clone();
    tauri::async_runtime::spawn(async move {
        let response = reqwest::get(model.download_url())
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("Download failed: {e}"));

        let mut response = match response {
            Ok(r) => r,
            Err(e) => {
                let _ = app.emit(&ch, serde_json::json!({"type": "error", "value": e}));
                return;
            }
        };

        let file = match std::fs::File::create(model.path()) {
            Ok(f) => f,
            Err(e) => {
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": format!("Create file: {e}")}),
                );
                return;
            }
        };

        let mut file = std::io::BufWriter::new(file);
        let mut downloaded_size = 0u64;
        let predicted_size =
            response.content_length().unwrap_or((model.disk_usage() * 1_000_000) as u64);

        while let Some(chunk) = match response.chunk().await {
            Ok(Some(c)) => Some(c),
            Ok(None) => None,
            Err(e) => {
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": format!("Stream error: {e}")}),
                );
                None
            }
        } {
            if file.write_all(&chunk).is_err() {
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": "Write failed"}),
                );
                return;
            }
            downloaded_size += chunk.len() as u64;

            let percent = (downloaded_size as f32 / predicted_size as f32) * 100.0;
            let _ = app.emit(
                &ch,
                serde_json::json!({
                    "type": "progress",
                    "value": percent.min(100.0),
                }),
            );
        }

        drop(file);

        // Verify file size
        let min_size = (model.disk_usage() * 1_000_000) as u64;
        match std::fs::metadata(model.path()) {
            Ok(meta) if meta.len() >= min_size => {}
            Ok(meta) => {
                let _ = std::fs::remove_file(model.path());
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": format!(
                        "Downloaded file too small: {} bytes (expected >= {})", meta.len(), min_size
                    )}),
                );
                return;
            }
            Err(e) => {
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": format!("Failed to verify file: {e}")}),
                );
                return;
            }
        }

        let _ = app.emit(&ch, serde_json::json!({"type": "done", "value": ""}));
    });

    Ok(channel_name)
}

#[tauri::command]
fn list_models() -> Vec<serde_json::Value> {
    use strum::IntoEnumIterator as _;
    transcriber::Model::iter()
        .map(|model| {
            serde_json::json!({
                "type": model,
                "name": model.name(),
                "mem_usage": model.average_memory_usage(),
                "disk_usage": model.disk_usage(),
                "is_downloaded": model.is_downloaded(),
                "can_run": model.can_run(),
                "category": model.category(),
                "type_name": model.model_type().name(),
            })
        })
        .collect()
}

#[tauri::command]
fn list_model_categories() -> Vec<serde_json::Value> {
    use strum::IntoEnumIterator as _;
    transcriber::Category::iter()
        .map(|c| serde_json::json!({ "type": c, "name": c.name() }))
        .collect()
}

// ── Voice → Game integration (event listener) ────────────────────────────

fn setup_voice_to_game(app: &tauri::AppHandle) {
    let app_handle = app.clone();
    app.listen("voice_text", move |event: tauri::Event| {
        let text = event.payload();

        // Trim JSON string quotes if present (Tauri event payload is a string)
        let text = text
            .strip_prefix('"')
            .and_then(|t| t.strip_suffix('"'))
            .unwrap_or(text)
            .to_string();

        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            return;
        }

        tracing::info!("[voice→game] Received: \"{trimmed}\"");

        // Parse command
        match parser::parse_command(&trimmed) {
            Some(cmd) => {
                tracing::info!("[voice→game] Recognised: {cmd:?}");
                let match_state: MatchState = app_handle.state::<MatchState>().inner().clone();
                execute_command(&app_handle, &match_state, cmd);
            }
            None => {
                tracing::info!("[voice→game] Unknown command: \"{trimmed}\"");
                if let Err(e) = app_handle.emit("command_unknown", &trimmed) {
                    tracing::warn!("Failed to emit command_unknown: {e}");
                }
            }
        }
    });
}

// ── Main ─────────────────────────────────────────────────────────────────

fn main() {
    let settings: Settings =
        Arc::new(Mutex::new(configuration::AppSettings::load_or_default()));

    let match_state: MatchState = Arc::new(Mutex::new(game::new_match("Time A", "Time B")));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(settings)
        .manage(match_state)
        .manage(Mutex::new(None::<transcriber::VoicePipeline>))
        .manage(Mutex::new(None::<TimerGuard>))
        .setup(|app| {
            setup_voice_to_game(&app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_match_cmd,
            end_match_cmd,
            goal_a_cmd,
            goal_b_cmd,
            restart_cmd,
            challenge_cmd,
            resolve_challenge_cmd,
            get_match_state,
            start_listening,
            stop_listening,
            list_microphone,
            get_settings,
            set_settings,
            download_model,
            list_models,
            list_model_categories,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod action_dispatcher;
mod audio;
mod capture;
mod command;
mod config;
mod game;
mod history;
mod match_service;
mod timer;
mod voice_coordinator;

use game::{MatchConfig, MatchState};
use match_service::Action;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use voice_coordinator::VoiceCoordinator;

/// Estado global gerenciado pelo Tauri
struct AppState {
    match_state: Mutex<MatchState>,
    config: Mutex<config::AppConfig>,
    voice: Mutex<VoiceCoordinator>,
    timer: timer::TimerManager,
}

#[tauri::command]
async fn execute_command(
    text: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<serde_json::Value, String> {
    let cmd = command::parse(&text).map_err(|e| e.reason)?;

    // MEDIUM-1 FIX: Single lock scope — read, process, write back atomically.
    // We hold the lock for the entire read-process-write cycle to prevent races.
    let result = {
        let mut state_lock = state.match_state.lock()
            .map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        let current = (*state_lock).clone();

        let result = match_service::process(&current, cmd);

        // Check for timer actions BEFORE dispatch
        let has_start_timer = result.actions.iter().any(|a| matches!(a, Action::StartTimer));
        let has_stop_timer = result.actions.iter().any(|a| matches!(a, Action::StopTimer));

        // Write new state while still holding lock
        *state_lock = result.new_state.clone();

        (result, has_start_timer, has_stop_timer, current.elapsed_secs)
    };

    // Dispatch actions (side effects) outside the lock
    action_dispatcher::dispatch(result.0.actions, &app).map_err(|e| e.to_string())?;

    // CRITICAL-2 FIX: Handle timer start/stop
    if result.2 {
        // Stop timer first
        state.timer.stop();
    }
    if result.1 {
        // Start timer with current elapsed and duration
        let duration_secs = result.0.new_state.config.duration_secs;
        state.timer.start(app, result.3, duration_secs);
    }

    Ok(serde_json::to_value(&result.0.new_state).unwrap_or_default())
}

#[tauri::command]
async fn start_listening(state: State<'_, AppState>) -> Result<(), String> {
    let cfg = state.config.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    let device = cfg.mic_device.clone();
    drop(cfg);

    let mut voice = state.voice.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    voice.start_listening(device).map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_listening(state: State<'_, AppState>, app: AppHandle) -> Result<Option<String>, String> {
    let mut voice = state.voice.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    let transcript = voice.stop_listening(&app).map_err(|e| e.to_string())?;
    Ok(transcript)
}

#[tauri::command]
async fn cancel_listening(state: State<'_, AppState>) -> Result<(), String> {
    let mut voice = state.voice.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    voice.cancel_listening().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<config::AppConfig, String> {
    let cfg = state.config.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    Ok(cfg.clone())
}

#[tauri::command]
async fn update_config(
    new_config: config::AppConfig,
    state: State<'_, AppState>,
) -> Result<(), String> {
    new_config.save().map_err(|e| e.to_string())?;
    audio::set_volume(new_config.volume);

    let mut cfg = state.config.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    *cfg = new_config;
    Ok(())
}

#[tauri::command]
async fn get_state(state: State<'_, AppState>) -> Result<MatchState, String> {
    let ms = state.match_state.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    Ok(ms.clone())
}

#[tauri::command]
async fn get_history(limit: Option<usize>) -> Result<Vec<history::HistoryEntry>, String> {
    history::list(limit).map_err(|e| e.to_string())
}

#[tauri::command]
async fn remove_history(id: String) -> Result<(), String> {
    history::remove(&id).map_err(|e| e.to_string())
}

#[tauri::command]
async fn clear_history() -> Result<(), String> {
    history::clear().map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_mic_devices() -> Result<Vec<String>, String> {
    capture::CaptureStream::list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_available_commands() -> Result<Vec<command::CommandHelp>, String> {
    Ok(command::available_commands())
}

#[tauri::command]
async fn reset_match(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    // MEDIUM-1 FIX: Same single-lock pattern
    let result = {
        let mut state_lock = state.match_state.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        let current = (*state_lock).clone();
        let result = match_service::process(&current, command::GameCommand::Reset);
        *state_lock = result.new_state.clone();
        result
    };
    action_dispatcher::dispatch(result.actions, &app).map_err(|e| e.to_string())?;
    Ok(())
}

fn main() {
    tracing_subscriber::fmt::init();

    let default_config = config::AppConfig::load().unwrap_or_else(|e| {
        tracing::warn!("Failed to load config, using default: {}", e);
        config::AppConfig::default()
    });

    let match_config = MatchConfig {
        team_a_name: default_config.team_a_name.clone(),
        team_b_name: default_config.team_b_name.clone(),
        duration_secs: default_config.match_duration_secs,
        timer_mode: default_config.timer_mode.clone(),
    };

    audio::set_volume(default_config.volume);

    tauri::Builder::default()
        .manage(AppState {
            match_state: Mutex::new(MatchState::new(match_config)),
            config: Mutex::new(default_config),
            voice: Mutex::new(VoiceCoordinator::new()),
            timer: timer::TimerManager::new(),
        })
        .setup(|app| {
            if let Ok(resource_dir) = app.path().resource_dir() {
                if let Err(e) = audio::preload_sounds(resource_dir) {
                    tracing::warn!("Failed to preload sounds: {}", e);
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            execute_command,
            start_listening,
            stop_listening,
            cancel_listening,
            get_config,
            update_config,
            get_state,
            get_history,
            remove_history,
            clear_history,
            list_mic_devices,
            get_available_commands,
            reset_match,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod action_dispatcher;
mod audio;
mod capture;
mod command;
mod config;
mod game;
mod history;
mod match_service;
mod voice_coordinator;

use game::{MatchConfig, MatchState};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use voice_coordinator::VoiceCoordinator;

/// Estado global gerenciado pelo Tauri
struct AppState {
    match_state: Mutex<MatchState>,
    config: Mutex<config::AppConfig>,
    voice: Mutex<VoiceCoordinator>,
}

#[tauri::command]
async fn execute_command(
    text: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<serde_json::Value, String> {
    let cmd = command::parse(&text).map_err(|e| e.reason)?;

    // FIX 5: Single lock scope — clone state, drop lock, process, then re-lock to write
    let result = {
        let state_lock = state.match_state.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        let current = (*state_lock).clone();
        drop(state_lock);
        match_service::process(&current, cmd)
    };
    action_dispatcher::dispatch(result.actions, &app).map_err(|e| e.to_string())?;

    let mut state_lock = state.match_state.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    *state_lock = result.new_state.clone();

    Ok(serde_json::to_value(&*state_lock).unwrap_or_default())
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
async fn stop_listening(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let mut voice = state.voice.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    voice.stop_listening(&app).map_err(|e| e.to_string())
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
async fn list_mic_devices() -> Result<Vec<String>, String> {
    capture::CaptureStream::list_devices().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_available_commands() -> Result<Vec<command::CommandHelp>, String> {
    Ok(command::available_commands())
}

#[tauri::command]
async fn reset_match(state: State<'_, AppState>, app: AppHandle) -> Result<(), String> {
    let current = state.match_state.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?.clone();

    let result = match_service::process(&current, command::GameCommand::Reset);
    action_dispatcher::dispatch(result.actions, &app).map_err(|e| e.to_string())?;

    let mut state_lock = state.match_state.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
    *state_lock = result.new_state;
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
            get_config,
            update_config,
            get_state,
            get_history,
            list_mic_devices,
            get_available_commands,
            reset_match,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod game;
mod command;
mod match_service;
mod action_dispatcher;
mod voice_coordinator;
mod capture;
mod audio;
mod history;
mod config;

use tauri::{AppHandle, Manager, State};
use std::sync::Mutex;
use game::MatchState;
use config::AppConfig;

struct AppState {
    match_state: Mutex<MatchState>,
    config: Mutex<AppConfig>,
    voice_coordinator: Mutex<voice_coordinator::VoiceCoordinator>,
}

#[tauri::command]
async fn execute_command(
    text: String,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<serde_json::Value, String> {
    let cmd = command::parse(&text).map_err(|e| e.reason)?;

    let now = chrono::Utc::now().timestamp() as u64;

    // #4: Clone state, process, then update in separate lock scopes
    // but process() is pure — no external side effects between lock releases
    let current = {
        let guard = state.match_state.lock().map_err(|e| e.to_string())?;
        guard.clone()
    };

    let result = match_service::process(&current, cmd, now);

    action_dispatcher::dispatch(result.actions, &app).await
        .map_err(|e| format!("{:?}", e))?;

    {
        let mut guard = state.match_state.lock().map_err(|e| e.to_string())?;
        *guard = result.new_state.clone();
    }

    serde_json::to_value(&result.new_state).map_err(|e| e.to_string())
}

#[tauri::command]
async fn start_listening(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let config = {
        let cfg = state.config.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?;
        Some(capture::CaptureConfig {
            device_name: cfg.mic_device.clone(),
            sample_rate: 16000,
            channels: 1,
        })
    };
    // VoiceCoordinator methods are sync (no actual await needed for cpal stream start)
    state.voice_coordinator.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?
        .start_listening(&app, config)
        .map_err(|e| format!("{:?}", e))
}

#[tauri::command]
async fn stop_listening(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<serde_json::Value, String> {
    let buffer = state.voice_coordinator.lock().map_err(|e: std::sync::PoisonError<_>| e.to_string())?
        .stop_listening(&app)
        .map_err(|e| format!("{:?}", e))?;
    // Return buffer info; transcription happens on frontend
    Ok(serde_json::json!({
        "sample_count": buffer.samples.len(),
        "sample_rate": buffer.sample_rate,
        "duration_secs": if buffer.sample_rate > 0 { buffer.samples.len() as f64 / buffer.sample_rate as f64 } else { 0.0 },
        "transcript": "" // placeholder — frontend will transcribe
    }))
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
async fn update_config(new_config: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    new_config.save().map_err(|e| format!("{:?}", e))?;
    let mut guard = state.config.lock().map_err(|e| e.to_string())?;
    *guard = new_config;
    Ok(())
}

#[tauri::command]
async fn get_state(state: State<'_, AppState>) -> Result<MatchState, String> {
    let match_state = state.match_state.lock().map_err(|e| e.to_string())?;
    Ok(match_state.clone())
}

#[tauri::command]
async fn get_history(limit: Option<usize>) -> Result<Vec<history::HistoryEntry>, String> {
    history::list(limit).await.map_err(|e| format!("{:?}", e))
}

#[tauri::command]
async fn list_mic_devices() -> Result<Vec<String>, String> {
    capture::CaptureStream::list_devices().map_err(|e| format!("{:?}", e))
}

#[tauri::command]
async fn get_available_commands() -> Result<Vec<command::CommandHelp>, String> {
    Ok(command::available_commands())
}

fn main() {
    let (tx, _) = std::sync::mpsc::channel();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState {
            match_state: Mutex::new(MatchState::new(AppConfig::default().into())),
            config: Mutex::new(AppConfig::default()),
            voice_coordinator: Mutex::new(voice_coordinator::VoiceCoordinator::new(tx)),
        })
        .setup(|app| {
            if let Some(resource_dir) = app.path().resource_dir().ok() {
                audio::preload_sounds(resource_dir).ok();
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            execute_command, start_listening, stop_listening,
            get_config, update_config, get_state, get_history,
            list_mic_devices, get_available_commands,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

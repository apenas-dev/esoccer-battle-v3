#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write as _;
use tauri::Emitter as _;

mod configuration;
mod recorder;
mod transcriber;

type GeneralConfig = std::sync::Arc<std::sync::Mutex<configuration::GeneralConfig>>;

fn project_directory() -> directories::ProjectDirs {
    directories::ProjectDirs::from("com.esoccer", "ESoccer", "ESoccerBattle")
        .expect("Cannot use app directory")
}

#[tauri::command]
fn list_microphone() -> Vec<recorder::DeviceResult> {
    recorder::list_microphone()
}

#[tauri::command]
fn get_general_config(general_config: tauri::State<'_, GeneralConfig>) -> configuration::GeneralConfig {
    general_config.lock().unwrap().clone()
}

#[tauri::command]
fn set_general_config(general_config_state: tauri::State<'_, GeneralConfig>, general_config: configuration::GeneralConfig) {
    configuration::save(&general_config);
    *general_config_state.lock().unwrap() = general_config;
}

#[tauri::command]
fn download_model(app: tauri::AppHandle, model: transcriber::Model) -> String {
    let channel_name = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
        .to_string();

    let ch = channel_name.clone();
    tauri::async_runtime::spawn(async move {
        let mut response = reqwest::get(model.download_url()).await.unwrap();
        let mut file = std::fs::File::create(model.path()).unwrap();

        let mut downloaded_size = 0u64;
        let predicted_size = response.content_length().unwrap_or((model.disk_usage() * 1_000_000) as u64);

        while let Some(chunk) = response.chunk().await.unwrap() {
            let length = chunk.len();
            file.write_all(&chunk).unwrap();
            downloaded_size += length as u64;

            app.emit(&ch, serde_json::json!({
                "type": "progress",
                "value": (downloaded_size as f32 / predicted_size as f32) * 100.0,
            }))
            .unwrap();
        }

        app.emit(&ch, serde_json::json!({
            "type": "done",
            "value": "",
        }))
        .unwrap();
    });

    channel_name
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
                "type_name": model.r#type().name(),
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

fn main() {
    let general_config: GeneralConfig =
        std::sync::Arc::new(std::sync::Mutex::new(configuration::GeneralConfig::default()));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(general_config)
        .invoke_handler(tauri::generate_handler![
            list_microphone,
            get_general_config,
            set_general_config,
            download_model,
            list_models,
            list_model_categories,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

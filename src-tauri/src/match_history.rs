use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchRecord {
    pub id: String,
    pub team_a_name: String,
    pub team_b_name: String,
    pub score_a: u32,
    pub score_b: u32,
    pub duration_secs: u32,
    pub finished_at: String,
}

fn history_path() -> PathBuf {
    let dir = crate::project_directory().cache_dir().to_path_buf();
    let _ = fs::create_dir_all(&dir);
    dir.join("history.json")
}

pub fn save_match(record: &MatchRecord) -> Result<(), String> {
    let mut history = load_history().unwrap_or_default();
    history.push(record.clone());
    let json = serde_json::to_string_pretty(&history).map_err(|e| format!("Serialize: {e}"))?;
    fs::write(history_path(), json).map_err(|e| format!("Write: {e}"))
}

pub fn load_history() -> Result<Vec<MatchRecord>, String> {
    let path = history_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let data = fs::read_to_string(&path).map_err(|e| format!("Read: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("Deserialize: {e}"))
}

pub fn clear_history() -> Result<(), String> {
    let path = history_path();
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Remove: {e}"))?;
    }
    Ok(())
}

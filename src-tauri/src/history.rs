use crate::match_service::MatchSnapshot;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: String,
    pub match_id: String,
    pub team_a_name: String,
    pub team_b_name: String,
    pub score_a: u32,
    pub score_b: u32,
    pub duration_secs: u64,
    pub finished_at: String,
}

fn history_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("esoccer-battle")
        .join("history.json")
}

fn ensure_dir() -> Result<(), HistoryError> {
    let path = history_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| HistoryError::Io(e.to_string()))?;
    }
    Ok(())
}

fn read_entries() -> Result<Vec<HistoryEntry>, HistoryError> {
    let path = history_path();
    if !path.exists() {
        return Ok(vec![]);
    }
    let contents = std::fs::read_to_string(&path).map_err(|e| HistoryError::Io(e.to_string()))?;
    if contents.trim().is_empty() {
        return Ok(vec![]);
    }
    serde_json::from_str(&contents).map_err(|e| HistoryError::Parse(e.to_string()))
}

fn write_entries(entries: &[HistoryEntry]) -> Result<(), HistoryError> {
    ensure_dir()?;
    let path = history_path();
    let tmp_path = path.with_extension("json.tmp");
    let contents = serde_json::to_string_pretty(entries).map_err(|e| HistoryError::Parse(e.to_string()))?;
    std::fs::write(&tmp_path, &contents).map_err(|e| HistoryError::Io(e.to_string()))?;
    std::fs::rename(&tmp_path, &path).map_err(|e| HistoryError::Io(e.to_string()))?;
    Ok(())
}

pub async fn save(snapshot: MatchSnapshot) -> Result<(), HistoryError> {
    let entry = HistoryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        match_id: snapshot.match_id,
        team_a_name: snapshot.team_a_name,
        team_b_name: snapshot.team_b_name,
        score_a: snapshot.score_a,
        score_b: snapshot.score_b,
        duration_secs: snapshot.duration_secs,
        finished_at: snapshot.finished_at,
    };

    let mut entries = read_entries()?;
    entries.push(entry);
    write_entries(&entries)?;
    Ok(())
}

pub async fn list(limit: Option<usize>) -> Result<Vec<HistoryEntry>, HistoryError> {
    let mut entries = read_entries()?;
    entries.reverse(); // Most recent first
    if let Some(n) = limit {
        entries.truncate(n);
    }
    Ok(entries)
}

pub async fn remove(id: &str) -> Result<(), HistoryError> {
    let mut entries = read_entries()?;
    entries.retain(|e| e.id != id);
    write_entries(&entries)?;
    Ok(())
}

pub async fn clear() -> Result<(), HistoryError> {
    write_entries(&[])?;
    Ok(())
}

#[derive(Debug)]
pub enum HistoryError {
    Io(String),
    Parse(String),
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryError::Io(e) => write!(f, "IO error: {}", e),
            HistoryError::Parse(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for HistoryError {}

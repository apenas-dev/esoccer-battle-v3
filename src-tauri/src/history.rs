use crate::match_service::MatchSnapshot;
use serde::{Deserialize, Serialize};
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

#[derive(Debug)]
pub enum HistoryError {
    Io(String),
    Parse(String),
}

/// Salva resultado de partida no histórico
pub fn save(snapshot: MatchSnapshot) -> Result<(), HistoryError> {
    let mut entries = load_entries()?;

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

    entries.push(entry);
    save_entries(&entries)
}

/// Lista todas as partidas (mais recente primeiro)
pub fn list(limit: Option<usize>) -> Result<Vec<HistoryEntry>, HistoryError> {
    let mut entries = load_entries()?;
    entries.reverse(); // most recent first
    match limit {
        Some(n) => Ok(entries.into_iter().take(n).collect()),
        None => Ok(entries),
    }
}

/// Remove partida do histórico por ID
pub fn remove(id: &str) -> Result<(), HistoryError> {
    let mut entries = load_entries()?;
    entries.retain(|e| e.id != id);
    save_entries(&entries)
}

/// Limpa todo o histórico
pub fn clear() -> Result<(), HistoryError> {
    save_entries(&[])
}

fn history_path() -> PathBuf {
    let base = directories::ProjectDirs::from("com", "esoccer", "battle")
        .map(|d| d.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    base.join("history.json")
}

fn load_entries() -> Result<Vec<HistoryEntry>, HistoryError> {
    let path = history_path();

    if !path.exists() {
        return Ok(Vec::new());
    }

    let content = std::fs::read_to_string(&path).map_err(|e| HistoryError::Io(e.to_string()))?;
    serde_json::from_str(&content).map_err(|e| HistoryError::Parse(e.to_string()))
}

fn save_entries(entries: &[HistoryEntry]) -> Result<(), HistoryError> {
    let path = history_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| HistoryError::Io(e.to_string()))?;
    }

    let content = serde_json::to_string_pretty(entries).map_err(|e| HistoryError::Parse(e.to_string()))?;
    std::fs::write(&path, content).map_err(|e| HistoryError::Io(e.to_string()))
}

impl std::fmt::Display for HistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HistoryError::Io(s) => write!(f, "IO error: {}", s),
            HistoryError::Parse(s) => write!(f, "Parse error: {}", s),
        }
    }
}

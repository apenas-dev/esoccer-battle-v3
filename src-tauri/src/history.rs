use crate::match_service::MatchSnapshot;
use serde::{Serialize, Deserialize};

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

pub async fn save(snapshot: MatchSnapshot) -> Result<(), HistoryError> {
    let _entry = HistoryEntry {
        id: uuid::Uuid::new_v4().to_string(),
        match_id: snapshot.match_id,
        team_a_name: snapshot.team_a_name,
        team_b_name: snapshot.team_b_name,
        score_a: snapshot.score_a,
        score_b: snapshot.score_b,
        duration_secs: snapshot.duration_secs,
        finished_at: snapshot.finished_at,
    };
    // TODO: implement file persistence
    let _ = _entry;
    Ok(())
}

pub async fn list(_limit: Option<usize>) -> Result<Vec<HistoryEntry>, HistoryError> {
    Ok(vec![])
}

pub async fn remove(_id: &str) -> Result<(), HistoryError> {
    Ok(())
}

pub async fn clear() -> Result<(), HistoryError> {
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

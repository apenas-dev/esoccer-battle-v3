//! Game engine: match state machine and transitions.

use serde::{Deserialize, Serialize};

// ── Types ────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MatchStatus {
    Idle,
    Playing,
    Challenge,
    Finished,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchState {
    pub status: MatchStatus,
    pub score_a: u32,
    pub score_b: u32,
    pub elapsed_seconds: u64,
    pub team_a_name: String,
    pub team_b_name: String,
    pub last_command: Option<String>,
}

// ── State transitions ────────────────────────────────────────────────────

pub fn new_match(team_a: &str, team_b: &str) -> MatchState {
    MatchState {
        status: MatchStatus::Idle,
        score_a: 0,
        score_b: 0,
        elapsed_seconds: 0,
        team_a_name: team_a.to_owned(),
        team_b_name: team_b.to_owned(),
        last_command: None,
    }
}

pub fn start_match(state: &mut MatchState) {
    if state.status != MatchStatus::Idle {
        return;
    }
    state.status = MatchStatus::Playing;
    state.score_a = 0;
    state.score_b = 0;
    state.elapsed_seconds = 0;
    state.last_command = Some("start_match".to_owned());
}

pub fn goal_a(state: &mut MatchState) {
    if state.status != MatchStatus::Playing {
        return;
    }
    state.score_a += 1;
    state.last_command = Some("goal_a".to_owned());
}

pub fn goal_b(state: &mut MatchState) {
    if state.status != MatchStatus::Playing {
        return;
    }
    state.score_b += 1;
    state.last_command = Some("goal_b".to_owned());
}

pub fn restart(state: &mut MatchState) {
    if state.status != MatchStatus::Playing {
        return;
    }
    state.elapsed_seconds = 0;
    state.last_command = Some("restart".to_owned());
}

pub fn challenge(state: &mut MatchState) {
    if state.status != MatchStatus::Playing {
        return;
    }
    state.status = MatchStatus::Challenge;
    state.last_command = Some("challenge".to_owned());
}

pub fn resolve_challenge(state: &mut MatchState) {
    if state.status != MatchStatus::Challenge {
        return;
    }
    state.status = MatchStatus::Playing;
    state.last_command = Some("resolve_challenge".to_owned());
}

pub fn end_match(state: &mut MatchState) {
    if state.status == MatchStatus::Finished {
        return;
    }
    state.status = MatchStatus::Finished;
    state.last_command = Some("end_match".to_owned());
}

pub fn tick(state: &mut MatchState) {
    if state.status == MatchStatus::Playing {
        state.elapsed_seconds += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle() {
        let mut s = new_match("A", "B");
        assert_eq!(s.status, MatchStatus::Idle);

        start_match(&mut s);
        assert_eq!(s.status, MatchStatus::Playing);
        assert_eq!(s.score_a, 0);

        goal_a(&mut s);
        assert_eq!(s.score_a, 1);

        goal_b(&mut s);
        assert_eq!(s.score_b, 1);

        tick(&mut s);
        assert_eq!(s.elapsed_seconds, 1);

        challenge(&mut s);
        assert_eq!(s.status, MatchStatus::Challenge);

        resolve_challenge(&mut s);
        assert_eq!(s.status, MatchStatus::Playing);

        restart(&mut s);
        assert_eq!(s.elapsed_seconds, 0);
        assert_eq!(s.score_a, 1); // scores preserved

        end_match(&mut s);
        assert_eq!(s.status, MatchStatus::Finished);

        // tick should NOT increment when not playing
        tick(&mut s);
        assert_eq!(s.elapsed_seconds, 0);
    }
}

use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use strsim::levenshtein;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum GameCommand {
    StartMatch,
    Restart,
    EndMatch,
    Challenge,
    GoalA,
    GoalB,
}

struct AliasEntry {
    command: GameCommand,
    alias: &'static str,
}

/// Alias table built once via `LazyLock`.
static ALIAS_TABLE: LazyLock<Vec<AliasEntry>> = LazyLock::new(|| {
    const ENTRIES: &[(&str, &[&str])] = &[
        ("StartMatch", &["iniciar partida", "comecar", "iniciar", "play", "começar"]),
        ("Restart", &["volta seis", "6", "volta", "seis metros", "volta 6"]),
        ("EndMatch", &["encerrar", "fim", "parar", "stop", "acabar"]),
        ("Challenge", &["dúvida", "duvida", "contestar", "protestar"]),
        // "gol" alone is ambiguous — require explicit team discriminator.
        ("GoalA", &["gol do time a", "gol time a", "ponto a", "gol pra mim"]),
        ("GoalB", &["gol do time b", "gol time b", "ponto b", "gol pra eles"]),
    ];

    let mut table: Vec<AliasEntry> = Vec::new();
    for &(variant, aliases) in ENTRIES {
        let command = match variant {
            "StartMatch" => GameCommand::StartMatch,
            "Restart" => GameCommand::Restart,
            "EndMatch" => GameCommand::EndMatch,
            "Challenge" => GameCommand::Challenge,
            "GoalA" => GameCommand::GoalA,
            "GoalB" => GameCommand::GoalB,
            _ => continue,
        };
        for &alias in aliases {
            table.push(AliasEntry {
                command: command.clone(),
                alias,
            });
        }
    }
    table
});

fn normalize(text: &str) -> String {
    text.nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .flat_map(|c| c.to_lowercase())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string()
}

pub fn parse_command(text: &str) -> Option<GameCommand> {
    let normalized = normalize(text);
    if normalized.is_empty() {
        return None;
    }

    // If input looks like a goal command, only allow GoalA/GoalB via substring match.
    let goal_prefixes = ["gol", "ponto"];
    let is_goal_input = goal_prefixes
        .iter()
        .any(|p| normalized.starts_with(p) || normalized.contains(&format!(" {p} ")));

    let mut best: Option<(GameCommand, usize)> = None;

    for entry in ALIAS_TABLE.iter() {
        let alias_norm = normalize(entry.alias);
        let dist = levenshtein(&normalized, &alias_norm);

        // Substring match: the full alias must appear as a word-aligned substring.
        let is_substring = is_word_substring(&normalized, &alias_norm);

        if is_substring {
            return Some(entry.command.clone());
        }

        // Goal commands require explicit team discriminators — no fuzzy matching.
        let is_goal = matches!(
            entry.command,
            GameCommand::GoalA | GameCommand::GoalB
        );
        if is_goal {
            continue;
        }

        // If input looks like a goal command, skip fuzzy matching for non-goal aliases.
        if is_goal_input {
            continue;
        }

        // Threshold: for very short aliases, use stricter matching
        let max_dist = if alias_norm.len() <= 1 { 1 } else { 2 };

        if dist <= max_dist {
            match &best {
                None => best = Some((entry.command.clone(), dist)),
                Some((_, best_dist)) if dist < *best_dist => {
                    best = Some((entry.command.clone(), dist));
                }
                Some(_) => {}
            }
        }
    }

    best.map(|(cmd, _)| cmd)
}

/// Check that `needle` appears in `haystack` as a complete phrase,
/// bounded by start/end or whitespace.
fn is_word_substring(haystack: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    // Quick containment check first
    if !haystack.contains(needle) {
        return false;
    }
    for range in haystack.match_indices(needle) {
        let start = range.0;
        let end = start + needle.len();
        let ok_before = start == 0 || haystack.as_bytes()[start - 1] == b' ';
        let ok_after = end == haystack.len() || haystack.as_bytes()[end] == b' ';
        if ok_before && ok_after {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_alias_match() {
        assert_eq!(parse_command("iniciar partida"), Some(GameCommand::StartMatch));
        assert_eq!(parse_command("fim"), Some(GameCommand::EndMatch));
        assert_eq!(parse_command("stop"), Some(GameCommand::EndMatch));
        assert_eq!(parse_command("duvida"), Some(GameCommand::Challenge));
        assert_eq!(parse_command("volta seis"), Some(GameCommand::Restart));
        // "gol a" and "gol b" are no longer aliases
        assert_eq!(parse_command("gol a"), None);
        assert_eq!(parse_command("gol b"), None);
        // Explicit team required
        assert_eq!(parse_command("gol time a"), Some(GameCommand::GoalA));
        assert_eq!(parse_command("gol time b"), Some(GameCommand::GoalB));
        assert_eq!(parse_command("gol do time a"), Some(GameCommand::GoalA));
        assert_eq!(parse_command("gol do time b"), Some(GameCommand::GoalB));
    }

    #[test]
    fn test_gol_alone_returns_none() {
        assert_eq!(parse_command("gol"), None);
    }

    #[test]
    fn test_accent_insensitive() {
        assert_eq!(parse_command("começar"), Some(GameCommand::StartMatch));
        assert_eq!(parse_command("comecar"), Some(GameCommand::StartMatch));
        assert_eq!(parse_command("DÚVIDA"), Some(GameCommand::Challenge));
        assert_eq!(parse_command("dúvida"), Some(GameCommand::Challenge));
    }

    #[test]
    fn test_case_insensitive() {
        assert_eq!(parse_command("INICIAR PARTIDA"), Some(GameCommand::StartMatch));
        assert_eq!(parse_command("Play"), Some(GameCommand::StartMatch));
        assert_eq!(parse_command("STOP"), Some(GameCommand::EndMatch));
        assert_eq!(parse_command("Fim"), Some(GameCommand::EndMatch));
    }

    #[test]
    fn test_fuzzy_match_one_edit() {
        assert_eq!(parse_command("inicar"), Some(GameCommand::StartMatch));
        assert_eq!(parse_command("parra"), Some(GameCommand::EndMatch));
    }

    #[test]
    fn test_fuzzy_match_two_edits() {
        assert_eq!(parse_command("encerrr"), Some(GameCommand::EndMatch));
    }

    #[test]
    fn test_no_match() {
        assert_eq!(parse_command("hello world"), None);
        assert_eq!(parse_command("abacaxi"), None);
        assert_eq!(parse_command(""), None);
        assert_eq!(parse_command("xxx"), None);
    }

    #[test]
    fn test_full_sentence_substring_match() {
        assert_eq!(
            parse_command("eu quero gol do time a por favor"),
            Some(GameCommand::GoalA)
        );
        // "gol a" is no longer an alias, so this should NOT match
        assert_eq!(parse_command("fala gol a rapaz"), None);
        assert_eq!(
            parse_command("aí foi gol pra mim"),
            Some(GameCommand::GoalA)
        );
        assert_eq!(
            parse_command("marcou gol do time b"),
            Some(GameCommand::GoalB)
        );
        assert_eq!(
            parse_command("faz o favor de encerrar a partida"),
            Some(GameCommand::EndMatch)
        );
        assert_eq!(
            parse_command("preciso contestar esse lance"),
            Some(GameCommand::Challenge)
        );
        assert_eq!(
            parse_command("eita vou protestar"),
            Some(GameCommand::Challenge)
        );
        assert_eq!(
            parse_command("volta seis pra ficar igual"),
            Some(GameCommand::Restart)
        );
    }

    #[test]
    fn test_word_boundary_substring() {
        // "gol" alone should not match "gol do time a"
        // because we require word-aligned boundaries for the FULL alias
        assert_eq!(parse_command("gol"), None);
        // "gol time a" is a valid alias
        assert_eq!(parse_command("fala gol time a rapaz"), Some(GameCommand::GoalA));
    }
}

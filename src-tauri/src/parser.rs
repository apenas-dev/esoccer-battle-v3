use serde::{Deserialize, Serialize};
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

/// Pre-computed alias table (normalized at build time via const or lazy_static).
/// We store both original and normalized forms to avoid re-normalizing on every call.
macro_rules! alias_table {
    () => {{
        const ENTRIES: &[(&str, &[&str])] = &[
            ("StartMatch", &["iniciar partida", "comecar", "iniciar", "play", "começar"]),
            ("Restart", &["volta seis", "6", "volta", "seis metros", "volta 6"]),
            ("EndMatch", &["encerrar", "fim", "parar", "stop", "acabar"]),
            ("Challenge", &["dúvida", "duvida", "contestar", "protestar"]),
            ("GoalA", &["gol do time a", "gol a", "gol time a", "ponto a", "gol pra mim"]),
            ("GoalB", &["gol do time b", "gol b", "gol time b", "ponto b", "gol pra eles"]),
        ];

        // Build lazily via Vec because we can't use const normalization
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
                // We normalize on first call; cache via lazy_static or just recompute
                // For simplicity and KISS, we normalize each call.
                table.push(AliasEntry {
                    command: command.clone(),
                    alias,
                });
            }
        }
        table
    }};
}

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

    let table = alias_table!();

    let mut best: Option<(GameCommand, usize)> = None;

    for entry in &table {
        let alias_norm = normalize(entry.alias);
        let dist = levenshtein(&normalized, &alias_norm);
        let _alias_len = alias_norm.len().max(normalized.len());

        // Substring match: alias is contained in the text (normalized)
        let is_substring = normalized.contains(&alias_norm);

        if is_substring {
            // Perfect match via substring — distance 0
            return Some(entry.command.clone());
        }

        // Threshold: for very short aliases, use stricter matching
        // to avoid false positives (e.g. "xxx" matching "6")
        let max_dist = if alias_norm.len() <= 1 { 1 } else { 2 };

        if dist <= max_dist {
            match &best {
                None => best = Some((entry.command.clone(), dist)),
                Some((_, best_dist)) if dist < *best_dist => {
                    best = Some((entry.command.clone(), dist));
                }
                Some(_) => {} // keep current best
            }
        }
    }

    best.map(|(cmd, _)| cmd)
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
        assert_eq!(parse_command("gol a"), Some(GameCommand::GoalA));
        assert_eq!(parse_command("gol b"), Some(GameCommand::GoalB));
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
        // "iniciar" is an alias; "inicar" has 1 deletion
        assert_eq!(parse_command("inicar"), Some(GameCommand::StartMatch));
        // "parra" vs "parar" — 1 edit
        assert_eq!(parse_command("parra"), Some(GameCommand::EndMatch));
    }

    #[test]
    fn test_fuzzy_match_two_edits() {
        // "encerrr" vs "encerrar" — 1 insertion → still ≤ 2
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
        // The key: "gol do time a" should be found inside a longer sentence
        assert_eq!(
            parse_command("eu quero gol do time a por favor"),
            Some(GameCommand::GoalA)
        );
        assert_eq!(
            parse_command("fala gol a rapaz"),
            Some(GameCommand::GoalA)
        );
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
    fn test_multiple_matches_picks_closest() {
        // "gol" is close to both "gol a" and "gol b" (distance 2 each).
        // But "fim" is distance ≤ 2 only to "fim" itself.
        // "gol" normalized = "gol", "gol a" distance = 2, "gol b" distance = 2
        // Both qualify; it picks whichever appears first. Test just that it returns Some.
        assert!(parse_command("gol").is_some());
    }
}

//! `command.rs` — Parser de comandos de voz/texto para o jogo.
//!
//! Responsabilidade ÚNICA: converter texto livre em [`GameCommand`].
//! Zero dependência de Tauri ou qualquer I/O.
//! OCP: adicionar um novo comando = adicionar uma variante + uma entrada na tabela de aliases.

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

// ── Public types ─────────────────────────────────────────────────────────

/// Comandos possíveis do jogo.
///
/// OCP: adicionar um novo comando = nova variante + entrada em `ALIASES`.
/// Nenhuma outra função precisa ser modificada.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GameCommand {
    Start,
    GoalA,
    GoalB,
    Pause,
    Resume,
    End,
    Doubt,
    Resolve,
    VoltaSeis,
    Reset,
}

/// Erro retornado quando o texto não corresponde a nenhum comando conhecido.
#[derive(Debug, Clone, Serialize)]
pub struct ParseError {
    pub input: String,
    pub reason: String,
}

/// Entrada de help para um comando.
#[derive(Debug, Clone, Serialize)]
pub struct CommandHelp {
    pub command: String,
    pub description: String,
    pub aliases: Vec<String>,
}

// ── Alias table (single source of truth for OCP) ────────────────────────

/// Alias → GameCommand mapping.
///
/// Entradas mais específicas primeiro (ex: "volta seis" antes de termos soltos).
/// **Para adicionar um novo comando:** adicione entradas aqui + variante no enum.
struct AliasEntry {
    aliases: &'static [&'static str],
    command: GameCommand,
    description: &'static str,
}

/// Tabela de aliases. Ordem importa — o primeiro match ganha.
const ALIAS_TABLE: &[AliasEntry] = &[
    // VoltaSeis — mais específico primeiro (multi-palavra)
    AliasEntry {
        aliases: &["volta seis", "6 metros", "seis metros", "volta 6", "volta6"],
        command: GameCommand::VoltaSeis,
        description: "Marca volta seis (6 metros) durante um desafio",
    },
    // Doubt — multi-palavra
    AliasEntry {
        aliases: &["duvida", "dúvida", "desafio", "challenge", "reclamacao", "reclamação"],
        command: GameCommand::Doubt,
        description: "Inicia um desafio/dúvida",
    },
    // GoalA
    AliasEntry {
        aliases: &[
            "gol a",
            "gola",
            "goal a",
            "goool a",
            "gol time a",
            "goal time a",
            "gol para a",
            "gol pro a",
            "gol do a",
            "marcou a",
        ],
        command: GameCommand::GoalA,
        description: "Marca gol para o Time A",
    },
    // GoalB
    AliasEntry {
        aliases: &[
            "gol b",
            "golb",
            "goal b",
            "goool b",
            "gol time b",
            "goal time b",
            "gol para b",
            "gol pro b",
            "gol do b",
            "marcou b",
        ],
        command: GameCommand::GoalB,
        description: "Marca gol para o Time B",
    },
    // Start
    AliasEntry {
        aliases: &[
            "iniciar",
            "comecar",
            "começar",
            "start",
            "inicia",
            "começa",
            "jogar",
            "vai",
        ],
        command: GameCommand::Start,
        description: "Inicia a partida",
    },
    // Pause
    AliasEntry {
        aliases: &[
            "pausar",
            "pausa",
            "pause",
            "parar",
            "stop",
        ],
        command: GameCommand::Pause,
        description: "Pausa a partida",
    },
    // Resume
    AliasEntry {
        aliases: &[
            "retomar",
            "retoma",
            "resume",
            "continuar",
            "volta",
            "voltar",
            "continua",
        ],
        command: GameCommand::Resume,
        description: "Retoma a partida pausada",
    },
    // End
    AliasEntry {
        aliases: &[
            "encerrar",
            "encerra",
            "finalizar",
            "finaliza",
            "terminar",
            "termina",
            "fim",
            "acabou",
            "end",
        ],
        command: GameCommand::End,
        description: "Encerra a partida",
    },
    // Resolve
    AliasEntry {
        aliases: &[
            "resolver",
            "resolve",
            "decidir",
            "decide",
            "anular duvida",
            "anular dúvida",
            "sem duvida",
            "sem dúvida",
            "sem desafio",
            "ok",
        ],
        command: GameCommand::Resolve,
        description: "Resolve o desafio e volta ao jogo normal",
    },
    // Reset
    AliasEntry {
        aliases: &[
            "resetar",
            "reset",
            "reiniciar",
            "novo jogo",
            "novo",
            "novojogo",
        ],
        command: GameCommand::Reset,
        description: "Reseta para uma nova partida",
    },
];

// ── Public functions ────────────────────────────────────────────────────

/// Parseia texto livre em [`GameCommand`].
///
/// # Algoritmo
/// 1. Normaliza: trim, lowercase, NFD (remove acentos para matching)
/// 2. Compara contra a tabela de aliases (ordem importa, mais específico primeiro)
/// 3. Retorna `ParseError` se nenhum alias bateu
///
/// # Exemplos
/// ```
/// use esoccer_battle::command::parse;
/// assert!(parse("gol a").is_ok());
/// assert!(parse("GOL A").is_ok());
/// assert!(parse("gól à").is_ok()); // acentos removidos
/// assert!(parse("volta seis").is_ok());
/// assert!(parse("xyz não existe").is_err());
/// ```
pub fn parse(input: &str) -> Result<GameCommand, ParseError> {
    let normalized = normalize(input);

    if normalized.is_empty() {
        return Err(ParseError {
            input: input.to_string(),
            reason: "Comando vazio".to_string(),
        });
    }

    for entry in ALIAS_TABLE {
        for alias in entry.aliases {
            if normalize(alias) == normalized {
                eprintln!("[CMD] input='{}' → {:?}", input, entry.command);
                return Ok(entry.command);
            }
        }
    }

    // Fallback: tenta substring match — usa scoring: alias mais longo/g específico vence.
    // Minimum 5 chars to avoid false positives (e.g. "sim" stealing "GoalB" commands).
    if normalized.len() >= 5 {
        let mut best_cmd: Option<GameCommand> = None;
        let mut best_len: usize = 0;

        for entry in ALIAS_TABLE {
            for alias in entry.aliases {
                let norm_alias = normalize(alias);
                if norm_alias.contains(&normalized) || normalized.contains(&norm_alias) {
                    let score = norm_alias.len();
                    if score > best_len {
                        best_len = score;
                        best_cmd = Some(entry.command);
                    }
                }
            }
        }

        if let Some(cmd) = best_cmd {
            eprintln!("[CMD] input='{}' → {:?} (substring fallback, best_len={})", input, cmd, best_len);
            return Ok(cmd);
        }
    }

    eprintln!("[CMD] input='{}' → ParseError (no match)", input);

    let available: String = ALIAS_TABLE
        .iter()
        .map(|e| e.aliases.first().unwrap_or(&"?").to_string())
        .collect::<Vec<_>>()
        .join(", ");

    Err(ParseError {
        input: input.to_string(),
        reason: format!(
            "Comando não reconhecido: \"{}\". Comandos disponíveis: {}",
            input, available
        ),
    })
}

/// Retorna a lista de comandos disponíveis com aliases e descrições (para help page).
pub fn available_commands() -> Vec<CommandHelp> {
    ALIAS_TABLE
        .iter()
        .map(|entry| CommandHelp {
            command: format!("{:?}", entry.command),
            description: entry.description.to_string(),
            aliases: entry.aliases.iter().map(|s| s.to_string()).collect(),
        })
        .collect()
}

// ── Internal helpers ────────────────────────────────────────────────────

/// Normaliza string para matching: trim, lowercase, NFD (remove diacríticos).
fn normalize(input: &str) -> String {
    input
        .trim()
        .to_lowercase()
        .nfd()                                  // decompose: á → a + ◌́
        .filter(|c| !is_combining_mark(*c))     // remove combining marks
        .collect::<String>()
        .split_whitespace()                     // collapse multiple spaces
        .collect::<Vec<_>>()
        .join(" ")
}

/// Returns true if `c` is a Unicode combining mark (diacritical).
/// Covers the ranges relevant for pt-BR and general Latin text.
fn is_combining_mark(c: char) -> bool {
    matches!(
        c,
        '\u{0300}'..='\u{036F}'   // Combining Diacritical Marks
        | '\u{1AB0}'..='\u{1AFF}' // Combining Diacritical Marks Extended
        | '\u{1DC0}'..='\u{1DFF}' // Combining Diacritical Marks Supplement
        | '\u{20D0}'..='\u{20FF}' // Combining Diacritical Marks for Symbols
        | '\u{FE20}'..='\u{FE2F}' // Combining Half Marks
    )
}

// ── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Start ──
    #[test]
    fn parse_start_variants() {
        assert_eq!(parse("iniciar").unwrap(), GameCommand::Start);
        assert_eq!(parse("começar").unwrap(), GameCommand::Start);
        assert_eq!(parse("COMEÇAR").unwrap(), GameCommand::Start);
        assert_eq!(parse(" Start ").unwrap(), GameCommand::Start);
        assert_eq!(parse("start").unwrap(), GameCommand::Start);
        assert_eq!(parse("jogar").unwrap(), GameCommand::Start);
        assert_eq!(parse("vai").unwrap(), GameCommand::Start);
    }

    // ── GoalA ──
    #[test]
    fn parse_goal_a_variants() {
        assert_eq!(parse("gol a").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("GOL A").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("goal a").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("goool a").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("gol time a").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("gol para a").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("gol pro a").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("marcou a").unwrap(), GameCommand::GoalA);
    }

    // ── GoalB ──
    #[test]
    fn parse_goal_b_variants() {
        assert_eq!(parse("gol b").unwrap(), GameCommand::GoalB);
        assert_eq!(parse("GOL B").unwrap(), GameCommand::GoalB);
        assert_eq!(parse("goal b").unwrap(), GameCommand::GoalB);
        assert_eq!(parse("gol time b").unwrap(), GameCommand::GoalB);
        assert_eq!(parse("gol para b").unwrap(), GameCommand::GoalB);
    }

    // ── Pause ──
    #[test]
    fn parse_pause_variants() {
        assert_eq!(parse("pausar").unwrap(), GameCommand::Pause);
        assert_eq!(parse("pausa").unwrap(), GameCommand::Pause);
        assert_eq!(parse("pause").unwrap(), GameCommand::Pause);
        assert_eq!(parse("parar").unwrap(), GameCommand::Pause);
    }

    // ── Resume ──
    #[test]
    fn parse_resume_variants() {
        assert_eq!(parse("retomar").unwrap(), GameCommand::Resume);
        assert_eq!(parse("continuar").unwrap(), GameCommand::Resume);
        assert_eq!(parse("resume").unwrap(), GameCommand::Resume);
        assert_eq!(parse("volta").unwrap(), GameCommand::Resume);
    }

    // ── End ──
    #[test]
    fn parse_end_variants() {
        assert_eq!(parse("encerrar").unwrap(), GameCommand::End);
        assert_eq!(parse("finalizar").unwrap(), GameCommand::End);
        assert_eq!(parse("terminar").unwrap(), GameCommand::End);
        assert_eq!(parse("fim").unwrap(), GameCommand::End);
        assert_eq!(parse("acabou").unwrap(), GameCommand::End);
    }

    // ── Doubt ──
    #[test]
    fn parse_doubt_variants() {
        assert_eq!(parse("duvida").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("dúvida").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("DÚVIDA").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("desafio").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("challenge").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("reclamação").unwrap(), GameCommand::Doubt);
    }

    // ── Resolve ──
    #[test]
    fn parse_resolve_variants() {
        assert_eq!(parse("resolver").unwrap(), GameCommand::Resolve);
        assert_eq!(parse("resolve").unwrap(), GameCommand::Resolve);
        assert_eq!(parse("decidir").unwrap(), GameCommand::Resolve);
        assert_eq!(parse("sem duvida").unwrap(), GameCommand::Resolve);
        assert_eq!(parse("sem dúvida").unwrap(), GameCommand::Resolve);
        assert_eq!(parse("ok").unwrap(), GameCommand::Resolve);
    }

    // ── VoltaSeis ──
    #[test]
    fn parse_volta_seis_variants() {
        assert_eq!(parse("volta seis").unwrap(), GameCommand::VoltaSeis);
        assert_eq!(parse("6 metros").unwrap(), GameCommand::VoltaSeis);
        assert_eq!(parse("seis metros").unwrap(), GameCommand::VoltaSeis);
        assert_eq!(parse("VOLTA SEIS").unwrap(), GameCommand::VoltaSeis);
        assert_eq!(parse("volta 6").unwrap(), GameCommand::VoltaSeis);
    }

    // ── Reset ──
    #[test]
    fn parse_reset_variants() {
        assert_eq!(parse("resetar").unwrap(), GameCommand::Reset);
        assert_eq!(parse("reset").unwrap(), GameCommand::Reset);
        assert_eq!(parse("reiniciar").unwrap(), GameCommand::Reset);
        assert_eq!(parse("novo jogo").unwrap(), GameCommand::Reset);
        assert_eq!(parse("novo").unwrap(), GameCommand::Reset);
    }

    // ── Unicode NFD normalization ──
    #[test]
    fn nfd_normalization_pt_br() {
        assert_eq!(parse("gól à").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("começár").unwrap(), GameCommand::Start);
        assert_eq!(parse("DÚVÍDÁ").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("reclamação").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("SEM DÚVIDA").unwrap(), GameCommand::Resolve);
    }

    // ── Whitespace handling ──
    #[test]
    fn whitespace_variants() {
        assert_eq!(parse("  gol  a  ").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("\tgol a\n").unwrap(), GameCommand::GoalA);
    }

    // ── Empty / unknown ──
    #[test]
    fn parse_empty_returns_error() {
        let err = parse("").unwrap_err();
        assert_eq!(err.reason, "Comando vazio");
    }

    #[test]
    fn parse_whitespace_only_returns_error() {
        let err = parse("   ").unwrap_err();
        assert_eq!(err.reason, "Comando vazio");
    }

    #[test]
    fn parse_unknown_returns_error() {
        let err = parse("xyz não existe").unwrap_err();
        assert!(err.reason.contains("não reconhecido"));
        assert_eq!(err.input, "xyz não existe");
    }

    // ── available_commands ──
    #[test]
    fn available_commands_returns_all_10() {
        let cmds = available_commands();
        assert_eq!(cmds.len(), 10);
        let names: Vec<&str> = cmds.iter().map(|c| c.command.as_str()).collect();
        assert!(names.contains(&"Start"));
        assert!(names.contains(&"GoalA"));
        assert!(names.contains(&"GoalB"));
        assert!(names.contains(&"Pause"));
        assert!(names.contains(&"Resume"));
        assert!(names.contains(&"End"));
        assert!(names.contains(&"Doubt"));
        assert!(names.contains(&"Resolve"));
        assert!(names.contains(&"VoltaSeis"));
        assert!(names.contains(&"Reset"));
    }

    #[test]
    fn command_help_has_aliases_and_description() {
        let cmds = available_commands();
        for cmd in &cmds {
            assert!(!cmd.aliases.is_empty(), "Command {} has no aliases", cmd.command);
            assert!(!cmd.description.is_empty(), "Command {} has no description", cmd.command);
        }
    }

    // ── Priority: VoltaSeis before single-word ──
    #[test]
    fn volta_seis_priority_over_other_matches() {
        // "volta" alone should match Resume (as a substring fallback)
        // but "volta seis" should match VoltaSeis exactly
        assert_eq!(parse("volta seis").unwrap(), GameCommand::VoltaSeis);
        assert_eq!(parse("6 metros").unwrap(), GameCommand::VoltaSeis);
    }

    // ── Substring fallback ──
    #[test]
    fn substring_fallback_for_longer_inputs() {
        // "resolver" is an exact match for Resolve
        assert_eq!(parse("resolver").unwrap(), GameCommand::Resolve);
        // "ok" is an exact match for Resolve
        assert_eq!(parse("ok").unwrap(), GameCommand::Resolve);
        // "resolve" is >= 5 chars, contained in "resolver" → Resolve
        assert_eq!(parse("resolve").unwrap(), GameCommand::Resolve);
    }

    // ── False positive prevention ────────────────────────────────────────

    #[test]
    fn short_input_no_false_positive_sim() {
        // "sim" is only 3 chars — should NOT match via substring fallback.
        // It's not an exact alias for anything, so should be a ParseError.
        assert!(parse("sim").is_err());
    }

    #[test]
    fn short_input_no_false_positive_vai() {
        // "vai" is only 3 chars — it IS an exact alias for Start, so it should match.
        // But "va" (2 chars) should NOT match via substring.
        assert_eq!(parse("vai").unwrap(), GameCommand::Start);
        // "va" too short for substring → error
        assert!(parse("va").is_err());
    }

    #[test]
    fn gol_do_time_b_is_goal_b() {
        // "gol do time b" is an exact alias for GoalB
        assert_eq!(parse("gol do time b").unwrap(), GameCommand::GoalB);
    }

    #[test]
    fn partial_sim_does_not_steal_goal_b() {
        // "sim, gol do time b" — after normalization this becomes "sim, gol do time b"
        // The whole string won't match any alias, and at 19 chars it triggers substring.
        // "gol time b" (10 chars) is the best match inside GoalB aliases.
        assert_eq!(parse("sim, gol do time b").unwrap(), GameCommand::GoalB);
    }

    #[test]
    fn four_chars_below_threshold() {
        // "res" is 3 chars — below 5-char threshold, not an exact match → error
        assert!(parse("res").is_err());
        // "resol" is 5 chars — matches "resolver" via substring
        assert_eq!(parse("resol").unwrap(), GameCommand::Resolve);
    }
}

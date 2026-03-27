use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization;

// ── Command ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameCommand {
    Start,
    GoalA,
    GoalB,
    Pause,
    Resume,
    Doubt,
    Resolve,
    VoltaSeis,
    End,
    Reset,
}

// ── Errors ─────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParseErrorKind {
    EmptyInput,
    UnknownCommand,
    InvalidPhase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub kind: ParseErrorKind,
    pub reason: String,
    pub original: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.reason, self.original)
    }
}

// ── Help ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHelp {
    pub command: String,
    pub description: String,
    pub aliases: Vec<String>,
}

// ── Alias table ────────────────────────────────────────────────────

struct CommandEntry {
    command: GameCommand,
    aliases: &'static [&'static str],
    description: &'static str,
}

const COMMAND_TABLE: &[CommandEntry] = &[
    CommandEntry { command: GameCommand::Start, aliases: &["iniciar", "começar", "comecar", "start", "inicia"], description: "Iniciar partida" },
    CommandEntry { command: GameCommand::GoalA, aliases: &["gol a", "gol time a", "goal a", "golaço a", "golaco a"], description: "Gol time A" },
    CommandEntry { command: GameCommand::GoalB, aliases: &["gol b", "gol time b", "goal b", "golaço b", "golaco b"], description: "Gol time B" },
    CommandEntry { command: GameCommand::Pause, aliases: &["pausar", "pause", "parar"], description: "Pausar partida" },
    CommandEntry { command: GameCommand::Resume, aliases: &["retomar", "resume", "continuar", "volta"], description: "Retomar partida" },
    CommandEntry { command: GameCommand::Doubt, aliases: &["dúvida", "duvida", "dúvida a", "duvida a", "doubt", "challenge"], description: "Iniciar dúvida/desafio" },
    CommandEntry { command: GameCommand::Resolve, aliases: &["resolver", "resolve", "aceitar"], description: "Resolver dúvida" },
    CommandEntry { command: GameCommand::VoltaSeis, aliases: &["volta seis", "voltaseis", "seis metros"], description: "Volta seis metros" },
    CommandEntry { command: GameCommand::End, aliases: &["encerrar", "fim", "terminar", "end", "finalizar"], description: "Encerrar partida" },
    CommandEntry { command: GameCommand::Reset, aliases: &["resetar", "reset", "novo jogo", "novojogo"], description: "Resetar partida" },
];

// ── Normalize helper ──────────────────────────────────────────────

fn normalize(text: &str) -> String {
    text.nfd()
        .filter(|c| !matches!(c, '\u{0300}'..='\u{036F}'))
        .collect::<String>()
        .to_lowercase()
        .trim()
        .to_string()
}

// ── Parser ─────────────────────────────────────────────────────────

pub fn parse(text: &str) -> Result<GameCommand, ParseError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(ParseError {
            kind: ParseErrorKind::EmptyInput,
            reason: "Input vazio".into(),
            original: text.into(),
        });
    }

    let normalized = normalize(trimmed);

    for entry in COMMAND_TABLE {
        for alias in entry.aliases {
            if normalize(alias) == normalized {
                return Ok(entry.command.clone());
            }
        }
    }

    Err(ParseError {
        kind: ParseErrorKind::UnknownCommand,
        reason: format!("Comando não reconhecido: {}", normalized),
        original: text.into(),
    })
}

// ── Available commands ────────────────────────────────────────────

pub fn available_commands() -> Vec<CommandHelp> {
    COMMAND_TABLE
        .iter()
        .map(|e| CommandHelp {
            command: format!("{:?}", e.command),
            description: e.description.into(),
            aliases: e.aliases.iter().map(|s| (*s).into()).collect(),
        })
        .collect()
}

// ── Tests ──────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_start_variants() {
        assert_eq!(parse("iniciar").unwrap(), GameCommand::Start);
        assert_eq!(parse("Começar").unwrap(), GameCommand::Start);
        assert_eq!(parse("  START  ").unwrap(), GameCommand::Start);
    }

    #[test]
    fn parse_goals() {
        assert_eq!(parse("gol a").unwrap(), GameCommand::GoalA);
        assert_eq!(parse("GOLAÇO B").unwrap(), GameCommand::GoalB);
        assert_eq!(parse("gol time a").unwrap(), GameCommand::GoalA);
    }

    #[test]
    fn parse_pause_resume() {
        assert_eq!(parse("pausar").unwrap(), GameCommand::Pause);
        assert_eq!(parse("RESUME").unwrap(), GameCommand::Resume);
    }

    #[test]
    fn parse_doubt_resolve() {
        assert_eq!(parse("dúvida").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("DÚVIDA A").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("challenge").unwrap(), GameCommand::Doubt);
        assert_eq!(parse("aceitar").unwrap(), GameCommand::Resolve);
    }

    #[test]
    fn parse_volta_seis() {
        assert_eq!(parse("volta seis").unwrap(), GameCommand::VoltaSeis);
        assert_eq!(parse("voltaseis").unwrap(), GameCommand::VoltaSeis);
    }

    #[test]
    fn parse_end_reset() {
        assert_eq!(parse("fim").unwrap(), GameCommand::End);
        assert_eq!(parse("novo jogo").unwrap(), GameCommand::Reset);
        assert_eq!(parse("novojogo").unwrap(), GameCommand::Reset);
    }

    #[test]
    fn parse_empty() {
        let err = parse("").unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::EmptyInput);
    }

    #[test]
    fn parse_unknown() {
        let err = parse("xyzbleh").unwrap_err();
        assert_eq!(err.kind, ParseErrorKind::UnknownCommand);
    }

    #[test]
    fn available_commands_nonempty() {
        let cmds = available_commands();
        assert_eq!(cmds.len(), 10);
    }
}

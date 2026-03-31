use serde::{Serialize, Deserialize};

/// 7 comandos — simples e completo
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum GameCommand {
    Start,
    GoalA,
    GoalB,
    Pause,
    Resume,
    End,
    Reset,
}

#[derive(Debug, Clone, Serialize)]
pub struct ParseError {
    pub input: String,
    pub reason: String,
}

/// Parseia texto livre em GameCommand
pub fn parse(input: &str) -> Result<GameCommand, ParseError> {
    let normalized = input
        .trim()
        .to_lowercase()
        .replace(|c: char| !c.is_ascii_alphanumeric() && c != ' ', "");

    let command = match normalized.as_str() {
        s if ["volta 6", "volta seis", "iniciar", "start"].contains(&s) => GameCommand::Start,
        s if ["gol do time a", "gol a", "goal a", "gol time a"].contains(&s) => GameCommand::GoalA,
        s if ["gol do time b", "gol b", "goal b", "gol time b"].contains(&s) => GameCommand::GoalB,
        s if ["duvida", "dúvida", "pause", "pausar"].contains(&s) => GameCommand::Pause,
        s if ["retornar", "volta", "continua", "continuar", "resume"].contains(&s) => GameCommand::Resume,
        s if ["encerrar", "terminar", "end", "finalizar"].contains(&s) => GameCommand::End,
        s if ["novo jogo", "reset", "reiniciar"].contains(&s) => GameCommand::Reset,
        _ => return Err(ParseError { input: input.to_string(), reason: format!("Comando não reconhecido: {}", input) }),
    };

    Ok(command)
}

#[derive(Debug, Clone, Serialize)]
pub struct CommandHelp {
    pub command: String,
    pub description: String,
    pub aliases: Vec<String>,
}

pub fn available_commands() -> Vec<CommandHelp> {
    vec![
        CommandHelp { command: "Start".to_string(), description: "Iniciar partida".to_string(), aliases: vec!["volta 6".into(), "volta seis".into(), "iniciar".into()] },
        CommandHelp { command: "GoalA".to_string(), description: "Gol do time A".to_string(), aliases: vec!["gol a".into(), "gol do time a".into()] },
        CommandHelp { command: "GoalB".to_string(), description: "Gol do time B".to_string(), aliases: vec!["gol b".into(), "gol do time b".into()] },
        CommandHelp { command: "Pause".to_string(), description: "Pausar partida".to_string(), aliases: vec!["dúvida".into(), "duvida".into()] },
        CommandHelp { command: "Resume".to_string(), description: "Retornar ao jogo".to_string(), aliases: vec!["retornar".into(), "volta".into()] },
        CommandHelp { command: "End".to_string(), description: "Encerrar partida".to_string(), aliases: vec!["encerrar".into(), "terminar".into()] },
        CommandHelp { command: "Reset".to_string(), description: "Novo jogo".to_string(), aliases: vec!["novo jogo".into(), "reset".into()] },
    ]
}

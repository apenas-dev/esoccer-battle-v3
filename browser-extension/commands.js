/**
 * S.O.G. Battle — Command definitions
 * Each command has patterns (for voice matching), a display response, and an audio file.
 * Sorted longest-pattern-first to avoid partial-match conflicts.
 */

const COMMANDS = [
  {
    id: "dupla-agora",
    label: "Dupla agora",
    patterns: ["dupla agora", "dupla"],
    response: "Dupla agora",
    audioFile: "dupla-agora.mp3",
  },
  {
    id: "volta-6",
    label: "Volta 6",
    patterns: ["volta seis", "volta 6", "seis minutos", "6 minutos"],
    response: "Volta 6",
    audioFile: "volta-6.mp3",
  },
  {
    id: "intervalo",
    label: "Intervalo",
    patterns: ["intervalo", "pausa", "tempo"],
    response: "Intervalo",
    audioFile: "intervalo.mp3",
  },
  {
    id: "resultado",
    label: "Resultado",
    patterns: ["resultado", "resultados", "placar"],
    response: "Resultado",
    audioFile: "resultado.mp3",
  },
  {
    id: "encerrar",
    label: "Encerrar",
    patterns: ["encerrar", "encerrar partida", "fim de jogo", "terminar"],
    response: "Encerrar",
    audioFile: "encerrar.mp3",
  },
];

// Export for use in other scripts
if (typeof globalThis !== "undefined") {
  globalThis.COMMANDS = COMMANDS;
}

import { useState, useCallback } from 'react';
import { motion } from 'framer-motion';
import { useMatchState } from '../hooks/useMatchState';
import { useVoiceCommands } from '../hooks/useVoiceCommands';
import { Scoreboard } from '../components/match/Scoreboard';
import { MatchTimer } from '../components/match/MatchTimer';
import { VoiceIndicator, type VoiceState } from '../components/match/VoiceIndicator';
import { CommandLog, type CommandEntry } from '../components/match/CommandLog';
import { MatchControls } from '../components/match/MatchControls';
import { type MatchStatus } from '../lib/types';

function mapStatus(status: string): MatchStatus {
  if (status === 'playing' || status === 'challenge' || status === 'finished' || status === 'idle' || status === 'paused') return status;
  return 'idle';
}

function mapVoiceState(status: string): VoiceState {
  if (status === 'playing') return 'listening';
  if (status === 'challenge') return 'processing';
  return 'idle';
}

function generateId(): string {
  return (typeof crypto !== 'undefined' && crypto.randomUUID)
    ? crypto.randomUUID()
    : `cmd-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

function useCommandLog() {
  const [commands, setCommands] = useState<CommandEntry[]>([]);
  return [commands, setCommands] as const;
}

function addCommand(setter: React.Dispatch<React.SetStateAction<CommandEntry[]>>, text: string, type: CommandEntry['type'] = 'control') {
  setter((prev) => [{ id: generateId(), text, timestamp: new Date(), type }, ...prev]);
}

export function MatchPageConnected() {
  const { matchState, startMatch, endMatch, goalA, goalB, restart, challenge, resolveChallenge } = useMatchState();
  const voice = useVoiceCommands(matchState.status === 'playing');

  const uiStatus = mapStatus(matchState.status);
  const voiceState = mapVoiceState(matchState.status);

  const [commands, setCommands] = useCommandLog();

  const handleStart = useCallback(() => { startMatch(); addCommand(setCommands, 'Partida iniciada'); }, [startMatch, setCommands]);
  const handleEnd = useCallback(() => { endMatch(); addCommand(setCommands, 'Partida encerrada'); }, [endMatch, setCommands]);
  const handleGoalA = useCallback(() => { goalA(); addCommand(setCommands, `⚽ Gol do ${matchState.team_a_name}`, 'goal'); }, [goalA, matchState.team_a_name, setCommands]);
  const handleGoalB = useCallback(() => { goalB(); addCommand(setCommands, `⚽ Gol do ${matchState.team_b_name}`, 'goal'); }, [goalB, matchState.team_b_name, setCommands]);
  const handleRestart = useCallback(() => { restart(); addCommand(setCommands, '↩ Volta seis'); }, [restart, setCommands]);
  const handleChallenge = useCallback(() => { challenge(); addCommand(setCommands, '❓ Dúvida', 'challenge'); }, [challenge, setCommands]);
  const handleResolveChallenge = useCallback(() => { resolveChallenge(); addCommand(setCommands, '✅ Dúvida resolvida', 'challenge'); }, [resolveChallenge, setCommands]);

  return (
    <div className="min-h-screen bg-[#0a0f1a] text-white flex flex-col items-center px-4 py-6 sm:px-6 sm:py-8">
      <motion.header initial={{ opacity: 0, y: -20 }} animate={{ opacity: 1, y: 0 }} className="w-full max-w-3xl mb-6 sm:mb-8">
        <h1 className="text-lg sm:text-xl font-bold text-center tracking-tight">
          <span className="text-[#00ff88]">E-Soccer</span>{' '}
          <span className="text-gray-400">Battle</span>
        </h1>
      </motion.header>

      <main className="w-full max-w-3xl flex flex-col items-center gap-6 sm:gap-8">
        <section aria-label="Placar">
          <Scoreboard
            teamAName={matchState.team_a_name}
            teamBName={matchState.team_b_name}
            scoreA={matchState.score_a}
            scoreB={matchState.score_b}
            status={uiStatus}
          />
        </section>
        <section aria-label="Cronômetro">
          <MatchTimer elapsedSeconds={matchState.elapsed_seconds} isRunning={matchState.status === 'playing'} />
        </section>
        <section aria-label="Indicador de voz">
          <VoiceIndicator voiceState={voiceState} />
          {voice.lastText && (
            <p className="text-xs text-gray-500 text-center mt-1 max-w-xs truncate">
              🎤 &ldquo;{voice.lastText}&rdquo;
            </p>
          )}
        </section>
        <section aria-label="Log de comandos">
          <CommandLog commands={commands} maxEntries={5} />
        </section>
        <section aria-label="Controles manuais">
          <MatchControls
            status={uiStatus}
            onStart={handleStart}
            onEnd={handleEnd}
            onUndo={handleRestart}
            onChallenge={handleChallenge}
            onResolveChallenge={handleResolveChallenge}
            onGoalA={handleGoalA}
            onGoalB={handleGoalB}
            teamAName={matchState.team_a_name}
            teamBName={matchState.team_b_name}
          />
        </section>
      </main>

      <footer className="mt-auto pt-8 pb-2">
        <p className="text-xs text-gray-700 text-center">
          E-Soccer Battle V3 · Tauri Connected
        </p>
      </footer>
    </div>
  );
}

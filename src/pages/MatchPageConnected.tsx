import { useState, useCallback, useEffect, useRef } from 'react';
import { motion } from 'framer-motion';
import { useMatchState } from '../hooks/useMatchState';
import { useVoiceCommands } from '../hooks/useVoiceCommands';
import { Scoreboard } from '../components/match/Scoreboard';
import { MatchTimer } from '../components/match/MatchTimer';
import { VoiceIndicator, type VoiceState } from '../components/match/VoiceIndicator';
import { CommandLog, type CommandEntry } from '../components/match/CommandLog';
import { MatchControls } from '../components/match/MatchControls';
import { startRecording, stopRecordingAndTranscribe, onRecordingState, type RecordingStatePayload, getSettings, setSettings } from '../lib/tauri';
import { generateId } from '../lib/utils';
import { type MatchStatus } from '../lib/types';

function mapStatus(status: string): MatchStatus {
  if (status === 'playing' || status === 'challenge' || status === 'finished' || status === 'idle' || status === 'paused') return status;
  return 'idle';
}

function useCommandLog() {
  const [commands, setCommands] = useState<CommandEntry[]>([]);
  return [commands, setCommands] as const;
}

function addCommand(setter: React.Dispatch<React.SetStateAction<CommandEntry[]>>, text: string, type: CommandEntry['type'] = 'control') {
  setter((prev) => [{ id: generateId(), text, timestamp: new Date(), type }, ...prev]);
}

interface MatchPageConnectedProps {
  onNavigateSettings?: () => void;
  onNavigateHelp?: () => void;
  onNavigateHistory?: () => void;
}

export function MatchPageConnected({ onNavigateSettings, onNavigateHelp, onNavigateHistory }: MatchPageConnectedProps) {
  const { matchState, startMatch, endMatch, goalA, goalB, challenge, resolveChallenge, pauseMatch, resumeMatch, undoGoal, restart, setScoreA, setScoreB } = useMatchState();
  const voice = useVoiceCommands(matchState.status === 'playing');

  const uiStatus = mapStatus(matchState.status);

  // ── Push-to-Talk state machine ──────────────────────
  const [recordingState, setRecordingState] = useState<'idle' | 'recording' | 'processing'>('idle');

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    onRecordingState((payload: RecordingStatePayload) => {
      setRecordingState(payload.status);
    }).then((fn) => { unlisten = fn; });
    return () => { unlisten?.(); };
  }, []);

  const voiceState: VoiceState = recordingState === 'recording' ? 'listening'
    : recordingState === 'processing' ? 'processing'
    : 'idle';

  const handleMicClick = useCallback(async () => {
    if (recordingState === 'idle') {
      try { await startRecording(); } catch (e) { console.error('startRecording failed:', e); }
    } else if (recordingState === 'recording') {
      try { await stopRecordingAndTranscribe(); } catch (e) { console.error('stopRecordingAndTranscribe failed:', e); }
    }
    // processing → ignored (button disabled)
  }, [recordingState]);

  const [commands, setCommands] = useCommandLog();
  const prevVoiceTextRef = useRef('');

  // Log comandos de voz no CommandLog
  useEffect(() => {
    if (voice.lastText && voice.lastText !== prevVoiceTextRef.current) {
      prevVoiceTextRef.current = voice.lastText;
      addCommand(setCommands, `🎤 "${voice.lastText}"`, 'voice');
    }
  }, [voice.lastText, setCommands]);

  const handleStart = useCallback(() => { startMatch(); addCommand(setCommands, 'Partida iniciada'); }, [startMatch, setCommands]);
  const handleEnd = useCallback(() => { endMatch(); addCommand(setCommands, 'Partida encerrada'); }, [endMatch, setCommands]);
  const handleGoalA = useCallback(() => { goalA(); addCommand(setCommands, `⚽ Gol do ${matchState.team_a_name}`, 'goal'); }, [goalA, matchState.team_a_name, setCommands]);
  const handleGoalB = useCallback(() => { goalB(); addCommand(setCommands, `⚽ Gol do ${matchState.team_b_name}`, 'goal'); }, [goalB, matchState.team_b_name, setCommands]);
  const handlePause = useCallback(() => { pauseMatch(); addCommand(setCommands, '⏸ Partida pausada'); }, [pauseMatch, setCommands]);
  const handleResume = useCallback(() => { resumeMatch(); addCommand(setCommands, '▶ Partida retomada'); }, [resumeMatch, setCommands]);
  const handleUndo = useCallback(() => { undoGoal(); addCommand(setCommands, '↶ Desfeito último gol'); }, [undoGoal, setCommands]);
  const handleRestart = useCallback(() => { restart(); addCommand(setCommands, '↩ Volta Seis — contagem reiniciada'); }, [restart, setCommands]);
  const handleScoreAChange = useCallback((newScore: number) => { setScoreA(newScore); }, [setScoreA]);
  const handleScoreBChange = useCallback((newScore: number) => { setScoreB(newScore); }, [setScoreB]);
  const handleChallenge = useCallback(() => { challenge(); addCommand(setCommands, '❓ Dúvida', 'challenge'); }, [challenge, setCommands]);
  const handleResolveChallenge = useCallback(() => { resolveChallenge(); addCommand(setCommands, '✅ Dúvida resolvida', 'challenge'); }, [resolveChallenge, setCommands]);
  const handleTeamNameChange = useCallback(async (team: 'a' | 'b', value: string) => {
    try {
      const settings = await getSettings();
      const updated = team === 'a'
        ? { ...settings, team_a_name: value }
        : { ...settings, team_b_name: value };
      await setSettings(updated);
    } catch (e) { console.error('Failed to save team name:', e); }
  }, []);

  return (
    <div className="min-h-screen bg-[#0a0f1a] text-white flex flex-col items-center px-4 py-6 sm:px-6 sm:py-8">
      <motion.header initial={{ opacity: 0, y: -20 }} animate={{ opacity: 1, y: 0 }} className="w-full max-w-3xl mb-6 sm:mb-8">
        <h1 className="text-lg sm:text-xl font-bold text-center tracking-tight flex-1">
          <span className="text-[#00ff88]">E-Soccer</span>{' '}
          <span className="text-gray-400">Battle</span>
        </h1>
        {onNavigateHelp && (
          <button
            onClick={onNavigateHelp}
            className="text-gray-500 hover:text-white transition-colors p-1.5 rounded-lg hover:bg-gray-800"
            aria-label="Ajuda"
          >
            ❓
          </button>
        )}
        {onNavigateHistory && (
          <button
            onClick={onNavigateHistory}
            className="text-gray-500 hover:text-white transition-colors p-1.5 rounded-lg hover:bg-gray-800"
            aria-label="Histórico"
          >
            📋
          </button>
        )}
        {onNavigateSettings && (
          <button
            onClick={onNavigateSettings}
            className="text-gray-500 hover:text-white transition-colors p-1.5 rounded-lg hover:bg-gray-800"
            aria-label="Configurações"
          >
            ⚙️
          </button>
        )}
      </motion.header>

      <main className="w-full max-w-3xl flex flex-col items-center gap-6 sm:gap-8">
        <section aria-label="Placar">
          <Scoreboard
            teamAName={matchState.team_a_name}
            teamBName={matchState.team_b_name}
            scoreA={matchState.score_a}
            scoreB={matchState.score_b}
            status={uiStatus}
            canEditNames={matchState.status === 'idle'}
            onTeamNameChange={handleTeamNameChange}
            onScoreAChange={handleScoreAChange}
            onScoreBChange={handleScoreBChange}
          />
        </section>
        <section aria-label="Cronômetro">
          <MatchTimer elapsedSeconds={matchState.elapsed_seconds} isRunning={matchState.status === 'playing'} />
        </section>
        <section aria-label="Indicador de voz">
          <VoiceIndicator voiceState={voiceState} onClick={handleMicClick} disabled={recordingState === 'processing'} />
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
            onPause={handlePause}
            onResume={handleResume}
            onEnd={handleEnd}
            onUndo={handleUndo}
            onRestart={handleRestart}
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

import { useState, useEffect } from 'react';
import type { GamePhase, PlayingSubPhase, TimerMode, CommandLogEntry } from '../../types';
import { Scoreboard } from './Scoreboard';
import { Timer } from './Timer';
import { Controls } from './Controls';
import { VoiceIndicator } from './VoiceIndicator';
import { CommandLog } from './CommandLog';
import { generateId } from '../../lib/utils';

interface MatchLayoutProps {
  phase: GamePhase;
  subPhase: PlayingSubPhase;
  teamAName: string;
  teamBName: string;
  scoreA: number;
  scoreB: number;
  displayTime: string;
  timerMode: TimerMode;
  totalDuration: number;
  elapsed: number;
  isLoading: boolean;
  voiceStatus: import('../../types').VoiceStatus;
  lastTranscript: string | null;
  isListening: boolean;
  /** BUG 2 FIX: Latest voice command text so we can log it. */
  lastVoiceCommand: string | null;
  onExecuteCommand: (text: string) => Promise<void>;
  onResetMatch: () => Promise<void>;
  onStartListening: () => void;
  onStopListening: () => void;
}

function detectFlashTeam(text: string): 'a' | 'b' | null {
  const lower = text.toLowerCase();
  if (lower.includes('gol') && lower.includes('a')) return 'a';
  if (lower.includes('gol') && lower.includes('b')) return 'b';
  return null;
}

export function MatchLayout({
  phase,
  subPhase,
  teamAName,
  teamBName,
  scoreA,
  scoreB,
  displayTime,
  timerMode,
  totalDuration,
  elapsed,
  isLoading,
  voiceStatus,
  lastTranscript,
  isListening,
  lastVoiceCommand,
  onExecuteCommand,
  onResetMatch,
  onStartListening,
  onStopListening,
}: MatchLayoutProps) {
  const [flashTeam, setFlashTeam] = useState<'a' | 'b' | null>(null);
  // BUG 2 FIX: Single source of truth for command log — lives here, receives both button and voice commands
  const [logEntries, setLogEntries] = useState<CommandLogEntry[]>([]);
  // Track last logged voice command to avoid duplicates
  const [loggedVoiceCommand, setLoggedVoiceCommand] = useState<string | null>(null);

  // BUG 2 + BUG 3 FIX: Log voice commands and trigger flash when a new voice transcript arrives
  useEffect(() => {
    if (lastVoiceCommand && lastVoiceCommand !== loggedVoiceCommand) {
      setLoggedVoiceCommand(lastVoiceCommand);
      setLogEntries((prev) => [
        ...prev,
        {
          id: generateId(),
          timestamp: new Date(),
          command: lastVoiceCommand,
          source: 'voice',
          success: true,
        },
      ]);
      const flash = detectFlashTeam(lastVoiceCommand);
      if (flash) {
        setFlashTeam(flash);
        setTimeout(() => setFlashTeam(null), 600);
      }
    }
  }, [lastVoiceCommand, loggedVoiceCommand]);

  const handleCommand = async (text: string) => {
    setLogEntries((prev) => [
      ...prev,
      {
        id: generateId(),
        timestamp: new Date(),
        command: text,
        source: 'button',
        success: text !== '',
      },
    ]);

    // BUG 3 FIX: Flash effect for goal button commands
    const flash = detectFlashTeam(text);
    if (flash) {
      setFlashTeam(flash);
      setTimeout(() => setFlashTeam(null), 600);
    }

    await onExecuteCommand(text);
  };

  if (isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-lg text-[var(--text-secondary)]">Carregando...</div>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-6">
      <Scoreboard
        teamAName={teamAName}
        teamBName={teamBName}
        scoreA={scoreA}
        scoreB={scoreB}
        phase={phase}
        subPhase={subPhase}
        flashTeam={flashTeam}
      />

      <Timer
        displayTime={displayTime}
        phase={phase}
        timerMode={timerMode}
        totalDuration={totalDuration}
        elapsed={elapsed}
      />

      <VoiceIndicator
        status={voiceStatus}
        lastTranscript={lastTranscript}
        isListening={isListening}
        onStart={onStartListening}
        onStop={onStopListening}
      />

      <Controls
        phase={phase}
        subPhase={subPhase}
        onExecuteCommand={handleCommand}
        onResetMatch={onResetMatch}
      />

      <CommandLog entries={logEntries} />
    </div>
  );
}

export type { MatchLayoutProps };

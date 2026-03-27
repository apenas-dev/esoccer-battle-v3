import { useState } from 'react';
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
  onExecuteCommand: (text: string) => Promise<void>;
  onResetMatch: () => Promise<void>;
  onStartListening: () => void;
  onStopListening: () => void;
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
  onExecuteCommand,
  onResetMatch,
  onStartListening,
  onStopListening,
}: MatchLayoutProps) {
  const [flashTeam, setFlashTeam] = useState<'a' | 'b' | null>(null);
  const [logEntries, setLogEntries] = useState<CommandLogEntry[]>([]);

  const handleCommand = async (text: string) => {
    const success = text !== '';
    setLogEntries((prev) => [
      ...prev,
      {
        id: generateId(),
        timestamp: new Date(),
        command: text,
        source: 'button',
        success,
      },
    ]);

    // Flash effect for goals
    if (text.includes('gol') && text.toLowerCase().includes('a')) {
      setFlashTeam('a');
      setTimeout(() => setFlashTeam(null), 600);
    } else if (text.includes('gol') && text.toLowerCase().includes('b')) {
      setFlashTeam('b');
      setTimeout(() => setFlashTeam(null), 600);
    }

    await onExecuteCommand(text);
  };

  const handleVoiceTranscript = async (text: string) => {
    setLogEntries((prev) => [
      ...prev,
      {
        id: generateId(),
        timestamp: new Date(),
        command: text,
        source: 'voice',
        success: true,
      },
    ]);

    if (text.toLowerCase().includes('gol') && text.toLowerCase().includes('a')) {
      setFlashTeam('a');
      setTimeout(() => setFlashTeam(null), 600);
    } else if (text.toLowerCase().includes('gol') && text.toLowerCase().includes('b')) {
      setFlashTeam('b');
      setTimeout(() => setFlashTeam(null), 600);
    }

    await onExecuteCommand(text);
  };

  // Expose voice transcript handler (child VoiceIndicator calls parent via props)
  // We handle this in MatchPage instead, so we just pass through
  void handleVoiceTranscript;

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

// Re-export handleVoiceTranscript as a way for MatchPage to connect voice to command log
export type { MatchLayoutProps };

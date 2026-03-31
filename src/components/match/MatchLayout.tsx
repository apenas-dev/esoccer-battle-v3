import type { GamePhase, MatchConfig } from '../../types';
import { Scoreboard } from './Scoreboard';
import { Timer } from './Timer';
import { Controls } from './Controls';
import { VoiceIndicator } from './VoiceIndicator';
import { CommandLog, CommandLogEntry } from './CommandLog';

interface MatchLayoutProps {
  phase: GamePhase;
  config: MatchConfig;
  scoreA: number;
  scoreB: number;
  displayTime: string;
  voiceStatus: import('../../types').VoiceStatus;
  isListening: boolean;
  lastTranscript: string | null;
  commandLog: CommandLogEntry[];
  onCommand: (text: string) => void;
  onVoiceStart: () => void;
  onVoiceStop: () => void;
}

export function MatchLayout({
  phase, config, scoreA, scoreB, displayTime,
  voiceStatus, isListening, lastTranscript, commandLog,
  onCommand, onVoiceStart, onVoiceStop,
}: MatchLayoutProps) {
  return (
    <div className="flex flex-col items-center justify-center min-h-screen p-6">
      <h1 className="text-2xl font-bold text-gray-300 mb-8">⚽ E-Soccer Battle</h1>
      
      <Scoreboard
        teamAName={config.team_a_name}
        teamBName={config.team_b_name}
        scoreA={scoreA}
        scoreB={scoreB}
        phase={phase}
      />

      <Timer
        displayTime={displayTime}
        phase={phase}
        durationSecs={config.duration_secs}
        timerMode={config.timer_mode}
      />

      <Controls phase={phase} onCommand={onCommand} />

      {phase !== 'finished' && (
        <VoiceIndicator
          status={voiceStatus}
          isListening={isListening}
          lastTranscript={lastTranscript}
          onStart={onVoiceStart}
          onStop={onVoiceStop}
        />
      )}

      <CommandLog entries={commandLog} />
    </div>
  );
}

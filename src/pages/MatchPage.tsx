import { useState, useEffect, useCallback, useRef } from 'react';
import { useMatchState } from '../hooks/useMatchState';
import { useVoicePipeline } from '../hooks/useVoicePipeline';
import { createSTTProvider } from '../services/stt/sttFactory';
import type { ISTTProvider } from '../services/stt/ISTTProvider';
import { MatchLayout } from '../components/match/MatchLayout';
import { generateId } from '../lib/utils';
import type { CommandLogEntry } from '../types';

export function MatchPage() {
  const { state, config, isLoading, displayTime, executeCommand, resetMatch } = useMatchState();
  const [provider, setProvider] = useState<ISTTProvider | null>(null);
  const [logEntries, setLogEntries] = useState<CommandLogEntry[]>([]);
  const logRef = useRef(logEntries);
  logRef.current = logEntries;

  // Create STT provider when config loads
  useEffect(() => {
    if (!config) return;
    createSTTProvider('auto', config).then(setProvider);
  }, [config]);

  const addLogEntry = useCallback((command: string, source: 'voice' | 'button', success: boolean) => {
    setLogEntries((prev) => [
      ...prev,
      { id: generateId(), timestamp: new Date(), command, source, success },
    ]);
  }, []);

  const handleVoiceTranscript = useCallback(
    async (text: string) => {
      addLogEntry(text, 'voice', true);
      await executeCommand(text);
    },
    [addLogEntry, executeCommand],
  );

  const voicePipeline = useVoicePipeline({
    provider: provider!,
    onTranscript: handleVoiceTranscript,
    onError: (e) => console.error('Voice error:', e),
  });

  // Handle button commands with logging
  const handleExecuteCommand = useCallback(
    async (text: string) => {
      addLogEntry(text, 'button', true);
      await executeCommand(text);
    },
    [addLogEntry, executeCommand],
  );

  if (!state) return null;

  return (
    <div className="mx-auto max-w-2xl">
      {/* We need to inline the MatchLayout logic to connect voice properly */}
      <MatchLayout
        phase={state.phase}
        subPhase={state.sub_phase}
        teamAName={state.config.team_a_name}
        teamBName={state.config.team_b_name}
        scoreA={state.score_a}
        scoreB={state.score_b}
        displayTime={displayTime}
        timerMode={state.config.timer_mode}
        totalDuration={state.config.duration_secs}
        elapsed={state.elapsed_secs}
        isLoading={isLoading}
        voiceStatus={voicePipeline.voiceStatus}
        lastTranscript={voicePipeline.lastTranscript}
        isListening={voicePipeline.isListening}
        onExecuteCommand={handleExecuteCommand}
        onResetMatch={resetMatch}
        onStartListening={voicePipeline.startListening}
        onStopListening={voicePipeline.stopListening}
      />
    </div>
  );
}

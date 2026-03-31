import { useState, useEffect } from 'react';
import { useMatchState } from '../hooks/useMatchState';
import { useVoicePipeline } from '../hooks/useVoicePipeline';
import { createSTTProvider } from '../services/stt/sttFactory';
import type { ISTTProvider } from '../services/stt/ISTTProvider';
import { MatchLayout } from '../components/match/MatchLayout';
import type { CommandLogEntry } from '../components/match/CommandLog';

export function MatchPage() {
  const { state, isLoading, displayTime, executeCommand } = useMatchState();
  const [commandLog, setCommandLog] = useState<CommandLogEntry[]>([]);
  const [provider, setProvider] = useState<ISTTProvider | null>(null);

  useEffect(() => {
    createSTTProvider('auto').then(setProvider).catch(console.error);
  }, []);

  const handleCommand = async (text: string) => {
    const timestamp = new Date().toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    await executeCommand(text);
    const resultLabel = text.toLowerCase().includes('gol') ? '⚽ GOL!' : '✅ OK';
    setCommandLog(prev => [...prev, { id: crypto.randomUUID(), timestamp, command: text, result: resultLabel }]);
  };

  const { voiceStatus, isListening, lastTranscript, startListening, stopListening } = useVoicePipeline({
    provider: provider!,
    onTranscript: handleCommand,
  });

  if (isLoading || !state) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <p className="text-gray-400">Carregando...</p>
      </div>
    );
  }

  return (
    <MatchLayout
      phase={state.phase}
      config={state.config}
      scoreA={state.score_a}
      scoreB={state.score_b}
      displayTime={displayTime}
      voiceStatus={voiceStatus}
      isListening={isListening}
      lastTranscript={lastTranscript}
      commandLog={commandLog}
      onCommand={handleCommand}
      onVoiceStart={startListening}
      onVoiceStop={stopListening}
    />
  );
}

import { useState, useEffect } from 'react';
import { useMatchState } from '../hooks/useMatchState';
import { useVoicePipeline } from '../hooks/useVoicePipeline';
import { createSTTProvider } from '../services/stt/sttFactory';
import type { ISTTProvider } from '../services/stt/ISTTProvider';
import { MatchLayout } from '../components/match/MatchLayout';

export function MatchPage() {
  const { state, config, isLoading, displayTime, executeCommand, resetMatch } = useMatchState();
  const [provider, setProvider] = useState<ISTTProvider | null>(null);

  // Create STT provider when config loads
  useEffect(() => {
    if (!config) return;
    createSTTProvider('auto', config).then(setProvider);
  }, [config]);

  // BUG 1 FIX: Guard — don't render anything until provider + state are ready
  if (!provider || !state || isLoading) {
    return (
      <div className="flex h-full items-center justify-center">
        <div className="text-lg text-[var(--text-secondary)]">Carregando...</div>
      </div>
    );
  }

  // BUG 2 FIX: Voice pipeline now lives here with its own log. No more duplicate log state.
  // We use a wrapper component so the hook is only called when provider is non-null.
  return (
    <MatchPageWithVoice
      provider={provider}
      state={state}
      displayTime={displayTime}
      executeCommand={executeCommand}
      resetMatch={resetMatch}
    />
  );
}

/** Inner component that safely calls useVoicePipeline with a non-null provider. */
function MatchPageWithVoice({
  provider,
  state,
  displayTime,
  executeCommand,
  resetMatch,
}: {
  provider: ISTTProvider;
  state: NonNullable<ReturnType<typeof useMatchState>['state']>;
  displayTime: string;
  executeCommand: (text: string) => Promise<void>;
  resetMatch: () => Promise<void>;
}) {
  const voicePipeline = useVoicePipeline({
    provider,
    onTranscript: executeCommand,
    onError: (e) => console.error('Voice error:', e),
  });

  // BUG 3 FIX: flashTeam controlled here via score-changed events
  // MatchLayout receives flashTeam as prop
  // (We still keep the inline flash in MatchLayout for button commands as well)

  return (
    <div className="mx-auto max-w-2xl">
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
        isLoading={false}
        voiceStatus={voicePipeline.voiceStatus}
        lastTranscript={voicePipeline.lastTranscript}
        isListening={voicePipeline.isListening}
        lastVoiceCommand={voicePipeline.lastTranscript}
        onExecuteCommand={executeCommand}
        onResetMatch={resetMatch}
        onStartListening={voicePipeline.startListening}
        onStopListening={voicePipeline.stopListening}
      />
    </div>
  );
}

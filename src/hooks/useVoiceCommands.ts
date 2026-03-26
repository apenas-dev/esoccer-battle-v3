import { useEffect, useRef, useState } from 'react';
import { createSTTProvider, type ISTTProvider, type STTProviderName } from '../services/stt';

export interface VoiceCommandState {
  lastText: string;
  lastUnknownText: string;
  isListening: boolean;
  providerName: string;
}

const idleState: VoiceCommandState = {
  lastText: '',
  lastUnknownText: '',
  isListening: false,
  providerName: '',
};

export function useVoiceCommands(
  shouldListen: boolean,
  preference?: STTProviderName,
): VoiceCommandState {
  const [state, setState] = useState<VoiceCommandState>(idleState);
  const providerRef = useRef<ISTTProvider | null>(null);

  useEffect(() => {
    const provider = createSTTProvider(preference);
    providerRef.current = provider;

    setState((prev) => ({
      ...prev,
      isListening: shouldListen && provider.isAvailable(),
      providerName: provider.name,
    }));

    if (!provider.isAvailable()) return;

    const unsubResult = provider.onResult((text) => {
      setState((prev) => ({ ...prev, lastText: text }));
    });

    const unsubError = provider.onError((error) => {
      console.error(`[STT:${provider.name}]`, error);
    });

    if (shouldListen) {
      provider.start({ language: 'pt-BR', continuous: false });
    }

    return () => {
      provider.stop();
      unsubResult();
      unsubError();
      providerRef.current = null;
    };
  }, [shouldListen, preference]);

  return state;
}

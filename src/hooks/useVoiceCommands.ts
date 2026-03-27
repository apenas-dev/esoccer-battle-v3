import { useEffect, useState, useRef, useCallback } from 'react';
import { createSTTProvider, type STTBackend } from '../services/stt';
import type { AppConfig } from '../types';

export interface VoiceCommandState {
  lastText: string;
  isListening: boolean;
  providerName: string;
}

const idleState: VoiceCommandState = {
  lastText: '',
  isListening: false,
  providerName: '',
};

export function useVoiceCommands(
  backend?: STTBackend,
  config?: AppConfig,
): VoiceCommandState & {
  startListening: () => Promise<void>;
  stopListening: () => Promise<string>;
} {
  const [state, setState] = useState<VoiceCommandState>(idleState);
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const providerRef = useRef<any>(null);
  const readyRef = useRef(false);

  useEffect(() => {
    let cancelled = false;

    if (!config) return;

    createSTTProvider(backend ?? 'auto', config).then((provider) => {
      if (cancelled) return;
      providerRef.current = provider;
      readyRef.current = true;
      setState((prev) => ({ ...prev, providerName: provider.name }));

      provider.onStatusChange = (status) => {
        setState((prev) => ({
          ...prev,
          isListening: status === 'listening',
        }));
      };
    }).catch((err) => {
      console.error('[STT] Provider creation failed:', err);
    });

    return () => {
      cancelled = true;
      readyRef.current = false;
      providerRef.current?.cancel();
      providerRef.current = null;
    };
  }, [backend, config]);

  const startListening = useCallback(async () => {
    const provider = providerRef.current;
    if (!provider || !readyRef.current) return;

    await provider.start();
  }, []);

  const stopListening = useCallback(async (): Promise<string> => {
    const provider = providerRef.current;
    if (!provider) return '';

    try {
      const text = await provider.stop();
      setState((prev) => ({ ...prev, lastText: text }));
      return text;
    } catch {
      return '';
    }
  }, []);

  return { ...state, startListening, stopListening };
}

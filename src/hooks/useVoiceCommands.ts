import { useEffect, useState, useRef, useCallback } from 'react';
import { createSTTProvider, type STTProviderName } from '../services/stt';

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
  preference?: STTProviderName,
): VoiceCommandState & {
  startListening: () => void;
  stopListening: () => Promise<string>;
} {
  const [state, setState] = useState<VoiceCommandState>(idleState);
  const providerRef = useRef<ReturnType<typeof createSTTProvider> | null>(null);
  const resolveRef = useRef<((text: string) => void) | null>(null);

  // Create provider once on mount
  useEffect(() => {
    providerRef.current = createSTTProvider(preference);
    setState((prev) => ({
      ...prev,
      providerName: providerRef.current!.name,
    }));
    return () => {
      providerRef.current?.stop();
      providerRef.current = null;
    };
  }, [preference]);

  const startListening = useCallback(() => {
    const provider = providerRef.current;
    if (!provider || !provider.isAvailable()) return;

    // Subscribe to results
    const unsubResult = provider.onResult((text, isFinal) => {
      if (isFinal && resolveRef.current) {
        resolveRef.current(text);
        resolveRef.current = null;
      }
      if (isFinal) {
        setState((prev) => ({ ...prev, lastText: text }));
      }
    });

    const unsubError = provider.onError((error) => {
      console.error(`[STT:${provider.name}]`, error);
      // On error, resolve with empty so the stop promise doesn't hang
      if (resolveRef.current) {
        resolveRef.current('');
        resolveRef.current = null;
      }
    });

    provider.start({ language: 'pt-BR', continuous: false });

    setState((prev) => ({ ...prev, isListening: true }));

    // Store unsub functions for cleanup on stop
    providerRef.current = Object.assign(provider, { _unsubs: [unsubResult, unsubError] } as never);
  }, []);

  const stopListening = useCallback((): Promise<string> => {
    return new Promise((resolve) => {
      const provider = providerRef.current;
      if (!provider) {
        resolve('');
        return;
      }

      resolveRef.current = resolve;

      // Set a timeout so the promise doesn't hang forever if no final result arrives
      const timeout = setTimeout(() => {
        if (resolveRef.current) {
          resolveRef.current('');
          resolveRef.current = null;
        }
      }, 2000);

      // Wrap resolve to also clear the timeout
      const originalResolve = resolve;
      resolveRef.current = (text: string) => {
        clearTimeout(timeout);
        originalResolve(text);
      };

      // Stop the provider — callbacks stay alive so final results can resolve the promise
      provider.stop();

      // Clean up subscriptions after stopping
      const extended = provider as unknown as { _unsubs?: Array<() => void> };
      extended._unsubs?.forEach((fn) => fn());

      setState((prev) => ({ ...prev, isListening: false }));
    });
  }, []);

  return { ...state, startListening, stopListening };
}

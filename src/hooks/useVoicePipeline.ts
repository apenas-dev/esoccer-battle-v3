import { useEffect, useState, useRef, useCallback } from 'react';
import { createSTTProvider, type ISTTProvider, type STTBackend } from '../services/stt';
import type { AppConfig } from '../types';

export interface UseVoicePipeline {
  isListening: boolean;
  isProcessing: boolean;
  lastCommand: string | null;
  error: string | null;
  startListening: () => void;
  stopListening: () => void;
  setProvider: (provider: ISTTProvider) => void;
}

interface UseVoicePipelineOptions {
  backend?: STTBackend;
  config: AppConfig;
  onCommand?: (text: string) => void;
}

export function useVoicePipeline(options: UseVoicePipelineOptions): UseVoicePipeline {
  const { backend, config, onCommand } = options;

  const [isListening, setIsListening] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false);
  const [lastCommand, setLastCommand] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const providerRef = useRef<ISTTProvider | null>(null);
  const onCommandRef = useRef(onCommand);

  useEffect(() => {
    onCommandRef.current = onCommand;
  }, [onCommand]);

  useEffect(() => {
    let cancelled = false;

    createSTTProvider(backend ?? 'auto', config).then((provider) => {
      if (cancelled) return;
      providerRef.current = provider;
      provider.onStatusChange = (status) => {
        if (status === 'listening') { setIsListening(true); setIsProcessing(false); }
        else if (status === 'processing') { setIsListening(false); setIsProcessing(true); }
        else { setIsListening(false); setIsProcessing(false); }
      };
    }).catch((err) => {
      if (!cancelled) setError(err.message);
    });

    return () => {
      cancelled = true;
      providerRef.current?.cancel();
    };
  }, [backend, config]);

  const setProvider = useCallback((provider: ISTTProvider) => {
    providerRef.current?.cancel();
    providerRef.current = provider;
    setIsListening(false);
    setIsProcessing(false);
    setError(null);
  }, []);

  const startListening = useCallback(() => {
    const provider = providerRef.current;
    if (!provider) { setError('No STT provider available'); return; }

    setError(null);
    provider.start().catch((err) => {
      setError(err.message);
      setIsListening(false);
    });
  }, []);

  const stopListening = useCallback(async () => {
    const provider = providerRef.current;
    if (!provider) return;

    try {
      const text = await provider.stop();
      if (text) {
        setLastCommand(text);
        onCommandRef.current?.(text);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsListening(false);
      setIsProcessing(false);
    }
  }, []);

  return { isListening, isProcessing, lastCommand, error, startListening, stopListening, setProvider };
}

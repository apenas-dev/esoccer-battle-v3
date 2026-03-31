import { useState, useCallback, useRef } from 'react';
import type { VoiceStatus } from '../types';
import type { ISTTProvider } from '../services/stt/ISTTProvider';

interface UseVoicePipelineReturn {
  voiceStatus: VoiceStatus;
  isListening: boolean;
  lastTranscript: string | null;
  lastError: string | null;
  startListening: () => Promise<void>;
  stopListening: () => Promise<void>;
}

interface UseVoicePipelineOptions {
  provider: ISTTProvider;
  onTranscript: (text: string) => Promise<void>;
  onError?: (error: string) => void;
}

export function useVoicePipeline({ provider, onTranscript, onError }: UseVoicePipelineOptions): UseVoicePipelineReturn {
  const [voiceStatus, setVoiceStatus] = useState<VoiceStatus>('idle');
  const [isListening, setIsListening] = useState(false);
  const [lastTranscript, setLastTranscript] = useState<string | null>(null);
  const [lastError, setLastError] = useState<string | null>(null);
  const providerRef = useRef(provider);
  providerRef.current = provider;

  // Wire up provider status callbacks
  provider.onStatusChange = (status: 'idle' | 'listening' | 'processing') => {
    if (status === 'idle') setVoiceStatus('idle');
    else if (status === 'listening') setVoiceStatus('listening');
    else if (status === 'processing') setVoiceStatus('processing');
  };

  const startListening = useCallback(async () => {
    try {
      setLastError(null);
      setVoiceStatus('listening');
      setIsListening(true);
      await providerRef.current.start();
    } catch (e) {
      const msg = String(e);
      setLastError(msg);
      setVoiceStatus('error');
      setIsListening(false);
      onError?.(msg);
    }
  }, [onError]);

  const stopListening = useCallback(async () => {
    try {
      setVoiceStatus('processing');
      const transcript = await providerRef.current.stop();
      setIsListening(false);
      setLastTranscript(transcript);
      setVoiceStatus('idle');
      if (transcript && transcript.trim()) {
        await onTranscript(transcript.trim());
      }
    } catch (e) {
      const msg = String(e);
      setLastError(msg);
      setVoiceStatus('error');
      onError?.(msg);
      setIsListening(false);
    }
  }, [onTranscript, onError]);

  return { voiceStatus, isListening, lastTranscript, lastError, startListening, stopListening };
}

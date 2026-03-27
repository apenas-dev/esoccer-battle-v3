import { useState, useCallback, useRef, useEffect } from 'react';
import type { VoiceStatus } from '../types';
import type { ISTTProvider } from '../services/stt/ISTTProvider';

interface UseVoicePipelineReturn {
  voiceStatus: VoiceStatus;
  isListening: boolean;
  lastTranscript: string | null;
  lastError: string | null;
  startListening: () => Promise<void>;
  stopListening: () => Promise<void>;
  cancelListening: () => void;
}

interface UseVoicePipelineOptions {
  provider: ISTTProvider;
  onTranscript: (text: string) => Promise<void>;
  onError?: (error: string) => void;
}

export function useVoicePipeline(options: UseVoicePipelineOptions): UseVoicePipelineReturn {
  const { provider, onTranscript, onError } = options;
  const [voiceStatus, setVoiceStatus] = useState<VoiceStatus>('idle');
  const [lastTranscript, setLastTranscript] = useState<string | null>(null);
  const [lastError, setLastError] = useState<string | null>(null);
  const activeRef = useRef(false);

  // Wire provider status changes to React state
  useEffect(() => {
    provider.onStatusChange = (status) => {
      if (status === 'listening') {
        setVoiceStatus('listening');
      } else if (status === 'processing') {
        setVoiceStatus('processing');
      } else {
        setVoiceStatus('idle');
      }
    };
    return () => {
      provider.onStatusChange = undefined;
    };
  }, [provider]);

  const startListening = useCallback(async () => {
    try {
      setLastError(null);
      activeRef.current = true;
      await provider.start();
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setLastError(msg);
      setVoiceStatus('error');
      onError?.(msg);
    }
  }, [provider, onError]);

  const stopListening = useCallback(async () => {
    if (!activeRef.current) return;
    activeRef.current = false;

    try {
      const transcript = await provider.stop();
      setVoiceStatus('idle');
      if (transcript) {
        setLastTranscript(transcript);
        await onTranscript(transcript);
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setLastError(msg);
      setVoiceStatus('error');
      onError?.(msg);
    }
  }, [provider, onTranscript, onError]);

  const cancelListening = useCallback(() => {
    activeRef.current = false;
    provider.cancel();
    setVoiceStatus('idle');
  }, [provider]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (activeRef.current) {
        provider.cancel();
      }
    };
  }, [provider]);

  return {
    voiceStatus,
    isListening: voiceStatus === 'listening',
    lastTranscript,
    lastError,
    startListening,
    stopListening,
    cancelListening,
  };
}

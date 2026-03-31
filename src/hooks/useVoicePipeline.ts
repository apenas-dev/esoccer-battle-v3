import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { VoiceStatus } from '../types';

interface UseVoicePipelineReturn {
  voiceStatus: VoiceStatus;
  isListening: boolean;
  lastTranscript: string | null;
  lastError: string | null;
  startListening: () => Promise<void>;
  stopListening: () => Promise<void>;
}

interface UseVoicePipelineOptions {
  onTranscript: (text: string) => Promise<void>;
  onError?: (error: string) => void;
}

export function useVoicePipeline({ onTranscript, onError }: UseVoicePipelineOptions): UseVoicePipelineReturn {
  const [voiceStatus, setVoiceStatus] = useState<VoiceStatus>('idle');
  const [isListening, setIsListening] = useState(false);
  const [lastTranscript, setLastTranscript] = useState<string | null>(null);
  const [lastError, setLastError] = useState<string | null>(null);

  const startListening = useCallback(async () => {
    try {
      setLastError(null);
      setVoiceStatus('listening');
      setIsListening(true);
      await invoke('start_listening');
    } catch (e) {
      const msg = String(e);
      setLastError(msg);
      setVoiceStatus('error');
      onError?.(msg);
    }
  }, [onError]);

  const stopListening = useCallback(async () => {
    try {
      setVoiceStatus('processing');
      const transcript = await invoke<string>('stop_listening');
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

import { useState, useCallback, useRef, useEffect } from 'react';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { VoiceStatus } from '../types';
import type { ISTTProvider } from '../services/stt/ISTTProvider';

interface UseVoicePipelineReturn {
  voiceStatus: VoiceStatus;
  isListening: boolean;
  lastTranscript: string | null;
  lastError: string | null;
  /** BUG 2 FIX: Whether the last voice command executed successfully. */
  lastCommandSuccess: boolean;
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
  const [lastCommandSuccess, setLastCommandSuccess] = useState(true);
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
    // BUG 1 FIX: Guard against rapid repeated clicks
    if (voiceStatus === 'listening' || voiceStatus === 'processing') return;
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
  }, [provider, onError, voiceStatus]);

  const stopListening = useCallback(async () => {
    if (!activeRef.current) return;
    activeRef.current = false;

    try {
      const transcript = await provider.stop();
      setVoiceStatus('idle');
      if (transcript) {
        setLastTranscript(transcript);
        try {
          await onTranscript(transcript);
          setLastCommandSuccess(true);
        } catch {
          setLastCommandSuccess(false);
        }
      }
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      setLastError(msg);
      setVoiceStatus('error');
      setLastCommandSuccess(false);
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

  // Listen for backend transcription-result event (async transcription)
  useEffect(() => {
    let unlisten: UnlistenFn | undefined;
    listen<{ text: string; success: boolean }>('transcription-result', (e) => {
      if (e.payload.text) {
        setLastTranscript(e.payload.text);
        setLastCommandSuccess(e.payload.success);
      }
    }).then((fn) => {
      unlisten = fn;
    });
    return () => {
      unlisten?.();
    };
  }, []);

  return {
    voiceStatus,
    isListening: voiceStatus === 'listening',
    lastTranscript,
    lastError,
    lastCommandSuccess,
    startListening,
    stopListening,
    cancelListening,
  };
}

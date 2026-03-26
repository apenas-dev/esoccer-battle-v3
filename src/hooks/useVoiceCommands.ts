import { useEffect, useRef, useState } from 'react';
import { onVoiceText, onCommandUnknown } from '../lib/tauri';

export interface VoiceCommandState {
  lastText: string;
  lastUnknownText: string;
  isListening: boolean;
}

export function useVoiceCommands(isListening: boolean) {
  const [state, setState] = useState<VoiceCommandState>({
    lastText: '',
    lastUnknownText: '',
    isListening: false,
  });
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    setState((prev) => ({ ...prev, isListening }));

    const unsubs: Promise<() => void>[] = [
      onVoiceText(({ text }) => {
        if (mountedRef.current) setState((prev) => ({ ...prev, lastText: text }));
      }),
      onCommandUnknown(({ text }) => {
        if (mountedRef.current) setState((prev) => ({ ...prev, lastUnknownText: text }));
      }),
    ];

    return () => {
      mountedRef.current = false;
      unsubs.forEach((p) => p.then((fn) => fn()));
    };
  }, [isListening]);

  return state;
}

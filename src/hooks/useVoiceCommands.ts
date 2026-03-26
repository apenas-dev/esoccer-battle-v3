import { useEffect, useRef, useState } from 'react';
import { onVoiceText, onCommandUnknown, isTauri } from '../lib/tauri';

export interface VoiceCommandState {
  lastText: string;
  lastUnknownText: string;
  isListening: boolean;
}

const idleState: VoiceCommandState = {
  lastText: '',
  lastUnknownText: '',
  isListening: false,
};

export function useVoiceCommands(_isListening: boolean): VoiceCommandState {
  const [state, setState] = useState<VoiceCommandState>(idleState);
  const mountedRef = useRef(true);

  useEffect(() => {
    if (!isTauri()) return;

    mountedRef.current = true;
    setState((prev) => ({ ...prev, isListening: _isListening }));

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
  }, [_isListening]);

  return state;
}

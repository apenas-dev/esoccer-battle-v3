export interface STTConfig {
  language: string;
  continuous?: boolean;
}

export interface ISTTProvider {
  readonly name: string;
  isAvailable(): boolean;
  start(config: STTConfig): void;
  stop(): void;
  onResult(cb: (text: string, isFinal: boolean) => void): () => void;
  onError(cb: (error: Error) => void): () => void;
}

export type STTProviderName = 'auto' | 'web-speech' | 'whisper';

export const STT_PROVIDER_KEY = 'stt_provider';

export function getSTTPreference(): STTProviderName {
  const stored = localStorage.getItem(STT_PROVIDER_KEY);
  if (stored === 'web-speech' || stored === 'whisper') return stored;
  return 'auto';
}

export function setSTTPreference(name: STTProviderName): void {
  localStorage.setItem(STT_PROVIDER_KEY, name);
}

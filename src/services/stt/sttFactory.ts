import type { ISTTProvider } from './ISTTProvider';
import { WhisperProvider } from './WhisperProvider';
import { WebSpeechProvider } from './WebSpeechProvider';

export type STTBackend = 'auto' | 'web-speech' | 'whisper';

export async function createSTTProvider(backend: STTBackend, language: string = 'pt-BR'): Promise<ISTTProvider> {
  if (backend === 'whisper') return new WhisperProvider();
  
  if (backend === 'web-speech') {
    const webSpeech = new WebSpeechProvider(language);
    if (await webSpeech.isAvailable()) return webSpeech;
    throw new Error('Web Speech API not available in this browser');
  }

  // Auto: try web-speech first, fall back to whisper
  const webSpeech = new WebSpeechProvider(language);
  if (await webSpeech.isAvailable()) return webSpeech;
  return new WhisperProvider();
}

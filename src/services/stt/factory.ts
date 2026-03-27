import { WebSpeechProvider } from './web-speech';
import { WhisperProvider } from './whisper';
import type { ISTTProvider, STTProviderName } from './types';
import { getSTTPreference } from './types';

export function createSTTProvider(preference?: STTProviderName): ISTTProvider {
  const pref = preference ?? getSTTPreference();

  // Create fresh instances each call to avoid stale internal state
  const webSpeech = new WebSpeechProvider();
  const whisper = new WhisperProvider();

  if (pref === 'whisper') return whisper;
  if (pref === 'web-speech' && webSpeech.isAvailable()) return webSpeech;
  if (pref === 'auto' && webSpeech.isAvailable()) return webSpeech;

  return whisper;
}

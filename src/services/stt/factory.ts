import { WebSpeechProvider } from './web-speech';
import { WhisperProvider } from './whisper';
import type { ISTTProvider, STTProviderName } from './types';
import { getSTTPreference } from './types';

const webSpeech = new WebSpeechProvider();
const whisper = new WhisperProvider();

export function createSTTProvider(preference?: STTProviderName): ISTTProvider {
  const pref = preference ?? getSTTPreference();

  if (pref === 'whisper') return whisper;
  if (pref === 'web-speech' && webSpeech.isAvailable()) return webSpeech;
  if (pref === 'auto' && webSpeech.isAvailable()) return webSpeech;

  return whisper;
}

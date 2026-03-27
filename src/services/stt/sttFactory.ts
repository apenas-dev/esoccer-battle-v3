import type { ISTTProvider } from './ISTTProvider';
import { WebSpeechProvider } from './WebSpeechProvider';
import { WhisperProvider } from './WhisperProvider';
import type { AppConfig } from '../../types';

export type STTBackend = 'auto' | 'web-speech' | 'whisper';

export async function createSTTProvider(
  backend: STTBackend,
  config: AppConfig,
): Promise<ISTTProvider> {
  const langMap: Record<string, string> = {
    pt_br: 'pt-BR',
    en: 'en-US',
    es: 'es-ES',
  };

  const lang = langMap[config.language] || 'pt-BR';

  if (backend === 'whisper') {
    return new WhisperProvider(config.whisper_model, lang);
  }

  if (backend === 'web-speech') {
    return new WebSpeechProvider(lang);
  }

  // 'auto' — try WebSpeech first, fallback to Whisper
  const webSpeech = new WebSpeechProvider(lang);
  if (await webSpeech.isAvailable()) {
    return webSpeech;
  }

  return new WhisperProvider(config.whisper_model, lang);
}

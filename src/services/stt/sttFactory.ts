// =============================================================================
// STT Factory — Creates the best available ISTTProvider
// Responsibility: Auto-detect and instantiate the correct STT backend
// Dependencies: ISTTProvider, WebSpeechProvider, WhisperProvider, types
// =============================================================================

import type { ISTTProvider } from './ISTTProvider';
import { WebSpeechProvider } from './WebSpeechProvider';
import { WhisperProvider } from './WhisperProvider';
import type { AppConfig, Language } from '../../types';

export type STTBackend = 'auto' | 'web-speech' | 'whisper';

/** Language code mapping for STT providers */
function toLanguageCode(lang: Language): string {
  switch (lang) {
    case 'pt_br': return 'pt-BR';
    case 'en': return 'en-US';
    case 'es': return 'es-ES';
  }
}

/**
 * Create an ISTTProvider based on backend preference and config.
 *
 * - `'auto'` → tries WebSpeech first, falls back to Whisper
 * - `'web-speech'` → WebSpeech only (throws if unavailable)
 * - `'whisper'` → Whisper only (throws if unavailable)
 */
export async function createSTTProvider(
  backend: STTBackend,
  config: AppConfig,
): Promise<ISTTProvider> {
  const language = toLanguageCode(config.language);

  // Always create both so we can probe availability
  const webSpeech = new WebSpeechProvider(language);
  const whisper = new WhisperProvider(config.whisper_model, language);

  const webAvailable = await webSpeech.isAvailable();
  const whisperAvailable = await whisper.isAvailable();

  switch (backend) {
    case 'web-speech':
      if (!webAvailable) throw new Error('Web Speech API is not available in this browser');
      return webSpeech;

    case 'whisper':
      if (!whisperAvailable) throw new Error('Whisper backend is not available (not running in Tauri)');
      return whisper;

    case 'auto':
    default:
      if (webAvailable) return webSpeech;
      if (whisperAvailable) return whisper;
      throw new Error('No STT provider available (Web Speech API not found and not running in Tauri)');
  }
}

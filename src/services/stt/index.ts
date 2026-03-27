// =============================================================================
// STT Service — Public API barrel
// =============================================================================

export type { ISTTProvider } from './ISTTProvider';
export { WebSpeechProvider } from './WebSpeechProvider';
export { WhisperProvider } from './WhisperProvider';
export { createSTTProvider } from './sttFactory';
export type { STTBackend } from './sttFactory';

// Legacy re-exports (kept for existing consumers until they migrate)
export type { STTProviderName, STTConfig } from './types';
export { getSTTPreference, setSTTPreference } from './types';

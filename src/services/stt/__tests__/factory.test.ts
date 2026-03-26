import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock tauri before any STT imports
vi.mock('../../../lib/tauri', () => ({
  isTauri: () => false,
  startRecording: vi.fn().mockResolvedValue(undefined),
  stopRecordingAndTranscribe: vi.fn().mockResolvedValue(undefined),
  onVoiceText: vi.fn().mockResolvedValue(() => {}),
  onCommandUnknown: vi.fn().mockResolvedValue(() => {}),
}));

describe('WebSpeechProvider', () => {
  let provider: WebSpeechProvider;

  beforeEach(async () => {
    vi.resetModules();
    const { WebSpeechProvider: WS } = await import('../web-speech');
    provider = new WS();
  });

  it('has name "web-speech"', () => {
    expect(provider.name).toBe('web-speech');
  });

  it('isAvailable returns false in jsdom (no SpeechRecognition)', () => {
    expect(provider.isAvailable()).toBe(false);
  });

  it('onResult returns unsubscribe function', () => {
    const unsub = provider.onResult(() => {});
    expect(typeof unsub).toBe('function');
  });

  it('onError returns unsubscribe function', () => {
    const unsub = provider.onError(() => {});
    expect(typeof unsub).toBe('function');
  });

  it('start does not throw when SpeechRecognition unavailable', () => {
    expect(() => provider.start({ language: 'pt-BR' })).not.toThrow();
  });

  it('stop does not throw when not started', () => {
    expect(() => provider.stop()).not.toThrow();
  });
});

describe('WhisperProvider', () => {
  let provider: WhisperProvider;

  beforeEach(async () => {
    vi.resetModules();
    const { WhisperProvider: WP } = await import('../whisper');
    provider = new WP();
  });

  it('has name "whisper"', () => {
    expect(provider.name).toBe('whisper');
  });

  it('isAvailable returns false when not in Tauri', () => {
    expect(provider.isAvailable()).toBe(false);
  });

  it('onResult returns unsubscribe function', () => {
    const unsub = provider.onResult(() => {});
    expect(typeof unsub).toBe('function');
  });

  it('onError returns unsubscribe function', () => {
    const unsub = provider.onError(() => {});
    expect(typeof unsub).toBe('function');
  });

  it('double start is idempotent', () => {
    provider.start({ language: 'pt-BR' });
    provider.start({ language: 'pt-BR' }); // should not throw
    provider.stop();
  });

  it('stop when not started is safe', () => {
    expect(() => provider.stop()).not.toThrow();
  });
});

describe('STT Factory', () => {
  let createSTTProvider: typeof import('../factory').createSTTProvider;

  beforeEach(async () => {
    vi.resetModules();
    const mod = await import('../factory');
    createSTTProvider = mod.createSTTProvider;
  });

  it('returns whisper when preference is "whisper"', () => {
    const provider = createSTTProvider('whisper');
    expect(provider.name).toBe('whisper');
  });

  it('returns whisper when preference is "auto" and Web Speech unavailable', () => {
    const provider = createSTTProvider('auto');
    expect(provider.name).toBe('whisper');
  });

  it('returns whisper when preference is "web-speech" but unavailable', () => {
    const provider = createSTTProvider('web-speech');
    expect(provider.name).toBe('whisper');
  });

  it('fallback works when no preference given', () => {
    const provider = createSTTProvider();
    expect(provider.name).toBe('whisper');
  });
});

describe('STT types helpers', () => {
  beforeEach(async () => {
    vi.resetModules();
    localStorage.clear();
  });

  afterEach(() => {
    localStorage.clear();
  });

  it('getSTTPreference defaults to auto', async () => {
    const { getSTTPreference } = await import('../types');
    expect(getSTTPreference()).toBe('auto');
  });

  it('getSTTPreference returns stored whisper value', async () => {
    const { getSTTPreference, setSTTPreference } = await import('../types');
    setSTTPreference('whisper');
    expect(getSTTPreference()).toBe('whisper');
  });

  it('setSTTPreference works for web-speech', async () => {
    const { getSTTPreference, setSTTPreference } = await import('../types');
    setSTTPreference('web-speech');
    expect(getSTTPreference()).toBe('web-speech');
  });
});

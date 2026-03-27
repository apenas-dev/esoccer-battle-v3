import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock @tauri-apps/api before any STT imports
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

describe('WebSpeechProvider', () => {
  let provider: InstanceType<typeof import('../WebSpeechProvider').WebSpeechProvider>;

  beforeEach(async () => {
    vi.resetModules();
    const { WebSpeechProvider: WS } = await import('../WebSpeechProvider');
    provider = new WS('en-US');
  });

  it('has name "web-speech"', () => {
    expect(provider.name).toBe('web-speech');
  });

  it('isAvailable returns false in jsdom (no SpeechRecognition)', async () => {
    expect(await provider.isAvailable()).toBe(false);
  });

  it('start rejects when SpeechRecognition unavailable', async () => {
    await expect(provider.start()).rejects.toThrow('Web Speech API not available');
  });

  it('stop returns empty string when not started', async () => {
    const text = await provider.stop();
    expect(text).toBe('');
  });

  it('cancel does not throw when not started', () => {
    expect(() => provider.cancel()).not.toThrow();
  });
});

describe('WhisperProvider', () => {
  let provider: InstanceType<typeof import('../WhisperProvider').WhisperProvider>;

  beforeEach(async () => {
    vi.resetModules();
    const { WhisperProvider: WP } = await import('../WhisperProvider');
    provider = new WP('base', 'pt-BR');
  });

  it('has name "whisper"', () => {
    expect(provider.name).toBe('whisper');
  });

  it('isAvailable returns false when not in Tauri', async () => {
    expect(await provider.isAvailable()).toBe(false);
  });

  it('stop returns empty when not started', async () => {
    const text = await provider.stop();
    expect(text).toBe('');
  });

  it('cancel does not throw when not started', () => {
    expect(() => provider.cancel()).not.toThrow();
  });
});

describe('STT Factory', () => {
  let createSTTProvider: typeof import('../sttFactory').createSTTProvider;

  const mockConfig = {
    mic_device: null,
    whisper_model: 'base' as const,
    language: 'pt_br' as const,
    voice_threshold: 0.5,
    team_a_name: 'A',
    team_b_name: 'B',
    theme: 'dark' as const,
    match_duration_secs: 300,
    timer_mode: 'countdown' as const,
    volume: 0.8,
  };

  beforeEach(async () => {
    vi.resetModules();
    const mod = await import('../sttFactory');
    createSTTProvider = mod.createSTTProvider;
  });

  it('returns whisper when backend is "whisper" and in Tauri', async () => {
    // Mock Tauri environment
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    try {
      const provider = await createSTTProvider('whisper', mockConfig);
      expect(provider.name).toBe('whisper');
    } finally {
      delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    }
  });

  it('returns whisper when backend is "auto" and Web Speech unavailable', async () => {
    (globalThis as Record<string, unknown>).__TAURI_INTERNALS__ = {};
    try {
      const provider = await createSTTProvider('auto', mockConfig);
      expect(provider.name).toBe('whisper');
    } finally {
      delete (globalThis as Record<string, unknown>).__TAURI_INTERNALS__;
    }
  });

  it('throws when no provider is available', async () => {
    await expect(createSTTProvider('auto', mockConfig)).rejects.toThrow('No STT provider available');
  });

  it('throws when web-speech requested but unavailable', async () => {
    await expect(createSTTProvider('web-speech', mockConfig)).rejects.toThrow('Web Speech API');
  });

  it('throws when whisper requested but not in Tauri', async () => {
    await expect(createSTTProvider('whisper', mockConfig)).rejects.toThrow('Whisper backend');
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

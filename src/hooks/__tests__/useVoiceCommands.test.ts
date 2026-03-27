import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';

// Mock @tauri-apps/api before any STT imports
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));

const mockProvider = {
  name: 'mock-provider',
  isAvailable: vi.fn(() => Promise.resolve(true)),
  start: vi.fn(() => Promise.resolve()),
  stop: vi.fn(() => Promise.resolve('gol do time a')),
  cancel: vi.fn(),
  onStatusChange: undefined as ((status: 'idle' | 'listening' | 'processing') => void) | undefined,
};

vi.mock('../../services/stt', () => ({
  createSTTProvider: vi.fn(() => Promise.resolve(mockProvider)),
}));

import { useVoiceCommands } from '../useVoiceCommands';
import { createSTTProvider } from '../../services/stt';
import type { VoiceCommandState } from '../useVoiceCommands';

const mockedCreateSTTProvider = vi.mocked(createSTTProvider);

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

describe('useVoiceCommands', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockProvider.start.mockResolvedValue(undefined);
    mockProvider.stop.mockResolvedValue('gol do time a');
    mockProvider.cancel.mockClear();
    mockProvider.onStatusChange = undefined;
  });

  it('creates provider on mount with config', async () => {
    await act(async () => {
      renderHook(() => useVoiceCommands('auto', mockConfig));
    });
    expect(mockedCreateSTTProvider).toHaveBeenCalledWith('auto', mockConfig);
  });

  it('returns correct initial state', async () => {
    let state!: VoiceCommandState & { startListening: () => Promise<void>; stopListening: () => Promise<string> };
    await act(async () => {
      const { result } = renderHook(() => useVoiceCommands('auto', mockConfig));
      state = result.current;
    });
    expect(state.lastText).toBe('');
    expect(state.isListening).toBe(false);
    expect(state.providerName).toBe('mock-provider');
    expect(typeof state.startListening).toBe('function');
    expect(typeof state.stopListening).toBe('function');
  });

  it('startListening calls provider.start', async () => {
    const { result } = renderHook(() => useVoiceCommands('auto', mockConfig));
    await act(async () => {
      await result.current.startListening();
    });
    expect(mockProvider.start).toHaveBeenCalled();
  });

  it('stopListening returns transcript', async () => {
    const { result } = renderHook(() => useVoiceCommands('auto', mockConfig));
    let text = '';
    await act(async () => {
      text = await result.current.stopListening();
    });
    expect(text).toBe('gol do time a');
    expect(mockProvider.stop).toHaveBeenCalled();
  });

  it('stopListening returns empty on error', async () => {
    mockProvider.stop.mockRejectedValueOnce(new Error('mic error'));
    const { result } = renderHook(() => useVoiceCommands('auto', mockConfig));
    let text = '';
    await act(async () => {
      text = await result.current.stopListening();
    });
    expect(text).toBe('');
  });

  it('cancel on unmount', async () => {
    const { unmount } = renderHook(() => useVoiceCommands('auto', mockConfig));
    await act(async () => {
      // Wait for the provider to be created
    });
    act(() => {
      unmount();
    });
    expect(mockProvider.cancel).toHaveBeenCalled();
  });

  it('onStatusChange updates isListening', async () => {
    const { result } = renderHook(() => useVoiceCommands('auto', mockConfig));
    await act(async () => {
      // Wait for provider init
    });

    act(() => {
      mockProvider.onStatusChange?.('listening');
    });
    expect(result.current.isListening).toBe(true);

    act(() => {
      mockProvider.onStatusChange?.('idle');
    });
    expect(result.current.isListening).toBe(false);
  });
});

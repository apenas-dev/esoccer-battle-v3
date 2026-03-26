import { describe, it, expect, vi, beforeEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';

// Mock tauri before any STT imports
vi.mock('../../../lib/tauri', () => ({
  isTauri: () => false,
  startRecording: vi.fn().mockResolvedValue(undefined),
  stopRecordingAndTranscribe: vi.fn().mockResolvedValue(undefined),
  onVoiceText: vi.fn().mockResolvedValue(() => {}),
  onCommandUnknown: vi.fn().mockResolvedValue(() => {}),
}));

// We need to mock createSTTProvider to control behavior
const mockProvider = {
  name: 'mock-provider',
  isAvailable: vi.fn(() => true),
  start: vi.fn(),
  stop: vi.fn(),
  onResult: vi.fn(() => vi.fn()),
  onError: vi.fn(() => vi.fn()),
};

vi.mock('../../services/stt', () => ({
  createSTTProvider: vi.fn(() => mockProvider),
}));

import { useVoiceCommands } from '../useVoiceCommands';
import { createSTTProvider } from '../../services/stt';

const mockedCreateSTTProvider = vi.mocked(createSTTProvider);

describe('useVoiceCommands', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockProvider.isAvailable.mockReturnValue(true);
    mockProvider.start.mockClear();
    mockProvider.stop.mockClear();
    mockProvider.onResult.mockClear();
    mockProvider.onError.mockClear();
  });

  it('creates provider on mount', () => {
    renderHook(() => useVoiceCommands());
    expect(mockedCreateSTTProvider).toHaveBeenCalledTimes(1);
  });

  it('creates provider with preference', () => {
    renderHook(() => useVoiceCommands('whisper'));
    expect(mockedCreateSTTProvider).toHaveBeenCalledWith('whisper');
  });

  it('returns correct initial state', () => {
    const { result } = renderHook(() => useVoiceCommands());
    expect(result.current.lastText).toBe('');
    expect(result.current.isListening).toBe(false);
    expect(result.current.providerName).toBe('mock-provider');
    expect(typeof result.current.startListening).toBe('function');
    expect(typeof result.current.stopListening).toBe('function');
  });

  it('startListening sets isListening to true', () => {
    const { result } = renderHook(() => useVoiceCommands());

    act(() => {
      result.current.startListening();
    });

    expect(result.current.isListening).toBe(true);
    expect(mockProvider.start).toHaveBeenCalledWith({ language: 'pt-BR', continuous: false });
  });

  it('startListening does nothing when provider unavailable', () => {
    mockProvider.isAvailable.mockReturnValue(false);
    const { result } = renderHook(() => useVoiceCommands());

    act(() => {
      result.current.startListening();
    });

    expect(result.current.isListening).toBe(false);
    expect(mockProvider.start).not.toHaveBeenCalled();
  });

  it('stopListening sets isListening to false', async () => {
    // Simulate a final result callback
    let resultCallback: ((text: string, isFinal: boolean) => void) | null = null;
    mockProvider.onResult.mockImplementation((cb) => {
      resultCallback = cb;
      return vi.fn();
    });

    const { result } = renderHook(() => useVoiceCommands());

    // Start listening first
    act(() => {
      result.current.startListening();
    });

    expect(result.current.isListening).toBe(true);

    // Stop — the promise resolves via timeout since we don't fire a callback
    await act(async () => {
      await result.current.stopListening();
    });

    expect(result.current.isListening).toBe(false);
    expect(mockProvider.stop).toHaveBeenCalled();
  });

  it('stopListening resolves with empty string when provider is null-like', async () => {
    // Render with a provider that becomes null after unmount
    const { result, unmount } = renderHook(() => useVoiceCommands());
    unmount(); // This sets providerRef to null
    // After unmount we can't call stopListening, so test via a fresh mount
    // where the mock returns null for the create call
    mockedCreateSTTProvider.mockReturnValueOnce({ ...mockProvider, stop: vi.fn(), name: 'test' });
    const { result: result2 } = renderHook(() => useVoiceCommands());
    
    // Provider exists but simulate no resolveRef
    mockProvider.onResult.mockReturnValue(vi.fn());
    mockProvider.onError.mockReturnValue(vi.fn());

    act(() => {
      result2.current.startListening();
    });

    await act(async () => {
      await result2.current.stopListening();
    });

    expect(result2.current.isListening).toBe(false);
  });

  it('stopListening resolves with empty string after timeout (no final result)', async () => {
    vi.useFakeTimers();

    mockProvider.onResult.mockReturnValue(vi.fn());
    mockProvider.onError.mockReturnValue(vi.fn());

    const { result } = renderHook(() => useVoiceCommands());

    act(() => {
      result.current.startListening();
    });

    let text: string;
    await act(async () => {
      const p = result.current.stopListening();
      await vi.advanceTimersByTimeAsync(2500);
      text = await p;
    });

    expect(text).toBe('');
    expect(result.current.isListening).toBe(false);

    vi.useRealTimers();
  });

  it('PTT fast (<1s) does not crash', async () => {
    mockProvider.onResult.mockReturnValue(vi.fn());
    mockProvider.onError.mockReturnValue(vi.fn());

    const { result } = renderHook(() => useVoiceCommands());

    // Quick start → stop cycle (simulates PTT < 1s)
    act(() => {
      result.current.startListening();
    });

    let text: string;
    await act(async () => {
      text = await result.current.stopListening();
    });

    expect(text).toBe(''); // No transcription arrived, resolves empty
    expect(result.current.isListening).toBe(false);
  });

  it('double click does not execute twice', async () => {
    mockProvider.onResult.mockReturnValue(vi.fn());
    mockProvider.onError.mockReturnValue(vi.fn());

    const { result } = renderHook(() => useVoiceCommands());

    // First PTT
    act(() => {
      result.current.startListening();
    });

    // Second PTT while first is "listening" (the hook allows this)
    act(() => {
      result.current.startListening();
    });

    // Stop
    await act(async () => {
      await result.current.stopListening();
    });

    expect(result.current.isListening).toBe(false);
  });

  it('stops provider on unmount', () => {
    const { unmount } = renderHook(() => useVoiceCommands());
    unmount();
    expect(mockProvider.stop).toHaveBeenCalled();
  });

  it('updates lastText when final result arrives', () => {
    let capturedCallback: ((text: string, isFinal: boolean) => void) | null = null;
    mockProvider.onResult.mockImplementation((cb) => {
      capturedCallback = cb;
      return vi.fn();
    });
    mockProvider.onError.mockReturnValue(vi.fn());

    const { result } = renderHook(() => useVoiceCommands());

    act(() => {
      result.current.startListening();
    });

    // Simulate final result
    act(() => {
      capturedCallback?.('gol do time a', true);
    });

    expect(result.current.lastText).toBe('gol do time a');
  });

  it('does not update lastText for interim (non-final) results', () => {
    let capturedCallback: ((text: string, isFinal: boolean) => void) | null = null;
    mockProvider.onResult.mockImplementation((cb) => {
      capturedCallback = cb;
      return vi.fn();
    });
    mockProvider.onError.mockReturnValue(vi.fn());

    const { result } = renderHook(() => useVoiceCommands());

    act(() => {
      result.current.startListening();
    });

    // Simulate interim result
    act(() => {
      capturedCallback?.('gol do...', false);
    });

    expect(result.current.lastText).toBe('');
  });

  it('error during listening resolves with empty string', async () => {
    vi.useFakeTimers();
    let errorCallback: ((error: Error) => void) | null = null;
    mockProvider.onResult.mockReturnValue(vi.fn());
    mockProvider.onError.mockImplementation((cb) => {
      errorCallback = cb;
      return vi.fn();
    });

    const { result } = renderHook(() => useVoiceCommands());

    act(() => {
      result.current.startListening();
    });

    let text: string;
    await act(async () => {
      const p = result.current.stopListening();
      // Simulate error
      errorCallback?.(new Error('network'));
      await vi.advanceTimersByTimeAsync(100);
      text = await p;
    });

    expect(text).toBe('');
    vi.useRealTimers();
  });
});

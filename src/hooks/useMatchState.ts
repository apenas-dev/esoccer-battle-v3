import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { UnlistenFn } from '@tauri-apps/api/event';
import type { MatchState, AppConfig, ScoreChangedPayload, TimeUpdatedPayload } from '../types';
import { formatTime } from '../lib/utils';

interface UseMatchStateReturn {
  state: MatchState | null;
  config: AppConfig | null;
  isLoading: boolean;
  displayTime: string;
  executeCommand: (text: string) => Promise<void>;
  resetMatch: () => Promise<void>;
  loadConfig: () => Promise<void>;
  updateConfig: (cfg: AppConfig) => Promise<void>;
}

const DEFAULT_STATE: MatchState = {
  phase: 'idle',
  sub_phase: 'normal',
  config: {
    team_a_name: 'Time A',
    team_b_name: 'Time B',
    duration_secs: 600,
    timer_mode: 'countdown',
  },
  score_a: 0,
  score_b: 0,
  elapsed_secs: 0,
  started_at: null,
  paused_elapsed_secs: 0,
  match_id: '',
};

export function useMatchState(): UseMatchStateReturn {
  const [state, setState] = useState<MatchState | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  // Initial load
  const loadState = useCallback(async () => {
    try {
      const s = await invoke<MatchState>('get_state');
      setState(s ?? DEFAULT_STATE);
    } catch {
      setState(DEFAULT_STATE);
    }
  }, []);

  const loadConfig = useCallback(async () => {
    try {
      const c = await invoke<AppConfig>('get_config');
      setConfig(c);
    } catch (e) {
      console.error('Failed to load config:', e);
    }
  }, []);

  useEffect(() => {
    let unlisteners: UnlistenFn[] = [];

    (async () => {
      await Promise.all([loadState(), loadConfig()]);
      setIsLoading(false);

      const un1 = await listen<ScoreChangedPayload>('score-changed', (e) => {
        setState((prev) => (prev ? { ...prev, score_a: e.payload.score_a, score_b: e.payload.score_b } : prev));
      });
      const un2 = await listen<{ phase: string; sub_phase?: string }>('phase-changed', (e) => {
        setState((prev) =>
          prev
            ? {
                ...prev,
                phase: e.payload.phase as MatchState['phase'],
                sub_phase: (e.payload.sub_phase as MatchState['sub_phase']) ?? prev.sub_phase,
              }
            : prev,
        );
      });
      const un3 = await listen<TimeUpdatedPayload>('time-updated', (e) => {
        setState((prev) => (prev ? { ...prev, elapsed_secs: e.payload.elapsed_secs } : prev));
      });
      const un4 = await listen<{ score_a: number; score_b: number }>('match-finished', (e) => {
        setState((prev) =>
          prev
            ? {
                ...prev,
                phase: 'finished',
                score_a: e.payload.score_a,
                score_b: e.payload.score_b,
              }
            : prev,
        );
      });

      // BUG 6 FIX: Removed local timer (setInterval). Backend is the single source of truth.
      // Time is updated via 'time-updated' events from the backend (un3 above).

      unlisteners = [un1, un2, un3, un4];
    })();

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, [loadState, loadConfig]);

  // Compute displayTime
  const displayTime = (() => {
    if (!state) return '00:00';
    if (state.config.timer_mode === 'countdown') {
      const remaining = Math.max(0, state.config.duration_secs - state.elapsed_secs);
      return formatTime(remaining);
    }
    return formatTime(state.elapsed_secs);
  })();

  const executeCommand = useCallback(async (text: string) => {
    try {
      const newState = await invoke<MatchState>('execute_command', { text });
      if (newState) setState(newState);
    } catch (e) {
      console.error('Command failed:', e);
    }
  }, []);

  const resetMatch = useCallback(async () => {
    try {
      await invoke('reset_match');
      await loadState();
    } catch (e) {
      console.error('Reset failed:', e);
    }
  }, [loadState]);

  const updateConfig = useCallback(async (cfg: AppConfig) => {
    try {
      await invoke('update_config', { newConfig: cfg });
      setConfig(cfg);
    } catch (e) {
      console.error('Update config failed:', e);
    }
  }, []);

  return {
    state,
    config,
    isLoading,
    displayTime,
    executeCommand,
    resetMatch,
    loadConfig,
    updateConfig,
  };
}

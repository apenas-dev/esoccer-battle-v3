import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type { MatchState, AppConfig, GamePhase } from '../types';

export function useMatchState() {
  const [state, setState] = useState<MatchState | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [displayTime, setDisplayTime] = useState('00:00');

  const formatTime = (secs: number): string => {
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  };

  useEffect(() => {
    const init = async () => {
      try {
        const [matchState, appConfig] = await Promise.all([
          invoke<MatchState>('get_state'),
          invoke<AppConfig>('get_config'),
        ]);
        setState(matchState);
        setConfig(appConfig);
        setDisplayTime(formatTime(matchState.elapsed_secs));
      } catch (e) {
        console.error('Failed to load state:', e);
      } finally {
        setIsLoading(false);
      }
    };
    init();
  }, []);

  // Timer interval
  useEffect(() => {
    if (!state || state.phase !== 'playing') return;
    const interval = setInterval(() => {
      setState(prev => {
        if (!prev) return prev;
        const next = prev.elapsed_secs + 1;
        setDisplayTime(formatTime(next));
        return { ...prev, elapsed_secs: next };
      });
    }, 1000);
    return () => clearInterval(interval);
  }, [state?.phase]);

  // Listen to Tauri events
  useEffect(() => {
    const unlisteners: Array<() => void> = [];

    const setup = async () => {
      unlisteners.push(
        await listen<GamePhase>('phase-changed', (e) => {
          setState(prev => prev ? { ...prev, phase: e.payload } : prev);
        }),
        await listen<{ score_a: number; score_b: number }>('score-changed', (e) => {
          setState(prev => prev ? { ...prev, ...e.payload } : prev);
        }),
        await listen<string>('timer-control', () => {
          // Timer start/stop handled by interval
        }),
      );
    };
    setup();
    return () => unlisteners.forEach(u => u());
  }, []);

  const executeCommand = async (text: string) => {
    try {
      const newState = await invoke<MatchState>('execute_command', { text });
      setState(newState);
      setDisplayTime(formatTime(newState.elapsed_secs));
    } catch (e) {
      console.error('Command failed:', e);
    }
  };

  const loadConfig = async () => {
    try {
      const c = await invoke<AppConfig>('get_config');
      setConfig(c);
    } catch (e) {
      console.error('Failed to load config:', e);
    }
  };

  const updateConfig = async (newConfig: AppConfig) => {
    try {
      await invoke('update_config', { newConfig });
      setConfig(newConfig);
    } catch (e) {
      console.error('Failed to save config:', e);
    }
  };

  return { state, config, isLoading, displayTime, executeCommand, loadConfig, updateConfig };
}

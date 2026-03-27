// =============================================================================
// E-Soccer Battle V3 — useMatchState Hook
// Responsibility: Sync match state via Tauri events + local timer (ADR-007)
// SRP: Match state management + backend communication ONLY
// =============================================================================

import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  MatchState,
  AppConfig,
  ScoreChangedPayload,
  PhaseChangedPayload,
  CommandLogEntry,
} from '../types';

// --- Public Return Type ---

export interface UseMatchStateReturn {
  /** Current match state (null until loaded) */
  state: MatchState | null;
  /** App config (null until loaded) */
  config: AppConfig | null;
  /** Whether initial state is being fetched */
  isLoading: boolean;
  /** Formatted timer display string (MM:SS) */
  displayTime: string;
  /** Command execution log */
  commandLog: CommandLogEntry[];
  /** Execute any text command via backend */
  executeCommand: (text: string) => Promise<void>;
  /** Start a new match */
  startMatch: () => Promise<void>;
  /** Pause the match */
  pauseMatch: () => Promise<void>;
  /** Resume from pause */
  resumeMatch: () => Promise<void>;
  /** End the match (goes to Finished) */
  endMatch: () => Promise<void>;
  /** Score goal for team A */
  goalA: () => Promise<void>;
  /** Score goal for team B */
  goalB: () => Promise<void>;
  /** Trigger challenge / doubt (sub_phase -> Challenge) */
  doubt: () => Promise<void>;
  /** Resolve challenge (sub_phase -> Normal) */
  resolve: () => Promise<void>;
  /** Volta seis / 6 metros (sub_phase -> Normal) */
  voltaSeis: () => Promise<void>;
  /** Reset to Idle (new match_id) */
  reset: () => Promise<void>;
  /** Reload state from backend */
  reloadState: () => Promise<void>;
  /** Load app config */
  loadConfig: () => Promise<void>;
  /** Update app config */
  updateConfig: (cfg: AppConfig) => Promise<void>;
}

// --- Timer formatting ---

function formatTime(elapsedSecs: number, durationSecs: number, mode: 'countdown' | 'countup'): string {
  if (mode === 'countup') {
    const m = Math.floor(elapsedSecs / 60);
    const s = elapsedSecs % 60;
    return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
  }
  // countdown
  const remaining = Math.max(0, durationSecs - elapsedSecs);
  const m = Math.floor(remaining / 60);
  const s = remaining % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

// --- Commands map ---

function commandToName(text: string): string {
  return text.toLowerCase().trim();
}

// --- Main Hook ---

export function useMatchState(): UseMatchStateReturn {
  const [state, setState] = useState<MatchState | null>(null);
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [commandLog, setCommandLog] = useState<CommandLogEntry[]>([]);

  const timerRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const elapsedRef = useRef(0);
  const mountedRef = useRef(true);

  // --- Timer Control (ADR-007: frontend-owned timer) ---

  const stopTimer = useCallback(() => {
    if (timerRef.current !== null) {
      clearInterval(timerRef.current);
      timerRef.current = null;
    }
  }, []);

  const startTimer = useCallback(() => {
    stopTimer();
    timerRef.current = setInterval(() => {
      elapsedRef.current += 1;
      if (mountedRef.current) {
        setState((prev) => {
          if (!prev) return prev;
          return { ...prev, elapsed_secs: elapsedRef.current };
        });
      }
    }, 1000);
  }, [stopTimer]);

  // Sync elapsed ref from state
  useEffect(() => {
    if (state) {
      elapsedRef.current = state.elapsed_secs;
    }
  }, [state?.phase]); // sync on phase changes

  // --- Send command to backend ---

  const sendCommand = useCallback(
    async (command: string, source: 'voice' | 'button' = 'button') => {
      const entry: CommandLogEntry = {
        id: crypto.randomUUID(),
        timestamp: new Date(),
        command,
        source,
        success: true,
      };
      try {
        await invoke('process_command', { command });
      } catch {
        entry.success = false;
      }
      setCommandLog((prev) => [entry, ...prev].slice(0, 100));
    },
    [],
  );

  // --- Specific command functions ---

  const startMatch = useCallback(() => sendCommand('start_match'), [sendCommand]);
  const pauseMatch = useCallback(() => sendCommand('pause'), [sendCommand]);
  const resumeMatch = useCallback(() => sendCommand('resume'), [sendCommand]);
  const endMatch = useCallback(() => sendCommand('end'), [sendCommand]);
  const goalA = useCallback(() => sendCommand('goal_a'), [sendCommand]);
  const goalB = useCallback(() => sendCommand('goal_b'), [sendCommand]);
  const doubt = useCallback(() => sendCommand('doubt'), [sendCommand]);
  const resolve = useCallback(() => sendCommand('resolve'), [sendCommand]);
  const voltaSeis = useCallback(() => sendCommand('volta_seis'), [sendCommand]);
  const reset = useCallback(() => sendCommand('reset'), [sendCommand]);
  const executeCommand = useCallback(
    (text: string) => sendCommand(commandToName(text), 'voice'),
    [sendCommand],
  );

  // --- Load initial state ---

  const reloadState = useCallback(async () => {
    try {
      const s = (await invoke<MatchState>('get_state')) as MatchState;
      if (mountedRef.current) {
        setState(s);
        elapsedRef.current = s.elapsed_secs;
      }
    } catch {
      // State may not exist yet (no match started)
    }
  }, []);

  const loadConfig = useCallback(async () => {
    try {
      const c = (await invoke<AppConfig>('get_config')) as AppConfig;
      if (mountedRef.current) setConfig(c);
    } catch {
      // Config may not exist yet
    }
  }, []);

  const updateConfig = useCallback(async (cfg: AppConfig) => {
    await invoke('update_config', { config: cfg });
    if (mountedRef.current) setConfig(cfg);
  }, []);

  // --- Mount: load state + listen to Tauri events ---

  useEffect(() => {
    mountedRef.current = true;

    (async () => {
      await Promise.all([reloadState(), loadConfig()]);
      if (mountedRef.current) setIsLoading(false);
    })();

    const cleanups: UnlistenFn[] = [];

    // Listen to phase-changed → manage local timer
    listen<PhaseChangedPayload>('phase-changed', (event) => {
      const { phase } = event.payload;
      if (mountedRef.current) {
        setState((prev) => (prev ? { ...prev, phase, sub_phase: event.payload.sub_phase } : prev));
      }
      if (phase === 'playing') {
        startTimer();
      } else {
        stopTimer();
      }
    }).then((fn) => cleanups.push(fn));

    // Listen to score-changed
    listen<ScoreChangedPayload>('score-changed', (event) => {
      const { score_a, score_b } = event.payload;
      if (mountedRef.current) {
        setState((prev) => (prev ? { ...prev, score_a, score_b } : prev));
      }
    }).then((fn) => cleanups.push(fn));

    // Listen to match-state-changed (full state sync)
    listen<MatchState>('match-state-changed', (event) => {
      if (mountedRef.current) {
        const newState = event.payload;
        setState(newState);
        elapsedRef.current = newState.elapsed_secs;

        // Sync timer to new phase
        if (newState.phase === 'playing') {
          startTimer();
        } else {
          stopTimer();
        }
      }
    }).then((fn) => cleanups.push(fn));

    // Listen to timer-control from backend (start/stop)
    listen<'start' | 'stop'>('timer-control', (event) => {
      if (event.payload === 'start') {
        startTimer();
      } else {
        stopTimer();
      }
    }).then((fn) => cleanups.push(fn));

    return () => {
      mountedRef.current = false;
      stopTimer();
      cleanups.forEach((fn) => fn());
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // --- Computed display time ---

  const displayTime = formatTime(
    state?.elapsed_secs ?? 0,
    state?.config.duration_secs ?? 300,
    state?.config.timer_mode ?? 'countdown',
  );

  return {
    state,
    config,
    isLoading,
    displayTime,
    commandLog,
    executeCommand,
    startMatch,
    pauseMatch,
    resumeMatch,
    endMatch,
    goalA,
    goalB,
    doubt,
    resolve,
    voltaSeis,
    reset,
    reloadState,
    loadConfig,
    updateConfig,
  };
}

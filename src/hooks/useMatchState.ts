import { useCallback, useEffect, useRef, useState } from 'react';
import {
  type MatchState,
  getMatchState,
  onMatchStateChanged,
  onTimerTick,
  startMatch as tauriStartMatch,
  endMatch as tauriEndMatch,
  goalA as tauriGoalA,
  goalB as tauriGoalB,
  restart as tauriRestart,
  challenge as tauriChallenge,
  resolveChallenge as tauriResolveChallenge,
} from '../lib/tauri';

const INITIAL_STATE: MatchState = {
  status: 'idle',
  score_a: 0,
  score_b: 0,
  elapsed_seconds: 0,
  team_a_name: 'Time A',
  team_b_name: 'Time B',
  last_command: null,
};

export function useMatchState() {
  const [matchState, setMatchState] = useState<MatchState>(INITIAL_STATE);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;

    const unsubs: Promise<() => void>[] = [
      onMatchStateChanged((state) => {
        if (mountedRef.current) setMatchState(state);
      }),
      onTimerTick(({ elapsed_seconds }) => {
        if (mountedRef.current)
          setMatchState((prev) => ({ ...prev, elapsed_seconds }));
      }),
    ];

    // Fetch initial state
    getMatchState()
      .then((s) => {
        if (mountedRef.current) setMatchState(s);
      })
      .catch(() => {});

    return () => {
      mountedRef.current = false;
      unsubs.forEach((p) => p.then((fn) => fn()));
    };
  }, []);

  const startMatch = useCallback(() => { tauriStartMatch(); }, []);
  const endMatch = useCallback(() => { tauriEndMatch(); }, []);
  const goalA = useCallback(() => { tauriGoalA(); }, []);
  const goalB = useCallback(() => { tauriGoalB(); }, []);
  const restart = useCallback(() => { tauriRestart(); }, []);
  const challenge = useCallback(() => { tauriChallenge(); }, []);
  const resolveChallenge = useCallback(() => { tauriResolveChallenge(); }, []);

  return { matchState, startMatch, endMatch, goalA, goalB, restart, challenge, resolveChallenge };
}

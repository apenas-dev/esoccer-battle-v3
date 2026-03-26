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
  challenge as tauriChallenge,
  resolveChallenge as tauriResolveChallenge,
  pauseMatch as tauriPauseMatch,
  resumeMatch as tauriResumeMatch,
  undoGoal as tauriUndoGoal,
  setScoreA as tauriSetScoreA,
  setScoreB as tauriSetScoreB,
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
  const [toast, setToast] = useState<string | null>(null);
  const mountedRef = useRef(true);

  const showToast = useCallback((msg: string) => {
    setToast(msg);
    setTimeout(() => setToast(null), 3000);
  }, []);

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

  const startMatch = useCallback(() => { tauriStartMatch().catch(e => showToast(String(e))); }, [showToast]);
  const endMatch = useCallback(() => { tauriEndMatch().catch(e => showToast(String(e))); }, [showToast]);
  const goalA = useCallback(() => { tauriGoalA().catch(e => showToast(String(e))); }, [showToast]);
  const goalB = useCallback(() => { tauriGoalB().catch(e => showToast(String(e))); }, [showToast]);
  const challenge = useCallback(() => { tauriChallenge().catch(e => showToast(String(e))); }, [showToast]);
  const resolveChallenge = useCallback(() => { tauriResolveChallenge().catch(e => showToast(String(e))); }, [showToast]);
  const pauseMatch = useCallback(() => { tauriPauseMatch().catch(e => showToast(String(e))); }, [showToast]);
  const resumeMatch = useCallback(() => { tauriResumeMatch().catch(e => showToast(String(e))); }, [showToast]);
  const undoGoal = useCallback(() => { tauriUndoGoal().catch(e => showToast(String(e))); }, [showToast]);
  const setScoreA = useCallback((score: number) => { tauriSetScoreA(score).catch(e => showToast(String(e))); }, [showToast]);
  const setScoreB = useCallback((score: number) => { tauriSetScoreB(score).catch(e => showToast(String(e))); }, [showToast]);

  return { matchState, toast, showToast, startMatch, endMatch, goalA, goalB, challenge, resolveChallenge, pauseMatch, resumeMatch, undoGoal, setScoreA, setScoreB };
}

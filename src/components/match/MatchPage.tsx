import { cn } from "../../lib/cn";
import { useCallback, useEffect, useState, type HTMLAttributes } from 'react';
import { motion } from 'framer-motion';
import { type MatchStatus } from '../../lib/types';

import { Scoreboard } from './Scoreboard';
import { MatchTimer } from './MatchTimer';
import { VoiceIndicator, type VoiceState } from './VoiceIndicator';
import { CommandLog, type CommandEntry } from './CommandLog';
import { MatchControls } from './MatchControls';

function generateId(): string {
  return (typeof crypto !== 'undefined' && crypto.randomUUID)
    ? crypto.randomUUID()
    : `cmd-${Date.now()}-${Math.random().toString(36).slice(2)}`;
}

export interface MatchPageProps extends HTMLAttributes<HTMLDivElement> {
  teamAName?: string;
  teamBName?: string;
}

export function MatchPage({ teamAName = 'Time A', teamBName = 'Time B', className, ...props }: MatchPageProps) {
  const [status, setStatus] = useState<MatchStatus>('idle');
  const [scoreA, setScoreA] = useState(0);
  const [scoreB, setScoreB] = useState(0);
  const [elapsed, setElapsed] = useState(0);
  const [timerRunning, setTimerRunning] = useState(false);
  const [voiceState, setVoiceState] = useState<VoiceState>('idle');
  const [commands, setCommands] = useState<CommandEntry[]>([]);

  useEffect(() => {
    if (!timerRunning) return;
    const id = setInterval(() => setElapsed((s) => s + 1), 1000);
    return () => clearInterval(id);
  }, [timerRunning]);

  useEffect(() => {
    setVoiceState(status === 'playing' ? 'listening' : (status === 'idle' || status === 'finished') ? 'idle' : voiceState === 'listening' ? 'idle' : voiceState);
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [status]);

  const addCmd = useCallback((text: string, type: CommandEntry['type'] = 'control') => {
    setCommands((p) => [{ id: generateId(), text, timestamp: new Date(), type }, ...p]);
  }, []);

  const handleStart = useCallback(() => {
    if (status === 'finished' || status === 'idle') { setScoreA(0); setScoreB(0); setElapsed(0); }
    setStatus('playing'); setTimerRunning(true); setVoiceState('listening');
    addCmd('Partida iniciada');
  }, [status, addCmd]);

  const handlePause = useCallback(() => { setStatus('paused'); setTimerRunning(false); setVoiceState('idle'); addCmd('Partida pausada'); }, [addCmd]);
  const handleResume = useCallback(() => { setStatus('playing'); setTimerRunning(true); setVoiceState('listening'); addCmd('Partida retomada'); }, [addCmd]);
  const handleEnd = useCallback(() => { setStatus('finished'); setTimerRunning(false); setVoiceState('idle'); addCmd('Partida encerrada'); }, [addCmd]);

  const handleGoalA = useCallback(() => { setScoreA((s) => s + 1); addCmd(`⚽ Gol do ${teamAName}`, 'goal'); }, [teamAName, addCmd]);
  const handleGoalB = useCallback(() => { setScoreB((s) => s + 1); addCmd(`⚽ Gol do ${teamBName}`, 'goal'); }, [teamBName, addCmd]);

  const handleUndo = useCallback(() => { addCmd('↩ Volta seis'); }, [addCmd]);

  const handleChallenge = useCallback(() => {
    setStatus('challenge');
    addCmd('❓ Dúvida / Contestação', 'challenge');
    setTimeout(() => {
      setStatus((prev: MatchStatus) => prev === 'challenge' ? 'playing' : prev);
      setTimerRunning(true);
      setVoiceState('listening');
    }, 5000);
  }, [addCmd]);

  const handleResolveChallenge = useCallback(() => {
    setStatus('playing');
    setTimerRunning(true);
    setVoiceState('listening');
    addCmd('✅ Dúvida resolvida', 'challenge');
  }, [addCmd]);

  return (
    <div className={cn('min-h-screen bg-[#0a0f1a] text-white flex flex-col items-center px-4 py-6 sm:px-6 sm:py-8', className)} {...props}>
      <motion.header initial={{ opacity: 0, y: -20 }} animate={{ opacity: 1, y: 0 }} className="w-full max-w-3xl mb-6 sm:mb-8">
        <h1 className="text-lg sm:text-xl font-bold text-center tracking-tight">
          <span className="text-[#00ff88]">E-Soccer</span>{' '}
          <span className="text-gray-400">Battle</span>
        </h1>
      </motion.header>

      <main className="w-full max-w-3xl flex flex-col items-center gap-6 sm:gap-8">
        <section aria-label="Placar">
          <Scoreboard teamAName={teamAName} teamBName={teamBName} scoreA={scoreA} scoreB={scoreB} status={status}
            onScoreAChange={() => handleGoalA()} onScoreBChange={() => handleGoalB()} />
        </section>
        <section aria-label="Cronômetro"><MatchTimer elapsedSeconds={elapsed} isRunning={timerRunning} /></section>
        <section aria-label="Indicador de voz"><VoiceIndicator voiceState={voiceState} /></section>
        <section aria-label="Log de comandos"><CommandLog commands={commands} maxEntries={5} /></section>
        <section aria-label="Controles manuais">
          <MatchControls status={status} onStart={handleStart} onPause={handlePause} onResume={handleResume}
            onEnd={handleEnd} onUndo={handleUndo} onChallenge={handleChallenge}
            onGoalA={handleGoalA} onGoalB={handleGoalB} teamAName={teamAName} teamBName={teamBName}
            onResolveChallenge={handleResolveChallenge} />
        </section>
      </main>

      <footer className="mt-auto pt-8 pb-2">
        <p className="text-xs text-gray-700 text-center">
          E-Soccer Battle V3 · Comandos de voz: iniciar, gol do time A/B, volta seis, dúvida, encerrar
        </p>
      </footer>
    </div>
  );
}

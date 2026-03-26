import { cn } from "../../lib/cn";
import { type HTMLAttributes, memo, useCallback, useEffect, useRef, useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { type MatchStatus } from '../../lib/types';

export type { MatchStatus } from '../../lib/types';

export interface ScoreboardProps extends HTMLAttributes<HTMLDivElement> {
  teamAName: string;
  teamBName: string;
  scoreA: number;
  scoreB: number;
  status: MatchStatus;
  onScoreAChange?: (newScore: number) => void;
  onScoreBChange?: (newScore: number) => void;
}

function ConfettiExplosion({ active, team }: { active: boolean; team: 'A' | 'B' }) {
  const [particles, setParticles] = useState<Array<{ id: number; color: string; delay: number; angle: number; distance: number }>>([]);

  useEffect(() => {
    if (!active) { setParticles([]); return; }
    const colors = team === 'A'
      ? ['#22d3ee', '#06b6d4', '#00ff88', '#fbbf24', '#fff']
      : ['#f87171', '#ef4444', '#fbbf24', '#00ff88', '#fff'];
    setParticles(Array.from({ length: 20 }, (_, i) => ({
      id: Date.now() + i,
      color: colors[Math.floor(Math.random() * colors.length)],
      delay: Math.random() * 0.3,
      angle: (Math.PI * 2 * i) / 20,
      distance: 80 + Math.random() * 120,
    })));
    const t = setTimeout(() => setParticles([]), 2000);
    return () => clearTimeout(t);
  }, [active, team]);

  if (!particles.length) return null;
  return (
    <div className="absolute inset-0 pointer-events-none overflow-hidden" aria-hidden="true">
      {particles.map((p) => (
        <motion.div key={p.id}
          initial={{ opacity: 1, scale: 0, x: '50%', y: '50%' }}
          animate={{ opacity: 0, scale: 1, x: `calc(50% + ${Math.cos(p.angle) * p.distance}px)`, y: `calc(50% + ${Math.sin(p.angle) * p.distance}px)` }}
          transition={{ duration: 1.2, delay: p.delay, ease: 'easeOut' }}
          className="absolute w-2 h-2 rounded-full" style={{ backgroundColor: p.color }} />
      ))}
    </div>
  );
}

function ScoreCell({ score, team, isEditing, onIncrement, onDecrement }: {
  score: number; team: 'A' | 'B'; isEditing: boolean;
  onIncrement: () => void; onDecrement: () => void;
}) {
  const accent = team === 'A' ? 'text-cyan-400' : 'text-red-400';
  const btnHover = team === 'A' ? 'hover:bg-cyan-900/40' : 'hover:bg-red-900/40';
  const glow = team === 'A' ? '0 0 30px rgba(34,211,238,0.4)' : '0 0 30px rgba(248,113,113,0.4)';

  return (
    <div className="flex flex-col items-center gap-1">
      {isEditing && (
        <div className="flex gap-2 mb-1">
          <button onClick={onDecrement} className={`w-8 h-8 rounded-full bg-gray-800 border border-gray-700 ${btnHover} text-gray-300 text-lg font-bold transition-colors flex items-center justify-center focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-400`} aria-label={`Diminuir gol time ${team}`}>−</button>
          <button onClick={onIncrement} className={`w-8 h-8 rounded-full bg-gray-800 border border-gray-700 ${btnHover} text-gray-300 text-lg font-bold transition-colors flex items-center justify-center focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-400`} aria-label={`Aumentar gol time ${team}`}>+</button>
        </div>
      )}
      <motion.div key={score}
        initial={{ scale: 1.4, opacity: 0.5 }} animate={{ scale: 1, opacity: 1 }}
        transition={{ type: 'spring', stiffness: 300, damping: 20 }}
        className={`text-[6rem] sm:text-[8rem] lg:text-[10rem] font-black leading-none tabular-nums ${accent}`}
        style={{ textShadow: glow }}
        aria-label={`Placar time ${team}: ${score} gols`} role="img">
        {score}
      </motion.div>
    </div>
  );
}

function StatusBadge({ status }: { status: MatchStatus }) {
  const cfg: Record<MatchStatus, { label: string; cls: string }> = {
    idle: { label: 'AGUARDANDO', cls: 'bg-gray-700 text-gray-400' },
    playing: { label: 'EM JOGO', cls: 'bg-emerald-900/60 text-[#00ff88] border border-emerald-500/30' },
    paused: { label: 'PAUSADO', cls: 'bg-amber-900/60 text-amber-400 border border-amber-500/30' },
    challenge: { label: 'DÚVIDA', cls: 'bg-violet-900/60 text-violet-400 border border-violet-500/30 animate-pulse' },
    finished: { label: 'ENCERRADO', cls: 'bg-blue-900/60 text-blue-400 border border-blue-500/30' },
  };
  const { label, cls } = cfg[status];
  const dotColor = status === 'playing' ? '#00ff88' : status === 'challenge' ? '#a78bfa' : null;
  return (
    <motion.div key={status} initial={{ opacity: 0, y: -8 }} animate={{ opacity: 1, y: 0 }}
      className={`inline-flex items-center gap-2 px-4 py-1.5 rounded-full text-sm font-bold uppercase tracking-widest ${cls}`}
      role="status" aria-label={`Status: ${label}`}>
      {dotColor && <span className="relative flex h-2 w-2">
        <span className="animate-ping absolute inline-flex h-full w-full rounded-full opacity-75" style={{ backgroundColor: dotColor }} />
        <span className="relative inline-flex rounded-full h-2 w-2" style={{ backgroundColor: dotColor }} />
      </span>}
      {label}
    </motion.div>
  );
}

function WinnerBanner({ scoreA, scoreB, teamAName, teamBName }: { scoreA: number; scoreB: number; teamAName: string; teamBName: string }) {
  const w = scoreA > scoreB ? teamAName : scoreB > scoreA ? teamBName : null;
  const accent = w === teamAName ? 'text-cyan-400' : w === teamBName ? 'text-red-400' : 'text-amber-400';
  return (
    <motion.div initial={{ opacity: 0, scale: 0.8 }} animate={{ opacity: 1, scale: 1 }}
      transition={{ type: 'spring', stiffness: 200, damping: 15 }}
      className={`text-2xl sm:text-3xl font-black ${accent} mt-2`} role="alert" aria-live="polite">
      {w ? `🏆 ${w} Venceu!` : '🤝 Empate!'}
    </motion.div>
  );
}

export const Scoreboard = memo(function Scoreboard({ teamAName, teamBName, scoreA, scoreB, status, onScoreAChange, onScoreBChange, className, ...props }: ScoreboardProps) {
  const [confettiTeam, setConfettiTeam] = useState<'A' | 'B' | null>(null);
  const prevA = useRef(scoreA);
  const prevB = useRef(scoreB);

  useEffect(() => {
    if (scoreA !== prevA.current) { setConfettiTeam('A'); prevA.current = scoreA; const t = setTimeout(() => setConfettiTeam(null), 2000); return () => clearTimeout(t); }
    if (scoreB !== prevB.current) { setConfettiTeam('B'); prevB.current = scoreB; const t = setTimeout(() => setConfettiTeam(null), 2000); return () => clearTimeout(t); }
  }, [scoreA, scoreB]);

  const isEditable = status === 'playing';
  const isChallenge = status === 'challenge';
  const incA = useCallback(() => onScoreAChange?.(scoreA + 1), [scoreA, onScoreAChange]);
  const decA = useCallback(() => onScoreAChange?.(Math.max(0, scoreA - 1)), [scoreA, onScoreAChange]);
  const incB = useCallback(() => onScoreBChange?.(scoreB + 1), [scoreB, onScoreBChange]);
  const decB = useCallback(() => onScoreBChange?.(Math.max(0, scoreB - 1)), [scoreB, onScoreBChange]);

  return (
    <div className={cn('relative w-full max-w-3xl mx-auto', className)} {...props}>
      <div className={`relative overflow-hidden rounded-2xl border p-6 sm:p-8 lg:p-10 transition-all duration-500 ${
        isChallenge ? 'bg-[#0d1117] border-violet-500/50 shadow-[0_0_40px_rgba(167,139,250,0.3)]'
          : status === 'finished' ? 'bg-[#0d1117] border-blue-500/30 shadow-[0_0_30px_rgba(96,165,250,0.2)]'
          : 'bg-[#0d1117] border-[#1e3a5f] shadow-[0_0_20px_rgba(0,255,136,0.15)]'
      }`} role="region" aria-label="Placar da partida">
        {confettiTeam && <ConfettiExplosion active team={confettiTeam} />}
        <AnimatePresence>
          {isChallenge && <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} exit={{ opacity: 0 }} className="absolute inset-0 bg-violet-500/5 pointer-events-none" aria-hidden="true" />}
        </AnimatePresence>
        <div className="flex justify-center mb-4"><StatusBadge status={status} /></div>
        <div className="flex items-center justify-center gap-4 sm:gap-8 lg:gap-16">
          <div className="flex flex-col items-center gap-2 flex-1 min-w-0">
            <h2 className="text-xl sm:text-2xl font-bold text-cyan-400 truncate max-w-[160px]" title={teamAName}>{teamAName}</h2>
            <ScoreCell score={scoreA} team="A" isEditing={isEditable} onIncrement={incA} onDecrement={decA} />
          </div>
          <span className="text-3xl sm:text-4xl font-black text-gray-600 select-none" aria-hidden="true">×</span>
          <div className="flex flex-col items-center gap-2 flex-1 min-w-0">
            <h2 className="text-xl sm:text-2xl font-bold text-red-400 truncate max-w-[160px]" title={teamBName}>{teamBName}</h2>
            <ScoreCell score={scoreB} team="B" isEditing={isEditable} onIncrement={incB} onDecrement={decB} />
          </div>
        </div>
        {status === 'finished' && <div className="flex justify-center mt-4"><WinnerBanner scoreA={scoreA} scoreB={scoreB} teamAName={teamAName} teamBName={teamBName} /></div>}
        {status === 'idle' && <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.5 }} className="flex justify-center mt-6"><p className="text-gray-500 text-sm">Fale <span className="text-[#00ff88] font-semibold">"iniciar partida"</span> ou clique em começar</p></motion.div>}
      </div>
    </div>
  );
});

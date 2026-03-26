import { cn } from "../../lib/cn";
import { type HTMLAttributes, memo } from 'react';
import { motion } from 'framer-motion';
import { type MatchStatus } from '../../lib/types';

export interface MatchControlsProps extends HTMLAttributes<HTMLDivElement> {
  status: MatchStatus;
  onStart?: () => void;
  onPause?: () => void;
  onResume?: () => void;
  onEnd?: () => void;
  onUndo?: () => void;
  onChallenge?: () => void;
  onResolveChallenge?: () => void;
  onGoalA?: () => void;
  onGoalB?: () => void;
  teamAName?: string;
  teamBName?: string;
}

const CtrlBtn = memo(function CtrlBtn({ label, icon, onClick, variant = 'default', disabled, ariaLabel }: {
  label: string; icon: React.ReactNode; onClick?: () => void;
  variant?: 'default' | 'primary' | 'danger' | 'warning' | 'goalA' | 'goalB' | 'resolve';
  disabled?: boolean; ariaLabel: string;
}) {
  const styles: Record<string, string> = {
    default: 'bg-gray-800 border-gray-700 text-gray-300 hover:bg-gray-700 hover:border-gray-600 focus-visible:ring-gray-500',
    primary: 'bg-emerald-900/50 border-emerald-500/30 text-[#00ff88] hover:bg-emerald-900/70 hover:border-emerald-500/50 focus-visible:ring-emerald-400',
    danger: 'bg-red-900/40 border-red-500/20 text-red-400 hover:bg-red-900/60 hover:border-red-500/40 focus-visible:ring-red-400',
    warning: 'bg-amber-900/40 border-amber-500/20 text-amber-400 hover:bg-amber-900/60 hover:border-amber-500/40 focus-visible:ring-amber-400',
    goalA: 'bg-cyan-900/40 border-cyan-500/20 text-cyan-400 hover:bg-cyan-900/60 hover:border-cyan-500/40 focus-visible:ring-cyan-400',
    goalB: 'bg-red-900/40 border-red-500/20 text-red-400 hover:bg-red-900/60 hover:border-red-500/40 focus-visible:ring-red-400',
    resolve: 'bg-violet-900/40 border-violet-500/20 text-violet-400 hover:bg-violet-900/60 hover:border-violet-500/40 focus-visible:ring-violet-400',
  };

  return (
    <motion.button whileHover={disabled ? {} : { scale: 1.05 }} whileTap={disabled ? {} : { scale: 0.95 }}
      onClick={onClick} disabled={disabled} aria-label={ariaLabel}
      className={`inline-flex items-center gap-2 px-4 py-2.5 rounded-xl border text-sm font-semibold transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0a0f1a] disabled:opacity-30 disabled:pointer-events-none ${styles[variant]}`}>
      <span className="text-base" aria-hidden="true">{icon}</span>
      <span>{label}</span>
    </motion.button>
  );
});

export const MatchControls = memo(function MatchControls({ status, onStart, onPause, onResume, onEnd, onUndo, onChallenge, onResolveChallenge, onGoalA, onGoalB, teamAName = 'Time A', teamBName = 'Time B', className, ...props }: MatchControlsProps) {
  const idle = status === 'idle';
  const playing = status === 'playing';
  const paused = status === 'paused';
  const challenge = status === 'challenge';
  const finished = status === 'finished';
  const canAct = playing || paused;

  return (
    <div className={cn('w-full max-w-2xl mx-auto', className)} role="toolbar" aria-label="Controles da partida" {...props}>
      <div className="flex flex-wrap items-center justify-center gap-2 sm:gap-3">
        {idle && <CtrlBtn label="Iniciar Partida" icon="▶" onClick={onStart} variant="primary" ariaLabel="Iniciar a partida" />}
        {playing && <CtrlBtn label="Pausar" icon="⏸" onClick={onPause} variant="warning" ariaLabel="Pausar a partida" />}
        {paused && <CtrlBtn label="Retomar" icon="▶" onClick={onResume} variant="primary" ariaLabel="Retomar a partida" />}
        {canAct && <CtrlBtn label="Volta Seis" icon="↩" onClick={onUndo} variant="default" ariaLabel="Desfazer último comando" />}
        {canAct && !challenge && <CtrlBtn label="Dúvida" icon="❓" onClick={onChallenge} variant="warning" ariaLabel="Acionar dúvida" />}
        {challenge && <CtrlBtn label="Resolver" icon="✅" onClick={onResolveChallenge} variant="resolve" ariaLabel="Resolver dúvida" />}
        {!idle && !finished && <CtrlBtn label="Encerrar" icon="⏹" onClick={onEnd} variant="danger" ariaLabel="Encerrar a partida" />}
        {finished && <CtrlBtn label="Nova Partida" icon="🔄" onClick={onStart} variant="primary" ariaLabel="Iniciar nova partida" />}
      </div>

      {canAct && (
        <motion.div initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.1 }}
          className="flex items-center justify-center gap-3 mt-3">
          <CtrlBtn label={`Gol ${teamAName}`} icon="⚽" onClick={onGoalA} variant="goalA" ariaLabel={`Gol para ${teamAName}`} />
          <CtrlBtn label={`Gol ${teamBName}`} icon="⚽" onClick={onGoalB} variant="goalB" ariaLabel={`Gol para ${teamBName}`} />
        </motion.div>
      )}

      <motion.p key={status} initial={{ opacity: 0 }} animate={{ opacity: 0.4 }} transition={{ delay: 0.5 }}
        className="text-center text-xs text-gray-600 mt-4">
        {idle && 'Comandos de voz disponíveis ao iniciar'}
        {playing && 'Mic ativo — diga "gol do time A" ou use os botões acima'}
        {paused && 'Partida pausada — diga "começar" ou clique em retomar'}
        {challenge && 'Dúvida registrada — clique em Resolver ou aguarde'}
        {finished && 'Partida encerrada — inicie uma nova partida'}
      </motion.p>
    </div>
  );
});

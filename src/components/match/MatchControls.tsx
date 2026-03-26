import { type HTMLAttributes } from 'react';
import { motion } from 'framer-motion';

function cn(...inputs: (string | undefined | false | null)[]) {
  return inputs.filter(Boolean).join(' ');
}

// ── Types ─────────────────────────────────────────────
type MatchStatus = 'idle' | 'playing' | 'paused' | 'challenge' | 'finished';

interface MatchControlsProps extends HTMLAttributes<HTMLDivElement> {
  /** Current match status */
  status: MatchStatus;
  /** Start/restart the match */
  onStart?: () => void;
  /** Pause the match */
  onPause?: () => void;
  /** Resume the match */
  onResume?: () => void;
  /** End the match */
  onEnd?: () => void;
  /** Trigger "volta seis" (undo last action) */
  onUndo?: () => void;
  /** Trigger "dúvida" (challenge) */
  onChallenge?: () => void;
  /** Goal for Team A */
  onGoalA?: () => void;
  /** Goal for Team B */
  onGoalB?: () => void;
  /** Team A display name */
  teamAName?: string;
  /** Team B display name */
  teamBName?: string;
}

// ── Button Variant ────────────────────────────────────
function ControlButton({
  label,
  icon,
  onClick,
  variant = 'default',
  disabled = false,
  ariaLabel,
}: {
  label: string;
  icon: React.ReactNode;
  onClick?: () => void;
  variant?: 'default' | 'primary' | 'danger' | 'warning' | 'goalA' | 'goalB';
  disabled?: boolean;
  ariaLabel: string;
}) {
  const variantStyles: Record<string, string> = {
    default: 'bg-gray-800 border-gray-700 text-gray-300 hover:bg-gray-700 hover:border-gray-600 focus-visible:ring-gray-500',
    primary: 'bg-emerald-900/50 border-emerald-500/30 text-[#00ff88] hover:bg-emerald-900/70 hover:border-emerald-500/50 focus-visible:ring-emerald-400',
    danger: 'bg-red-900/40 border-red-500/20 text-red-400 hover:bg-red-900/60 hover:border-red-500/40 focus-visible:ring-red-400',
    warning: 'bg-amber-900/40 border-amber-500/20 text-amber-400 hover:bg-amber-900/60 hover:border-amber-500/40 focus-visible:ring-amber-400',
    goalA: 'bg-cyan-900/40 border-cyan-500/20 text-cyan-400 hover:bg-cyan-900/60 hover:border-cyan-500/40 focus-visible:ring-cyan-400',
    goalB: 'bg-red-900/40 border-red-500/20 text-red-400 hover:bg-red-900/60 hover:border-red-500/40 focus-visible:ring-red-400',
  };

  return (
    <motion.button
      whileHover={disabled ? {} : { scale: 1.05 }}
      whileTap={disabled ? {} : { scale: 0.95 }}
      onClick={onClick}
      disabled={disabled}
      aria-label={ariaLabel}
      className={`
        inline-flex items-center gap-2 px-4 py-2.5 rounded-xl border
        text-sm font-semibold transition-colors duration-150
        focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-[#0a0f1a]
        disabled:opacity-30 disabled:pointer-events-none disabled:cursor-not-allowed
        ${variantStyles[variant]}
      `}
    >
      <span className="text-base" aria-hidden="true">{icon}</span>
      <span>{label}</span>
    </motion.button>
  );
}

// ── Component ─────────────────────────────────────────
export function MatchControls({
  status,
  onStart,
  onPause,
  onResume,
  onEnd,
  onUndo,
  onChallenge,
  onGoalA,
  onGoalB,
  teamAName = 'Time A',
  teamBName = 'Time B',
  className,
  ...props
}: MatchControlsProps) {
  const isIdle = status === 'idle';
  const isPlaying = status === 'playing';
  const isPaused = status === 'paused';
  const isChallenge = status === 'challenge';
  const isFinished = status === 'finished';
  const canAct = isPlaying || isPaused;

  return (
    <div
      className={cn('w-full max-w-2xl mx-auto', className)}
      role="toolbar"
      aria-label="Controles da partida"
      {...props}
    >
      {/* Main controls */}
      <div className="flex flex-wrap items-center justify-center gap-2 sm:gap-3">
        {/* Idle: Start button */}
        {isIdle && (
          <ControlButton
            label="Iniciar Partida"
            icon="▶"
            onClick={onStart}
            variant="primary"
            ariaLabel="Iniciar a partida"
          />
        )}

        {/* Playing/Paused: Core controls */}
        {isPlaying && (
          <ControlButton
            label="Pausar"
            icon="⏸"
            onClick={onPause}
            variant="warning"
            ariaLabel="Pausar a partida"
          />
        )}

        {isPaused && (
          <ControlButton
            label="Retomar"
            icon="▶"
            onClick={onResume}
            variant="primary"
            ariaLabel="Retomar a partida"
          />
        )}

        {/* Volta seis — available when playing or paused */}
        {canAct && (
          <ControlButton
            label="Volta Seis"
            icon="↩"
            onClick={onUndo}
            variant="default"
            ariaLabel="Desfazer último comando (volta seis)"
          />
        )}

        {/* Dúvida / Contestar */}
        {canAct && !isChallenge && (
          <ControlButton
            label="Dúvida"
            icon="❓"
            onClick={onChallenge}
            variant="warning"
            ariaLabel="Acionar dúvida ou contestação"
          />
        )}

        {/* Encerrar — available when not idle */}
        {!isIdle && !isFinished && (
          <ControlButton
            label="Encerrar"
            icon="⏹"
            onClick={onEnd}
            variant="danger"
            ariaLabel="Encerrar a partida"
          />
        )}

        {/* Finished: Restart */}
        {isFinished && (
          <ControlButton
            label="Nova Partida"
            icon="🔄"
            onClick={onStart}
            variant="primary"
            ariaLabel="Iniciar nova partida"
          />
        )}
      </div>

      {/* Goal buttons — only during gameplay */}
      {canAct && (
        <motion.div
          initial={{ opacity: 0, y: 10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
          className="flex items-center justify-center gap-3 mt-3"
        >
          <ControlButton
            label={`Gol ${teamAName}`}
            icon="⚽"
            onClick={onGoalA}
            variant="goalA"
            ariaLabel={`Registrar gol para ${teamAName}`}
          />
          <ControlButton
            label={`Gol ${teamBName}`}
            icon="⚽"
            onClick={onGoalB}
            variant="goalB"
            ariaLabel={`Registrar gol para ${teamBName}`}
          />
        </motion.div>
      )}

      {/* Hint text */}
      <motion.p
        key={status}
        initial={{ opacity: 0 }}
        animate={{ opacity: 0.4 }}
        transition={{ delay: 0.5 }}
        className="text-center text-xs text-gray-600 mt-4"
      >
        {isIdle && 'Comandos de voz disponíveis ao iniciar'}
        {isPlaying && 'Mic ativo — diga "gol do time A" ou use os botões acima'}
        {isPaused && 'Partida pausada — diga "começar" ou clique em retomar'}
        {isChallenge && 'Dúvida registrada — aguardando resolução'}
        {isFinished && 'Partida encerrada — inicie uma nova partida'}
      </motion.p>
    </div>
  );
}

export { type MatchControlsProps };

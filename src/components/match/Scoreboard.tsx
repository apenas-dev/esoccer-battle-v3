import { type HTMLAttributes, useState, useCallback, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

function cn(...inputs: (string | undefined | false | null)[]) {
  return inputs.filter(Boolean).join(' ');
}

// ── Types ─────────────────────────────────────────────
type MatchStatus = 'idle' | 'playing' | 'paused' | 'challenge' | 'finished';

interface ScoreboardProps extends HTMLAttributes<HTMLDivElement> {
  /** Display name for Team A */
  teamAName: string;
  /** Display name for Team B */
  teamBName: string;
  /** Current score for Team A */
  scoreA: number;
  /** Current score for Team B */
  scoreB: number;
  /** Match status — affects visual styling */
  status: MatchStatus;
  /** Which team just scored — triggers confetti */
  lastScorer?: 'A' | 'B' | null;
  /** Called when scoreA should change */
  onScoreAChange?: (newScore: number) => void;
  /** Called when scoreB should change */
  onScoreBChange?: (newScore: number) => void;
}

// ── Confetti Particle ─────────────────────────────────
interface Particle {
  id: number;
  x: number;
  color: string;
  delay: number;
  angle: number;
  distance: number;
}

function ConfettiExplosion({ active, team }: { active: boolean; team: 'A' | 'B' }) {
  const [particles, setParticles] = useState<Particle[]>([]);

  useEffect(() => {
    if (!active) {
      setParticles([]);
      return;
    }

    const colors =
      team === 'A'
        ? ['#22d3ee', '#06b6d4', '#00ff88', '#fbbf24', '#ffffff']
        : ['#f87171', '#ef4444', '#fbbf24', '#00ff88', '#ffffff'];

    const newParticles: Particle[] = Array.from({ length: 20 }, (_, i) => ({
      id: Date.now() + i,
      x: Math.random() * 100,
      color: colors[Math.floor(Math.random() * colors.length)],
      delay: Math.random() * 0.3,
      angle: (Math.PI * 2 * i) / 20,
      distance: 80 + Math.random() * 120,
    }));

    setParticles(newParticles);
    const timer = setTimeout(() => setParticles([]), 2000);
    return () => clearTimeout(timer);
  }, [active, team]);

  if (particles.length === 0) return null;

  return (
    <div className="absolute inset-0 pointer-events-none overflow-hidden" aria-hidden="true">
      {particles.map((p) => (
        <motion.div
          key={p.id}
          initial={{ opacity: 1, scale: 0, x: '50%', y: '50%' }}
          animate={{
            opacity: 0,
            scale: 1,
            x: `calc(50% + ${Math.cos(p.angle) * p.distance}px)`,
            y: `calc(50% + ${Math.sin(p.angle) * p.distance}px)`,
          }}
          transition={{ duration: 1.2, delay: p.delay, ease: 'easeOut' }}
          className="absolute w-2 h-2 rounded-full"
          style={{ backgroundColor: p.color }}
        />
      ))}
    </div>
  );
}

// ── Score Cell ────────────────────────────────────────
function ScoreCell({
  score,
  team,
  isEditing,
  onStartEdit,
  onIncrement,
  onDecrement,
}: {
  score: number;
  team: 'A' | 'B';
  isEditing: boolean;
  onStartEdit: () => void;
  onIncrement: () => void;
  onDecrement: () => void;
}) {
  const accentColor = team === 'A' ? 'text-cyan-400' : 'text-red-400';
  const glowColor = team === 'A' ? 'shadow-cyan-500/30' : 'shadow-red-500/30';
  const btnColor = team === 'A' ? 'hover:bg-cyan-900/40' : 'hover:bg-red-900/40';

  return (
    <div className="flex flex-col items-center gap-1 relative">
      {/* Increment/decrement buttons — visible when playing */}
      {isEditing && (
        <div className="flex gap-2 mb-1">
          <button
            onClick={onDecrement}
            className={`w-8 h-8 rounded-full bg-gray-800 border border-gray-700 ${btnColor} text-gray-300 text-lg font-bold transition-colors duration-150 flex items-center justify-center focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-400`}
            aria-label={`Diminuir gol do ${team === 'A' ? 'time A' : 'time B'}`}
          >
            −
          </button>
          <button
            onClick={onIncrement}
            className={`w-8 h-8 rounded-full bg-gray-800 border border-gray-700 ${btnColor} text-gray-300 text-lg font-bold transition-colors duration-150 flex items-center justify-center focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-amber-400`}
            aria-label={`Aumentar gol do ${team === 'A' ? 'time A' : 'time B'}`}
          >
            +
          </button>
        </div>
      )}

      {/* Score display */}
      <motion.div
        key={score}
        initial={{ scale: 1.4, opacity: 0.5 }}
        animate={{ scale: 1, opacity: 1 }}
        transition={{ type: 'spring', stiffness: 300, damping: 20 }}
        onClick={isEditing ? onStartEdit : undefined}
        className={`
          relative text-[6rem] sm:text-[8rem] lg:text-[10rem] font-black leading-none tabular-nums
          ${accentColor} ${glowColor}
          ${isEditing ? 'cursor-pointer hover:opacity-80 transition-opacity' : ''}
        `}
        style={isEditing ? {} : { textShadow: team === 'A' ? '0 0 30px rgba(34,211,238,0.4)' : '0 0 30px rgba(248,113,113,0.4)' }}
        aria-label={`Placar ${team === 'A' ? 'time A' : 'time B'}: ${score} gols`}
        role="img"
      >
        {score}
      </motion.div>
    </div>
  );
}

// ── Status Badge ──────────────────────────────────────
function StatusBadge({ status }: { status: MatchStatus }) {
  const config: Record<MatchStatus, { label: string; className: string }> = {
    idle: { label: 'AGUARDANDO', className: 'bg-gray-700 text-gray-400' },
    playing: { label: 'EM JOGO', className: 'bg-emerald-900/60 text-[#00ff88] border border-emerald-500/30' },
    paused: { label: 'PAUSADO', className: 'bg-amber-900/60 text-amber-400 border border-amber-500/30' },
    challenge: { label: 'DÚVIDA', className: 'bg-violet-900/60 text-violet-400 border border-violet-500/30 animate-pulse' },
    finished: { label: 'ENCERRADO', className: 'bg-blue-900/60 text-blue-400 border border-blue-500/30' },
  };

  const { label, className } = config[status];

  return (
    <motion.div
      key={status}
      initial={{ opacity: 0, y: -8 }}
      animate={{ opacity: 1, y: 0 }}
      className={`inline-flex items-center gap-2 px-4 py-1.5 rounded-full text-sm font-bold uppercase tracking-widest ${className}`}
      role="status"
      aria-label={`Status da partida: ${label}`}
    >
      {status === 'playing' && (
        <span className="relative flex h-2 w-2">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#00ff88] opacity-75" />
          <span className="relative inline-flex rounded-full h-2 w-2 bg-[#00ff88]" />
        </span>
      )}
      {status === 'challenge' && (
        <span className="relative flex h-2 w-2">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-violet-400 opacity-75" />
          <span className="relative inline-flex rounded-full h-2 w-2 bg-violet-400" />
        </span>
      )}
      {label}
    </motion.div>
  );
}

// ── Winner Banner ─────────────────────────────────────
function WinnerBanner({ scoreA, scoreB, teamAName, teamBName }: { scoreA: number; scoreB: number; teamAName: string; teamBName: string }) {
  let message: string;
  let accent: string;

  if (scoreA > scoreB) {
    message = `🏆 ${teamAName} Venceu!`;
    accent = 'text-cyan-400';
  } else if (scoreB > scoreA) {
    message = `🏆 ${teamBName} Venceu!`;
    accent = 'text-red-400';
  } else {
    message = '🤝 Empate!';
    accent = 'text-amber-400';
  }

  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.8 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ type: 'spring', stiffness: 200, damping: 15 }}
      className={`text-2xl sm:text-3xl font-black ${accent} mt-2`}
      role="alert"
      aria-live="polite"
    >
      {message}
    </motion.div>
  );
}

// ── Scoreboard Component ──────────────────────────────
export function Scoreboard({
  teamAName,
  teamBName,
  scoreA,
  scoreB,
  status,
  lastScorer = null,
  onScoreAChange,
  onScoreBChange,
  className,
  ...props
}: ScoreboardProps) {
  const [showConfetti, setShowConfetti] = useState(false);
  const [prevScoreA, setPrevScoreA] = useState(scoreA);
  const [prevScoreB, setPrevScoreB] = useState(scoreB);

  useEffect(() => {
    if (scoreA !== prevScoreA || scoreB !== prevScoreB) {
      setShowConfetti(true);
      setPrevScoreA(scoreA);
      setPrevScoreB(scoreB);
      const timer = setTimeout(() => setShowConfetti(false), 2000);
      return () => clearTimeout(timer);
    }
  }, [scoreA, scoreB, prevScoreA, prevScoreB]);

  const isEditable = status === 'playing';

  const handleIncrementA = useCallback(() => onScoreAChange?.(scoreA + 1), [scoreA, onScoreAChange]);
  const handleDecrementA = useCallback(() => onScoreAChange?.(Math.max(0, scoreA - 1)), [scoreA, onScoreAChange]);
  const handleIncrementB = useCallback(() => onScoreBChange?.(scoreB + 1), [scoreB, onScoreBChange]);
  const handleDecrementB = useCallback(() => onScoreBChange?.(Math.max(0, scoreB - 1)), [scoreB, onScoreBChange]);

  const isChallenge = status === 'challenge';

  return (
    <div
      className={cn(
        'relative w-full max-w-3xl mx-auto',
        className
      )}
      {...props}
    >
      <div
        className={`
          relative overflow-hidden rounded-2xl border p-6 sm:p-8 lg:p-10
          transition-all duration-500
          ${isChallenge
            ? 'bg-[#0d1117] border-violet-500/50 shadow-[0_0_40px_rgba(167,139,250,0.3)]'
            : status === 'finished'
              ? 'bg-[#0d1117] border-blue-500/30 shadow-[0_0_30px_rgba(96,165,250,0.2)]'
              : 'bg-[#0d1117] border-[#1e3a5f] shadow-[0_0_20px_rgba(0,255,136,0.15)]'
          }
        `}
        role="region"
        aria-label="Placar da partida"
      >
        {/* Confetti layer */}
        <ConfettiExplosion active={showConfetti} team={lastScorer ?? (scoreA > prevScoreA ? 'A' : 'B')} />

        {/* Challenge flash overlay */}
        <AnimatePresence>
          {isChallenge && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="absolute inset-0 bg-violet-500/5 pointer-events-none"
              aria-hidden="true"
            />
          )}
        </AnimatePresence>

        {/* Status */}
        <div className="flex justify-center mb-4">
          <StatusBadge status={status} />
        </div>

        {/* Main scoreboard layout */}
        <div className="flex items-center justify-center gap-4 sm:gap-8 lg:gap-16">
          {/* Team A */}
          <div className="flex flex-col items-center gap-2 flex-1 min-w-0">
            <h2 className="text-xl sm:text-2xl font-bold text-cyan-400 truncate max-w-[160px]" title={teamAName}>
              {teamAName}
            </h2>
            <ScoreCell
              score={scoreA}
              team="A"
              isEditing={isEditable}
              onStartEdit={() => {}}
              onIncrement={handleIncrementA}
              onDecrement={handleDecrementA}
            />
          </div>

          {/* VS divider */}
          <div className="flex flex-col items-center gap-2">
            <span className="text-3xl sm:text-4xl font-black text-gray-600 select-none" aria-hidden="true">×</span>
          </div>

          {/* Team B */}
          <div className="flex flex-col items-center gap-2 flex-1 min-w-0">
            <h2 className="text-xl sm:text-2xl font-bold text-red-400 truncate max-w-[160px]" title={teamBName}>
              {teamBName}
            </h2>
            <ScoreCell
              score={scoreB}
              team="B"
              isEditing={isEditable}
              onStartEdit={() => {}}
              onIncrement={handleIncrementB}
              onDecrement={handleDecrementB}
            />
          </div>
        </div>

        {/* Winner banner (finished state) */}
        {status === 'finished' && (
          <div className="flex justify-center mt-4">
            <WinnerBanner scoreA={scoreA} scoreB={scoreB} teamAName={teamAName} teamBName={teamBName} />
          </div>
        )}

        {/* Idle state CTA */}
        {status === 'idle' && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.5 }}
            className="flex justify-center mt-6"
          >
            <p className="text-gray-500 text-sm">
              Fale <span className="text-[#00ff88] font-semibold">"iniciar partida"</span> ou clique em começar
            </p>
          </motion.div>
        )}
      </div>
    </div>
  );
}

export { type ScoreboardProps, type MatchStatus };

import { motion } from 'framer-motion';
import type { GamePhase, PlayingSubPhase } from '../../types';
import { cn } from '../../lib/utils';

interface ScoreboardProps {
  teamAName: string;
  teamBName: string;
  scoreA: number;
  scoreB: number;
  phase: GamePhase;
  subPhase: PlayingSubPhase;
  flashTeam?: 'a' | 'b' | null;
}

export function Scoreboard({ teamAName, teamBName, scoreA, scoreB, phase, subPhase, flashTeam }: ScoreboardProps) {
  const isActive = phase === 'playing' || phase === 'paused';

  return (
    <div className="flex flex-col items-center gap-2">
      {/* Phase indicator */}
      <div className="flex items-center gap-2 text-xs font-medium text-[var(--text-secondary)]">
        <span
          className={cn(
            'rounded-full px-2 py-0.5 text-xs font-bold uppercase',
            phase === 'playing' && subPhase === 'challenge'
              ? 'bg-neon-yellow/20 text-neon-yellow'
              : phase === 'playing'
                ? 'bg-neon-green/20 text-neon-green'
                : phase === 'paused'
                  ? 'bg-neon-blue/20 text-neon-blue'
                  : 'bg-gray-500/20 text-gray-500',
          )}
        >
          {phase === 'playing' && subPhase === 'challenge' ? '⚠️ Dúvida' : phase}
        </span>
      </div>

      {/* Scoreboard */}
      <div className="flex items-center gap-6 rounded-2xl border border-[var(--border-color)] bg-[var(--bg-card)] p-6 shadow-lg">
        {/* Team A */}
        <motion.div
          className="flex flex-col items-center gap-2"
          animate={flashTeam === 'a' ? { scale: [1, 1.15, 1] } : {}}
          transition={{ duration: 0.4 }}
        >
          <span className="text-sm font-medium text-[var(--text-secondary)]">{teamAName}</span>
          <motion.span
            className={cn(
              'text-6xl font-black tabular-nums',
              flashTeam === 'a' ? 'neon-glow-green text-neon-green' : 'text-[var(--text-primary)]',
            )}
            key={scoreA}
            initial={{ scale: 1.3 }}
            animate={{ scale: 1 }}
            transition={{ duration: 0.3 }}
          >
            {scoreA}
          </motion.span>
        </motion.div>

        {/* Divider */}
        <div className="flex flex-col items-center gap-1">
          <span className={cn('text-2xl font-light', isActive ? 'text-[var(--text-primary)]' : 'text-gray-500')}>×</span>
        </div>

        {/* Team B */}
        <motion.div
          className="flex flex-col items-center gap-2"
          animate={flashTeam === 'b' ? { scale: [1, 1.15, 1] } : {}}
          transition={{ duration: 0.4 }}
        >
          <span className="text-sm font-medium text-[var(--text-secondary)]">{teamBName}</span>
          <motion.span
            className={cn(
              'text-6xl font-black tabular-nums',
              flashTeam === 'b' ? 'neon-glow-green text-neon-green' : 'text-[var(--text-primary)]',
            )}
            key={scoreB}
            initial={{ scale: 1.3 }}
            animate={{ scale: 1 }}
            transition={{ duration: 0.3 }}
          >
            {scoreB}
          </motion.span>
        </motion.div>
      </div>
    </div>
  );
}

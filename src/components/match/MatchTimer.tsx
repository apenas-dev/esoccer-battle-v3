import { type HTMLAttributes, useEffect, useRef } from 'react';
import { motion } from 'framer-motion';

function cn(...inputs: (string | undefined | false | null)[]) {
  return inputs.filter(Boolean).join(' ');
}

// ── Types ─────────────────────────────────────────────
interface MatchTimerProps extends HTMLAttributes<HTMLDivElement> {
  /** Elapsed time in seconds (controlled externally) */
  elapsedSeconds: number;
  /** Whether the timer is currently running */
  isRunning: boolean;
}

// ── Helpers ───────────────────────────────────────────
function formatTime(totalSeconds: number): string {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, '0')}:${String(seconds).padStart(2, '0')}`;
}

// ── Component ─────────────────────────────────────────
export function MatchTimer({
  elapsedSeconds,
  isRunning,
  className,
  ...props
}: MatchTimerProps) {
  const formatted = formatTime(elapsedSeconds);
  const prevFormatted = useRef(formatted);

  // Detect digit change for animation
  const hasChanged = formatted !== prevFormatted.current;
  useEffect(() => {
    prevFormatted.current = formatted;
  }, [formatted]);

  const minutes = formatted.slice(0, 2);
  const seconds = formatted.slice(3, 5);

  return (
    <div
      className={cn('flex flex-col items-center gap-1', className)}
      role="timer"
      aria-label={`Tempo de jogo: ${formatted}`}
      aria-live="polite"
      {...props}
    >
      {/* Timer display */}
      <div className="flex items-baseline tabular-nums">
        {/* Minutes */}
        <motion.span
          key={minutes}
          initial={hasChanged ? { y: -10, opacity: 0 } : false}
          animate={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
          className="text-5xl sm:text-6xl lg:text-7xl font-bold text-gray-200 tracking-tight"
          aria-hidden="true"
        >
          {minutes}
        </motion.span>

        {/* Separator — blinks when running */}
        <span className="text-5xl sm:text-6xl lg:text-7xl font-bold mx-1" aria-hidden="true">
          <motion.span
            animate={isRunning ? { opacity: [1, 0.3, 1] } : { opacity: 1 }}
            transition={isRunning ? { duration: 1, repeat: Infinity, ease: 'easeInOut' } : undefined}
            className="text-[#00ff88]"
          >
            :
          </motion.span>
        </span>

        {/* Seconds */}
        <motion.span
          key={seconds}
          initial={hasChanged ? { y: -10, opacity: 0 } : false}
          animate={{ y: 0, opacity: 1 }}
          transition={{ duration: 0.15, ease: 'easeOut' }}
          className="text-5xl sm:text-6xl lg:text-7xl font-bold text-gray-200 tracking-tight"
          aria-hidden="true"
        >
          {seconds}
        </motion.span>
      </div>

      {/* Running indicator */}
      <div className="flex items-center gap-2">
        {isRunning && (
          <motion.span
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="relative flex h-2 w-2"
            aria-hidden="true"
          >
            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#00ff88] opacity-75" />
            <span className="relative inline-flex rounded-full h-2 w-2 bg-[#00ff88]" />
          </motion.span>
        )}
        <span className={`text-xs font-semibold uppercase tracking-widest ${isRunning ? 'text-[#00ff88]' : 'text-gray-600'}`}>
          {isRunning ? 'Ao vivo' : elapsedSeconds === 0 ? 'Cronômetro' : 'Parado'}
        </span>
      </div>
    </div>
  );
}

export { type MatchTimerProps, formatTime };

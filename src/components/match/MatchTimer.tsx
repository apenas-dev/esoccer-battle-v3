import { cn } from "../../lib/cn";
import { type HTMLAttributes, memo, useEffect, useRef } from 'react';
import { motion } from 'framer-motion';


// ── Types ─────────────────────────────────────────────
export interface MatchTimerProps extends HTMLAttributes<HTMLDivElement> {
  elapsedSeconds: number;
  isRunning: boolean;
}

export function formatTime(totalSeconds: number): string {
  const m = Math.floor(totalSeconds / 60);
  const s = totalSeconds % 60;
  return `${String(m).padStart(2, '0')}:${String(s).padStart(2, '0')}`;
}

// ── Component ─────────────────────────────────────────
export const MatchTimer = memo(function MatchTimer({ elapsedSeconds, isRunning, className, ...props }: MatchTimerProps) {
  const formatted = formatTime(elapsedSeconds);
  const prevRef = useRef(formatted);
  const changed = formatted !== prevRef.current;
  useEffect(() => { prevRef.current = formatted; }, [formatted]);

  const minutes = formatted.slice(0, 2);
  const seconds = formatted.slice(3, 5);

  return (
    <div className={cn('flex flex-col items-center gap-1', className)}
      role="timer" aria-label={`Tempo de jogo: ${formatted}`} aria-live="polite" {...props}>
      <div className="flex items-baseline tabular-nums">
        <motion.span key={`min-${minutes}`} initial={changed ? { y: -10, opacity: 0 } : false} animate={{ y: 0, opacity: 1 }} transition={{ duration: 0.15 }}
          className="text-5xl sm:text-6xl lg:text-7xl font-bold text-gray-200 tracking-tight" aria-hidden="true">{minutes}</motion.span>
        <span className="text-5xl sm:text-6xl lg:text-7xl font-bold mx-1" aria-hidden="true">
          <motion.span animate={isRunning ? { opacity: [1, 0.3, 1] } : { opacity: 1 }}
            transition={isRunning ? { duration: 1, repeat: Infinity, ease: 'easeInOut' } : undefined}
            className="text-[#00ff88]">:</motion.span>
        </span>
        <motion.span key={`sec-${seconds}`} initial={changed ? { y: -10, opacity: 0 } : false} animate={{ y: 0, opacity: 1 }} transition={{ duration: 0.15 }}
          className="text-5xl sm:text-6xl lg:text-7xl font-bold text-gray-200 tracking-tight" aria-hidden="true">{seconds}</motion.span>
      </div>
      <div className="flex items-center gap-2">
        {isRunning && <span className="relative flex h-2 w-2" aria-hidden="true">
          <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-[#00ff88] opacity-75" />
          <span className="relative inline-flex rounded-full h-2 w-2 bg-[#00ff88]" />
        </span>}
        <span className={`text-xs font-semibold uppercase tracking-widest ${isRunning ? 'text-[#00ff88]' : 'text-gray-600'}`}>
          {isRunning ? 'Ao vivo' : elapsedSeconds === 0 ? 'Cronômetro' : 'Parado'}
        </span>
      </div>
    </div>
  );
});

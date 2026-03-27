import type { GamePhase, TimerMode } from '../../types';
import { cn } from '../../lib/utils';

interface TimerProps {
  displayTime: string;
  phase: GamePhase;
  timerMode: TimerMode;
  totalDuration: number;
  elapsed: number;
}

export function Timer({ displayTime, phase, timerMode, totalDuration, elapsed }: TimerProps) {
  const isActive = phase === 'playing';
  const progress = timerMode === 'countdown'
    ? Math.max(0, 1 - elapsed / totalDuration)
    : Math.min(1, elapsed / totalDuration);

  const circumference = 2 * Math.PI * 45;
  const strokeDashoffset = circumference * (1 - progress);

  const isLow = timerMode === 'countdown' && progress < 0.1 && isActive;

  return (
    <div className="flex flex-col items-center gap-2">
      <div className="relative flex h-32 w-32 items-center justify-center">
        <svg className="absolute inset-0 -rotate-90" viewBox="0 0 100 100">
          {/* Background circle */}
          <circle cx="50" cy="50" r="45" fill="none" stroke="var(--border-color)" strokeWidth="4" />
          {/* Progress circle */}
          <circle
            cx="50"
            cy="50"
            r="45"
            fill="none"
            stroke={isLow ? '#ff3131' : '#39ff14'}
            strokeWidth="4"
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={strokeDashoffset}
            className={cn(isActive ? 'transition-all duration-1000' : 'transition-none')}
            style={{ filter: isLow ? 'drop-shadow(0 0 6px #ff3131)' : 'drop-shadow(0 0 6px #39ff14)' }}
          />
        </svg>
        <span
          className={cn(
            'text-3xl font-bold tabular-nums',
            isLow && 'neon-glow-red text-neon-red',
            isActive && !isLow && 'neon-glow-green text-neon-green',
            !isActive && 'text-[var(--text-secondary)]',
          )}
        >
          {displayTime}
        </span>
      </div>
      <span className="text-xs text-[var(--text-secondary)]">
        {timerMode === 'countdown' ? 'Regressivo' : 'Progressivo'}
      </span>
    </div>
  );
}

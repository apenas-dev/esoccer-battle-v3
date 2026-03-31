interface TimerProps {
  displayTime: string;
  phase: string;
  durationSecs: number;
  timerMode: string;
}

export function Timer({ displayTime, phase, durationSecs, timerMode }: TimerProps) {
  const elapsedSecs = (() => {
    const parts = displayTime.split(':').map(Number);
    return (parts[0] ?? 0) * 60 + (parts[1] ?? 0);
  })();

  const progress = timerMode === 'countdown' && durationSecs > 0
    ? Math.max(0, Math.min(100, ((durationSecs - elapsedSecs) / durationSecs) * 100))
    : 0;

  const isPlaying = phase === 'playing';
  const isPaused = phase === 'paused';

  return (
    <div className="w-full max-w-md mx-auto">
      <div className="text-center">
        <p className={`text-4xl font-mono font-bold ${isPaused ? 'text-yellow-400' : isPlaying ? 'text-white' : 'text-gray-400'}`}>
          {isPaused ? '⏸ ' : ''}{displayTime}
        </p>
        {timerMode === 'countdown' && durationSecs > 0 && (
          <div className="mt-3 h-2 bg-gray-800 rounded-full overflow-hidden">
            <div
              className="h-full bg-blue-500 transition-all duration-1000 rounded-full"
              style={{ width: `${progress}%` }}
            />
          </div>
        )}
      </div>
    </div>
  );
}

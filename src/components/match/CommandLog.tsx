import { useEffect, useRef } from 'react';
import type { CommandLogEntry } from '../../types';
import { cn } from '../../lib/utils';

interface CommandLogProps {
  entries: CommandLogEntry[];
}

export function CommandLog({ entries }: CommandLogProps) {
  const scrollRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [entries]);

  if (entries.length === 0) {
    return (
      <div className="rounded-lg border border-dashed border-[var(--border-color)] p-4 text-center text-sm text-[var(--text-secondary)]">
        Nenhum comando executado
      </div>
    );
  }

  return (
    <div ref={scrollRef} className="max-h-48 overflow-y-auto rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)]">
      {entries.map((entry) => (
        <div
          key={entry.id}
          className={cn(
            'flex items-center justify-between border-b border-[var(--border-color)] px-3 py-2 text-sm last:border-b-0',
          )}
        >
          <div className="flex items-center gap-2">
            <span
              className={cn(
                'rounded-full px-1.5 py-0.5 text-xs font-medium',
                entry.source === 'voice'
                  ? 'bg-neon-blue/20 text-neon-blue'
                  : 'bg-gray-200 text-gray-700 dark:bg-gray-700 dark:text-gray-300',
              )}
            >
              {entry.source === 'voice' ? '🎤' : '🖱️'}
            </span>
            <span className="font-medium text-[var(--text-primary)]">{entry.command}</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="text-xs text-[var(--text-secondary)]">
              {entry.timestamp.toLocaleTimeString('pt-BR')}
            </span>
            <span className={entry.success ? 'text-neon-green' : 'text-neon-red'}>
              {entry.success ? '✓' : '✗'}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}

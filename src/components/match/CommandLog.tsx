import { type HTMLAttributes } from 'react';
import { motion, AnimatePresence } from 'framer-motion';

function cn(...inputs: (string | undefined | false | null)[]) {
  return inputs.filter(Boolean).join(' ');
}

// ── Types ─────────────────────────────────────────────
interface CommandEntry {
  id: string;
  text: string;
  timestamp: Date;
  type?: 'goal' | 'challenge' | 'control';
}

interface CommandLogProps extends HTMLAttributes<HTMLDivElement> {
  /** Array of recognized commands (most recent first) */
  commands: CommandEntry[];
  /** Maximum number of visible entries */
  maxEntries?: number;
}

// ── Type Icon ─────────────────────────────────────────
function CommandIcon({ type }: { type?: 'goal' | 'challenge' | 'control' }) {
  if (type === 'goal') {
    return (
      <span className="text-[#fbbf24]" aria-hidden="true">⚽</span>
    );
  }
  if (type === 'challenge') {
    return (
      <span className="text-violet-400" aria-hidden="true">❓</span>
    );
  }
  return (
    <span className="text-gray-500" aria-hidden="true">🎙️</span>
  );
}

// ── Format Time ───────────────────────────────────────
function formatTimestamp(date: Date): string {
  return date.toLocaleTimeString('pt-BR', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}

// ── Component ─────────────────────────────────────────
export function CommandLog({
  commands,
  maxEntries = 5,
  className,
  ...props
}: CommandLogProps) {
  const visibleCommands = commands.slice(0, maxEntries);

  return (
    <div
      className={cn(
        'w-full max-w-md mx-auto',
        className
      )}
      role="log"
      aria-label="Histórico de comandos de voz"
      aria-live="polite"
      {...props}
    >
      {/* Header */}
      <div className="flex items-center gap-2 mb-2 px-1">
        <svg
          className="w-4 h-4 text-gray-500"
          fill="none"
          viewBox="0 0 24 24"
          strokeWidth={1.5}
          stroke="currentColor"
          aria-hidden="true"
        >
          <path strokeLinecap="round" strokeLinejoin="round" d="M3.75 12h16.5m-16.5 3.75h16.5M3.75 19.5h16.5M5.625 4.5h12.75a1.875 1.875 0 010 3.75H5.625a1.875 1.875 0 010-3.75z" />
        </svg>
        <h3 className="text-xs font-semibold uppercase tracking-widest text-gray-500">
          Comandos
        </h3>
        {commands.length > 0 && (
          <span className="ml-auto text-xs text-gray-600 tabular-nums">
            {commands.length}
          </span>
        )}
      </div>

      {/* Command list */}
      <div className="space-y-1 min-h-[2rem]">
        <AnimatePresence mode="popLayout">
          {visibleCommands.length === 0 ? (
            <motion.p
              key="empty"
              initial={{ opacity: 0 }}
              animate={{ opacity: 0.5 }}
              className="text-sm text-gray-600 italic px-1"
            >
              Nenhum comando reconhecido
            </motion.p>
          ) : (
            visibleCommands.map((cmd) => (
              <motion.div
                key={cmd.id}
                layout
                initial={{ opacity: 0, x: -20, scale: 0.95 }}
                animate={{ opacity: 1, x: 0, scale: 1 }}
                exit={{ opacity: 0, x: 20, scale: 0.95 }}
                transition={{ type: 'spring', stiffness: 400, damping: 30 }}
                className={`
                  flex items-center gap-3 px-3 py-2 rounded-lg
                  transition-colors duration-200
                  ${cmd.type === 'goal'
                    ? 'bg-amber-900/20 border border-amber-500/10'
                    : cmd.type === 'challenge'
                      ? 'bg-violet-900/20 border border-violet-500/10'
                      : 'bg-gray-800/50 border border-gray-700/30'
                  }
                `}
                aria-label={`Comando: ${cmd.text}`}
              >
                <CommandIcon type={cmd.type} />
                <span className={`flex-1 text-sm font-medium ${
                  cmd.type === 'goal'
                    ? 'text-amber-300'
                    : cmd.type === 'challenge'
                      ? 'text-violet-300'
                      : 'text-gray-300'
                }`}>
                  {cmd.text}
                </span>
                <span className="text-xs text-gray-600 tabular-nums font-mono">
                  {formatTimestamp(cmd.timestamp)}
                </span>
              </motion.div>
            ))
          )}
        </AnimatePresence>
      </div>
    </div>
  );
}

export { type CommandLogProps, type CommandEntry };

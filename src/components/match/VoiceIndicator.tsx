import { motion } from 'framer-motion';
import type { VoiceStatus } from '../../types';
import { cn } from '../../lib/utils';

interface VoiceIndicatorProps {
  status: VoiceStatus;
  lastTranscript: string | null;
  isListening: boolean;
  onStart: () => void;
  onStop: () => void;
}

export function VoiceIndicator({ status, lastTranscript, isListening, onStart, onStop }: VoiceIndicatorProps) {
  return (
    <div className="flex flex-col items-center gap-3">
      {/* PTT Button */}
      <motion.button
        onMouseDown={onStart}
        onMouseUp={onStop}
        onTouchStart={onStart}
        onTouchEnd={onStop}
        className={cn(
          'flex h-20 w-20 items-center justify-center rounded-full text-3xl transition-colors',
          isListening
            ? 'bg-neon-red text-white'
            : 'bg-[var(--bg-card)] text-[var(--text-secondary)] border-2 border-[var(--border-color)]',
        )}
        animate={isListening ? { scale: [1, 1.1, 1] } : { scale: 1 }}
        transition={{ duration: 0.8, repeat: isListening ? Infinity : 0 }}
        aria-label={isListening ? 'Release to stop listening' : 'Press to talk'}
      >
        🎤
        {isListening && (
          <motion.div
            className="absolute h-20 w-20 rounded-full border-4 border-neon-red"
            animate={{ scale: [1, 1.5], opacity: [0.6, 0] }}
            transition={{ duration: 1, repeat: Infinity }}
          />
        )}
      </motion.button>

      {/* Status */}
      <div className="text-center">
        <span className="text-xs text-[var(--text-secondary)]">
          {status === 'listening' && '🎤 Ouvindo...'}
          {status === 'processing' && '⏳ Processando...'}
          {status === 'error' && '❌ Erro'}
          {status === 'idle' && !lastTranscript && 'Pressione para falar'}
          {status === 'idle' && lastTranscript && 'Pronto'}
        </span>
      </div>

      {/* Last transcript */}
      {lastTranscript && (
        <div className="rounded-lg bg-[var(--bg-card)] border border-[var(--border-color)] px-3 py-1.5 text-sm text-[var(--text-secondary)] max-w-xs text-center">
          &ldquo;{lastTranscript}&rdquo;
        </div>
      )}
    </div>
  );
}

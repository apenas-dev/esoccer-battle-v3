import { cn } from "../../lib/cn";
import { type HTMLAttributes, memo } from 'react';
import { motion } from 'framer-motion';


// ── Types ─────────────────────────────────────────────
export type VoiceState = 'idle' | 'listening' | 'processing' | 'error';

export interface VoiceIndicatorProps extends HTMLAttributes<HTMLDivElement> {
  voiceState: VoiceState;
  label?: string;
}

// ── Wave Bars ─────────────────────────────────────────
function WaveBar({ delay, isActive }: { delay: number; isActive: boolean }) {
  return (
    <motion.div className="w-1 rounded-full bg-[#00ff88]"
      animate={isActive ? { height: [8, 28, 12, 24, 8], opacity: [0.5, 1, 0.7, 1, 0.5] } : { height: 8, opacity: 0.3 }}
      transition={isActive ? { duration: 1.2, repeat: Infinity, delay, ease: 'easeInOut' } : { duration: 0.3 }}
      aria-hidden="true" />
  );
}

// ── Pulse Ring ────────────────────────────────────────
function PulseRing({ state }: { state: VoiceState }) {
  if (state === 'idle') return null;
  const color = state === 'listening' ? '#00ff88' : state === 'processing' ? '#fbbf24' : '#f87171';
  return (
    <motion.div className="absolute inset-0 rounded-full border-2" style={{ borderColor: color }}
      animate={{ scale: [1, 1.8], opacity: [0.6, 0] }} transition={{ duration: 1.5, repeat: Infinity, ease: 'easeOut' }}
      aria-hidden="true" />
  );
}

// ── Component ─────────────────────────────────────────
export const VoiceIndicator = memo(function VoiceIndicator({ voiceState, label, className, ...props }: VoiceIndicatorProps) {
  const isActive = voiceState === 'listening';
  const cfg: Record<VoiceState, { text: string; color: string; bg: string }> = {
    idle: { text: label ?? 'Mic desligado', color: 'text-gray-500', bg: 'bg-gray-800' },
    listening: { text: label ?? 'Ouvindo...', color: 'text-[#00ff88]', bg: 'bg-emerald-900/40' },
    processing: { text: 'Processando...', color: 'text-amber-400', bg: 'bg-amber-900/40' },
    error: { text: 'Erro no mic', color: 'text-red-400', bg: 'bg-red-900/40' },
  };
  const c = cfg[voiceState];

  return (
    <div className={cn('flex flex-col items-center gap-3', className)}
      role="status" aria-label={c.text} aria-live="polite" {...props}>
      <div className="relative flex items-center justify-center w-16 h-16 sm:w-20 sm:h-20">
        <PulseRing state={voiceState} />
        <motion.div className={`relative z-10 flex items-center justify-center w-16 h-16 sm:w-20 sm:h-20 rounded-full transition-colors duration-300 ${c.bg}`}
          animate={isActive ? { scale: [1, 1.05, 1] } : {}}
          transition={isActive ? { duration: 2, repeat: Infinity, ease: 'easeInOut' } : undefined}>
          <svg className={`w-7 h-7 sm:w-8 sm:h-8 ${c.color}`} fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" aria-hidden="true">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 18.75a6 6 0 006-6v-1.5m-6 7.5a6 6 0 01-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 01-3-3V4.5a3 3 0 116 0v8.25a3 3 0 01-3 3z" />
          </svg>
        </motion.div>
      </div>
      {isActive && (
        <div className="flex items-center gap-1 h-8" aria-hidden="true">
          {[0, 0.15, 0.3, 0.1, 0.25, 0.05, 0.2].map((d, i) => <WaveBar key={i} delay={d} isActive />)}
        </div>
      )}
      <motion.span key={voiceState} initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }}
        className={`text-xs sm:text-sm font-semibold uppercase tracking-widest ${c.color}`}>
        {c.text}
      </motion.span>
    </div>
  );
});

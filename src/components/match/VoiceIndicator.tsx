import { cn } from "../../lib/cn";
import { type HTMLAttributes, memo } from 'react';
import { motion } from 'framer-motion';


// ── Types ─────────────────────────────────────────────
export type VoiceState = 'idle' | 'listening' | 'processing' | 'error';

export interface VoiceIndicatorProps extends HTMLAttributes<HTMLDivElement> {
  voiceState: VoiceState;
  label?: string;
  onClick?: () => void;
  disabled?: boolean;
}

// ── Wave Bars ─────────────────────────────────────────
function WaveBar({ delay, isActive }: { delay: number; isActive: boolean }) {
  const barColor = isActive ? 'bg-[#ef4444]' : 'bg-gray-600';
  return (
    <motion.div className={cn("w-1.5 rounded-full", barColor)}
      animate={isActive ? { height: [8, 32, 12, 28, 8], opacity: [0.5, 1, 0.7, 1, 0.5] } : { height: 8, opacity: 0.3 }}
      transition={isActive ? { duration: 1.2, repeat: Infinity, delay, ease: 'easeInOut' } : { duration: 0.3 }}
      aria-hidden="true" />
  );
}

// ── Pulse Ring ────────────────────────────────────────
function PulseRing({ state }: { state: VoiceState }) {
  if (state === 'idle' || state === 'error') return null;
  const color = state === 'listening' ? '#ef4444' : '#fbbf24';
  return (
    <motion.div className="absolute inset-0 rounded-full border-2" style={{ borderColor: color }}
      animate={{ scale: [1, 1.8], opacity: [0.6, 0] }} transition={{ duration: 1.5, repeat: Infinity, ease: 'easeOut' }}
      aria-hidden="true" />
  );
}

// ── Component ─────────────────────────────────────────
export const VoiceIndicator = memo(function VoiceIndicator({ voiceState, label, onClick, disabled, className, ...props }: VoiceIndicatorProps) {
  const isActive = voiceState === 'listening';
  const isClickable = !!onClick && !disabled;

  const cfg: Record<VoiceState, { text: string; color: string; bg: string; ringBorder: string }> = {
    idle: { text: label ?? 'Toque para gravar', color: 'text-gray-400', bg: 'bg-gray-800 hover:bg-gray-700', ringBorder: 'ring-gray-600' },
    listening: { text: label ?? 'Gravando... toque para processar', color: 'text-[#ef4444]', bg: 'bg-red-900/50', ringBorder: 'ring-red-500' },
    processing: { text: 'Processando...', color: 'text-amber-400', bg: 'bg-amber-900/40', ringBorder: 'ring-amber-500' },
    error: { text: 'Erro no mic', color: 'text-red-400', bg: 'bg-red-900/40', ringBorder: 'ring-red-500' },
  };
  const c = cfg[voiceState];

  return (
    <div className={cn('flex flex-col items-center gap-3', className)}
      role="button"
      tabIndex={isClickable ? 0 : -1}
      aria-label={c.text}
      aria-live="polite"
      aria-disabled={disabled}
      {...props}>
      <div
        className={cn(
          'relative flex items-center justify-center rounded-full transition-all duration-300',
          'w-20 h-20 sm:w-24 sm:h-24',
          isClickable && 'cursor-pointer active:scale-95',
          !isClickable && 'cursor-default',
          disabled && 'opacity-60',
        )}
        onClick={isClickable ? onClick : undefined}
        onKeyDown={isClickable ? (e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onClick(); } } : undefined}
      >
        <PulseRing state={voiceState} />
        <motion.div
          className={cn(
            `relative z-10 flex items-center justify-center w-20 h-20 sm:w-24 sm:h-24 rounded-full transition-colors duration-300 ${c.bg}`,
            isClickable && 'ring-2',
            isClickable && c.ringBorder,
          )}
          animate={isActive ? { scale: [1, 1.05, 1] } : {}}
          transition={isActive ? { duration: 2, repeat: Infinity, ease: 'easeInOut' } : undefined}>
          <svg className={cn('w-8 h-8 sm:w-10 sm:h-10', c.color)} fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor" aria-hidden="true">
            <path strokeLinecap="round" strokeLinejoin="round" d="M12 18.75a6 6 0 006-6v-1.5m-6 7.5a6 6 0 01-6-6v-1.5m6 7.5v3.75m-3.75 0h7.5M12 15.75a3 3 0 01-3-3V4.5a3 3 0 116 0v8.25a3 3 0 01-3 3z" />
          </svg>
        </motion.div>
      </div>
      {isActive && (
        <div className="flex items-center gap-1.5 h-8" aria-hidden="true">
          {[0, 0.15, 0.3, 0.1, 0.25, 0.05, 0.2].map((d, i) => <WaveBar key={i} delay={d} isActive />)}
        </div>
      )}
      <motion.span key={voiceState} initial={{ opacity: 0, y: 4 }} animate={{ opacity: 1, y: 0 }}
        className={`text-xs sm:text-sm font-semibold uppercase tracking-widest ${c.color} max-w-[200px] text-center`}>
        {c.text}
      </motion.span>
    </div>
  );
});

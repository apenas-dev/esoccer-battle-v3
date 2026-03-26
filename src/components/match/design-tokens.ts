/**
 * E-Soccer Battle V2 — Design Tokens
 * Dark mode base + neon accents + stadium energy
 *
 * These tokens map directly to Tailwind classes.
 * Use them as reference — components use Tailwind classes inline.
 */

export const tokens = {
  colors: {
    // Backgrounds
    bg: {
      primary: '#0a0f1a',      // bg-[#0a0f1a]  — deep dark base
      surface: '#111827',       // bg-gray-900    — cards/panels
      elevated: '#1f2937',      // bg-gray-800    — hover/active states
      field: '#15803d',         // bg-green-700   — grass green accent
    },
    // Neon accents
    neon: {
      green: '#00ff88',         // text-[#00ff88]  — primary neon
      gold: '#fbbf24',          // text-amber-400  — gold accent
      cyan: '#22d3ee',          // text-cyan-400   — Team A accent
      red: '#f87171',           // text-red-400    — Team B accent
      purple: '#a78bfa',        // text-violet-400 — challenge/duvida
    },
    // Scoreboard
    scoreboard: {
      bg: '#0d1117',            // bg-[#0d1117]    — scoreboard panel
      border: '#1e3a5f',        // border-[#1e3a5f]— scoreboard border
      glow: '0 0 20px rgba(0, 255, 136, 0.3)', // neon green glow
      goldGlow: '0 0 20px rgba(251, 191, 36, 0.3)',
    },
    // Status
    status: {
      idle: '#6b7280',          // text-gray-500
      playing: '#00ff88',       // text-[#00ff88]
      paused: '#fbbf24',        // text-amber-400
      challenge: '#a78bfa',     // text-violet-400
      finished: '#60a5fa',      // text-blue-400
    },
  },
  typography: {
    fontFamily: {
      display: 'system-ui, -apple-system, sans-serif', // Fallback — use Inter/Orbitron if available
      mono: 'ui-monospace, monospace',
    },
    score: {
      size: '6rem',             // text-[6rem] or text-8xl
      weight: '900',            // font-black
      lineHeight: '1',
    },
    teamName: {
      size: '1.25rem',          // text-xl
      weight: '700',            // font-bold
    },
    timer: {
      size: '2.5rem',           // text-4xl
      weight: '600',            // font-semibold
    },
  },
  spacing: {
    scoreboard: {
      padding: '1.5rem 2rem',   // p-6 px-8
      gap: '1rem',              // gap-4
      borderRadius: '1rem',     // rounded-2xl
    },
    controlButtons: {
      size: '2.5rem',           // h-10 w-10
      gap: '0.5rem',            // gap-2
    },
  },
  animations: {
    pulse: 'animate-pulse',
    bounce: 'animate-bounce',
    spin: 'animate-spin',
    // Custom via Framer Motion:
    // - score increment: scale(1.3) → scale(1)
    // - confetti on goal: particles explosion
    // - voice pulse: opacity + scale breathing
    // - challenge flash: border glow purple
  },
  borderRadius: {
    sm: '0.375rem',             // rounded
    md: '0.5rem',               // rounded-lg
    lg: '1rem',                 // rounded-2xl
    xl: '1.5rem',               // rounded-3xl
    full: '9999px',             // rounded-full
  },
} as const;

export type DesignTokens = typeof tokens;

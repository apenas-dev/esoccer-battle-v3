/**
 * E-Soccer Battle V3 — Design Tokens
 * Dark mode base + neon accents + stadium energy
 */
export const tokens = {
  colors: {
    bg: { primary: '#0a0f1a', surface: '#111827', elevated: '#1f2937', field: '#15803d' },
    neon: { green: '#00ff88', gold: '#fbbf24', cyan: '#22d3ee', red: '#f87171', purple: '#a78bfa' },
    scoreboard: { bg: '#0d1117', border: '#1e3a5f' },
    status: {
      idle: '#6b7280', playing: '#00ff88', paused: '#fbbf24',
      challenge: '#a78bfa', finished: '#60a5fa',
    },
  },
  typography: {
    score: { size: '6rem', weight: '900' },
    teamName: { size: '1.25rem', weight: '700' },
    timer: { size: '2.5rem', weight: '600' },
  },
  borderRadius: { sm: '0.375rem', md: '0.5rem', lg: '1rem', xl: '1.5rem', full: '9999px' },
} as const;
export type DesignTokens = typeof tokens;

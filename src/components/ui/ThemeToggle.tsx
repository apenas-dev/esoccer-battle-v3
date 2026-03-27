// =============================================================================
// ThemeToggle — Dark/Light theme switch
// SRP: Toggle theme class on <html> element
// DIP: Receives current theme + onChange callback, zero external deps
// =============================================================================

import type { Theme } from '../../types';
import { cn } from '../../lib/cn';

export interface ThemeToggleProps {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  className?: string;
}

export function ThemeToggle({ theme, onThemeChange, className }: ThemeToggleProps) {
  const isDark = theme === 'dark';

  return (
    <button
      type="button"
      onClick={() => onThemeChange(isDark ? 'light' : 'dark')}
      className={cn(
        'relative flex items-center gap-2 px-3 py-2 rounded-lg',
        'bg-zinc-800 border border-zinc-700 text-zinc-300',
        'hover:bg-zinc-700 hover:border-zinc-600 transition-colors duration-200',
        'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
        className,
      )}
      aria-label={isDark ? 'Switch to light theme' : 'Switch to dark theme'}
    >
      {/* Sun icon */}
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        className={cn(
          'w-4 h-4 transition-all duration-300',
          isDark ? 'text-amber-400 scale-100' : 'text-zinc-500 scale-75',
        )}
      >
        <circle cx="12" cy="12" r="5" />
        <line x1="12" y1="1" x2="12" y2="3" />
        <line x1="12" y1="21" x2="12" y2="23" />
        <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
        <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
        <line x1="1" y1="12" x2="3" y2="12" />
        <line x1="21" y1="12" x2="23" y2="12" />
        <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
        <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
      </svg>

      {/* Toggle track */}
      <div className="relative w-10 h-5 rounded-full bg-zinc-600 transition-colors duration-300">
        <div
          className={cn(
            'absolute top-0.5 w-4 h-4 rounded-full transition-all duration-300',
            isDark
              ? 'left-0.5 bg-cyan-400 shadow-[0_0_6px_rgba(6,182,212,0.6)]'
              : 'left-5 bg-amber-400 shadow-[0_0_6px_rgba(251,191,36,0.6)]',
          )}
        />
      </div>

      {/* Moon icon */}
      <svg
        xmlns="http://www.w3.org/2000/svg"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
        className={cn(
          'w-4 h-4 transition-all duration-300',
          !isDark ? 'text-indigo-400 scale-100' : 'text-zinc-500 scale-75',
        )}
      >
        <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
      </svg>
    </button>
  );
}

// =============================================================================
// Button — Reusable button with neon/dark theme variants
// SRP: Render a styled <button> with variant/size/disabled support
// DIP: Zero external deps, all config via props
// =============================================================================

import { type ButtonHTMLAttributes, forwardRef } from 'react';
import { cn } from '../../lib/cn';

export type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'neon';
export type ButtonSize = 'sm' | 'md' | 'lg';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
}

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    'bg-emerald-600 text-white hover:bg-emerald-500 active:bg-emerald-700 shadow-[0_0_12px_rgba(16,185,129,0.3)] hover:shadow-[0_0_20px_rgba(16,185,129,0.5)]',
  secondary:
    'bg-zinc-700 text-zinc-200 hover:bg-zinc-600 active:bg-zinc-800 border border-zinc-600',
  danger:
    'bg-red-600 text-white hover:bg-red-500 active:bg-red-700 shadow-[0_0_12px_rgba(239,68,68,0.3)] hover:shadow-[0_0_20px_rgba(239,68,68,0.5)]',
  ghost:
    'bg-transparent text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 active:bg-zinc-700',
  neon:
    'bg-transparent text-cyan-400 border border-cyan-500/50 hover:bg-cyan-500/10 active:bg-cyan-500/20 shadow-[0_0_8px_rgba(6,182,212,0.2)] hover:shadow-[0_0_16px_rgba(6,182,212,0.4)]',
};

const sizeClasses: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm rounded-md',
  md: 'px-4 py-2 text-base rounded-lg',
  lg: 'px-6 py-3 text-lg rounded-xl',
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = 'primary', size = 'md', className, disabled, children, ...props }, ref) => {
    return (
      <button
        ref={ref}
        disabled={disabled}
        className={cn(
          'inline-flex items-center justify-center font-semibold transition-all duration-200',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400 focus-visible:ring-offset-2 focus-visible:ring-offset-zinc-900',
          'disabled:opacity-40 disabled:cursor-not-allowed disabled:pointer-events-none',
          'select-none',
          variantClasses[variant],
          sizeClasses[size],
          className,
        )}
        {...props}
      >
        {children}
      </button>
    );
  },
);

Button.displayName = 'Button';

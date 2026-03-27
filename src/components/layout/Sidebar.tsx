// =============================================================================
// Sidebar — Navigation sidebar with neon/dark theme
// SRP: Render navigation links, highlight active route
// DIP: Receives items + active path + onNavigate callback, zero routing deps
// =============================================================================

import { cn } from '../../lib/cn';

export interface SidebarItem {
  id: string;
  label: string;
  icon: React.ReactNode;
}

export interface SidebarProps {
  items: SidebarItem[];
  activeId: string;
  onNavigate: (id: string) => void;
  className?: string;
}

export function Sidebar({ items, activeId, onNavigate, className }: SidebarProps) {
  return (
    <nav
      className={cn(
        'flex flex-col w-16 lg:w-56 h-full bg-zinc-950 border-r border-zinc-800',
        'py-4 px-2 lg:px-3 gap-1',
        className,
      )}
      aria-label="Main navigation"
    >
      {/* Logo */}
      <div className="flex items-center justify-center lg:justify-start gap-2 px-2 mb-6">
        <span className="text-2xl">⚽</span>
        <span className="hidden lg:block text-sm font-bold text-cyan-400 tracking-wide">
          E-Soccer
        </span>
      </div>

      {/* Nav items */}
      <ul className="flex flex-col gap-1 flex-1" role="list">
        {items.map((item) => {
          const isActive = item.id === activeId;
          return (
            <li key={item.id}>
              <button
                type="button"
                onClick={() => onNavigate(item.id)}
                className={cn(
                  'flex items-center justify-center lg:justify-start gap-3 w-full',
                  'px-2 py-2.5 rounded-lg text-sm font-medium transition-all duration-200',
                  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-cyan-400',
                  isActive
                    ? 'bg-cyan-500/10 text-cyan-400 border border-cyan-500/30 shadow-[0_0_12px_rgba(6,182,212,0.15)]'
                    : 'text-zinc-400 hover:bg-zinc-800 hover:text-zinc-200 border border-transparent',
                )}
                aria-current={isActive ? 'page' : undefined}
              >
                <span className={cn('w-5 h-5 shrink-0', isActive && 'drop-shadow-[0_0_4px_rgba(6,182,212,0.6)]')}>
                  {item.icon}
                </span>
                <span className="hidden lg:block">{item.label}</span>
              </button>
            </li>
          );
        })}
      </ul>
    </nav>
  );
}

import { cn } from '../../lib/utils';
import type { Page } from '../../types';

interface SidebarProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
}

const navItems: { page: Page; label: string; icon: string }[] = [
  { page: 'match', label: 'Partida', icon: '⚽' },
  { page: 'settings', label: 'Configurações', icon: '⚙️' },
  { page: 'history', label: 'Histórico', icon: '📊' },
  { page: 'help', label: 'Ajuda', icon: '❓' },
];

export function Sidebar({ currentPage, onNavigate }: SidebarProps) {
  return (
    <nav className="flex h-full w-16 flex-col items-center gap-2 border-r border-[var(--border-color)] bg-[var(--bg-secondary)] py-4 sm:w-48">
      <div className="mb-4 text-lg font-bold text-neon-green">⚽</div>
      {navItems.map((item) => (
        <button
          key={item.page}
          onClick={() => onNavigate(item.page)}
          className={cn(
            'flex w-full items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium transition-colors',
            'hover:bg-[var(--bg-card)]',
            currentPage === item.page
              ? 'bg-blue-600 text-white hover:bg-blue-700'
              : 'text-[var(--text-secondary)]',
          )}
        >
          <span>{item.icon}</span>
          <span className="hidden sm:inline">{item.label}</span>
        </button>
      ))}
    </nav>
  );
}

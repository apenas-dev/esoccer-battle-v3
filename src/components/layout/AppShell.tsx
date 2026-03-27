import { Sidebar } from './Sidebar';
import { ThemeToggle } from '../ui/ThemeToggle';
import type { Page, Theme } from '../../types';

interface AppShellProps {
  currentPage: Page;
  onNavigate: (page: Page) => void;
  theme: Theme;
  onToggleTheme: () => void;
  children: React.ReactNode;
}

export function AppShell({ currentPage, onNavigate, theme, onToggleTheme, children }: AppShellProps) {
  return (
    <div className="flex h-screen overflow-hidden bg-[var(--bg-primary)]">
      <Sidebar currentPage={currentPage} onNavigate={onNavigate} />
      <div className="flex flex-1 flex-col overflow-hidden">
        <header className="flex items-center justify-between border-b border-[var(--border-color)] bg-[var(--bg-secondary)] px-4 py-3">
          <h1 className="text-lg font-bold text-[var(--text-primary)]">E-Soccer Battle V3</h1>
          <ThemeToggle theme={theme} onToggle={onToggleTheme} />
        </header>
        <main className="flex-1 overflow-y-auto p-4">{children}</main>
      </div>
    </div>
  );
}

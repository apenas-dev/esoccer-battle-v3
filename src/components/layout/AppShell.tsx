// =============================================================================
// AppShell — Application shell (sidebar + content area)
// SRP: Compose Sidebar + main content slot with consistent layout
// DIP: Receives all data via props, renders children in content area
// =============================================================================

import { type ReactNode } from 'react';
import { cn } from '../../lib/cn';
import type { SidebarItem } from './Sidebar';
import { Sidebar } from './Sidebar';

export interface AppShellProps {
  sidebarItems: SidebarItem[];
  activeRoute: string;
  onNavigate: (id: string) => void;
  children: ReactNode;
  className?: string;
}

export function AppShell({ sidebarItems, activeRoute, onNavigate, children, className }: AppShellProps) {
  return (
    <div className={cn('flex h-screen w-screen overflow-hidden bg-zinc-900 text-zinc-100', className)}>
      {/* Sidebar */}
      <Sidebar items={sidebarItems} activeId={activeRoute} onNavigate={onNavigate} />

      {/* Main content */}
      <main className="flex-1 overflow-y-auto overflow-x-hidden">
        {children}
      </main>
    </div>
  );
}

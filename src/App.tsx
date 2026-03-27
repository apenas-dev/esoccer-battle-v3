import { useState, useEffect } from 'react';
import { AppShell } from './components/layout/AppShell';
import { MatchPage } from './pages/MatchPage';
import { SettingsPage } from './pages/SettingsPage';
import { HistoryPage } from './pages/HistoryPage';
import { HelpPage } from './pages/HelpPage';
import type { Page, Theme } from './types';

export default function App() {
  const [page, setPage] = useState<Page>('match');
  const [theme, setTheme] = useState<Theme>(() => {
    return (localStorage.getItem('theme') as Theme) || 'dark';
  });

  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'dark') {
      root.classList.add('dark');
    } else {
      root.classList.remove('dark');
    }
    localStorage.setItem('theme', theme);
  }, [theme]);

  const toggleTheme = () => {
    setTheme((t) => (t === 'dark' ? 'light' : 'dark'));
  };

  const renderPage = () => {
    switch (page) {
      case 'match':
        return <MatchPage />;
      case 'settings':
        return <SettingsPage />;
      case 'history':
        return <HistoryPage />;
      case 'help':
        return <HelpPage />;
    }
  };

  return (
    <AppShell currentPage={page} onNavigate={setPage} theme={theme} onToggleTheme={toggleTheme}>
      {renderPage()}
    </AppShell>
  );
}

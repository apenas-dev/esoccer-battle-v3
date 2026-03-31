import { useState } from 'react';
import { Sidebar, type PageId } from './Sidebar';
import { MatchPage } from '../../pages/MatchPage';
import { SettingsPage } from '../../pages/SettingsPage';
import { HistoryPage } from '../../pages/HistoryPage';
import { HelpPage } from '../../pages/HelpPage';

export function AppShell() {
  const [currentPage, setCurrentPage] = useState<PageId>('match');

  const renderPage = () => {
    switch (currentPage) {
      case 'match': return <MatchPage />;
      case 'settings': return <SettingsPage />;
      case 'history': return <HistoryPage />;
      case 'help': return <HelpPage />;
    }
  };

  return (
    <div className="flex min-h-screen bg-gray-950">
      <Sidebar currentPage={currentPage} onNavigate={setCurrentPage} />
      <main className="flex-1 overflow-y-auto">
        {renderPage()}
      </main>
    </div>
  );
}

import { lazy, Suspense, useState } from 'react';
import { isTauri } from './lib/tauri';

const MatchPageConnected = lazy(() =>
  isTauri()
    ? import('./pages/MatchPageConnected').then((m) => ({ default: m.MatchPageConnected }))
    : Promise.resolve({ default: () => <></> })
);

const SettingsPage = lazy(() =>
  isTauri()
    ? import('./pages/SettingsPage').then((m) => ({ default: m.SettingsPage }))
    : Promise.resolve({ default: () => <></> })
);

const HistoryPage = lazy(() =>
  isTauri()
    ? import('./pages/HistoryPage').then((m) => ({ default: m.HistoryPage }))
    : Promise.resolve({ default: () => <></> })
);

const HelpPage = lazy(() =>
  isTauri()
    ? import('./pages/HelpPage').then((m) => ({ default: m.HelpPage }))
    : Promise.resolve({ default: () => <></> })
);

function Fallback() {
  return (
    <div className="min-h-screen bg-[#0a0f1a] text-white flex items-center justify-center">
      <p className="text-gray-500">Carregando...</p>
    </div>
  );
}

type Page = 'match' | 'settings' | 'help' | 'history';

export function App() {
  const [page, setPage] = useState<Page>('match');

  if (!isTauri()) {
    // Modo não-Tauri: renderiza MatchPage genérica
    const MatchPage = lazy(() => import('./components/match').then((m) => ({ default: m.MatchPage })));
    return (
      <Suspense fallback={<Fallback />}>
        <MatchPage />
      </Suspense>
    );
  }

  return (
    <Suspense fallback={<Fallback />}>
      {page === 'match' ? (
        <MatchPageConnected
          onNavigateSettings={() => setPage('settings')}
          onNavigateHelp={() => setPage('help')}
          onNavigateHistory={() => setPage('history')}
        />
      ) : page === 'help' ? (
        <HelpPage onBack={() => setPage('match')} />
      ) : page === 'history' ? (
        <HistoryPage onBack={() => setPage('match')} />
      ) : (
        <SettingsPage onBack={() => setPage('match')} />
      )}
    </Suspense>
  );
}

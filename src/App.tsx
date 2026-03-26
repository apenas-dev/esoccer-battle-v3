import { lazy, Suspense } from 'react';
import { isTauri } from './lib/tauri';
import { MatchPage } from './components/match';

const MatchPageConnected = lazy(() =>
  isTauri()
    ? import('./pages/MatchPageConnected').then((m) => ({ default: m.MatchPageConnected }))
    : Promise.resolve({ default: () => <></> })
);

function Fallback() {
  return (
    <div className="min-h-screen bg-[#0a0f1a] text-white flex items-center justify-center">
      <p className="text-gray-500">Carregando...</p>
    </div>
  );
}

export function App() {
  if (isTauri()) {
    return (
      <Suspense fallback={<Fallback />}>
        <MatchPageConnected />
      </Suspense>
    );
  }

  return <MatchPage />;
}

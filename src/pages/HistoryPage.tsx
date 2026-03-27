import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Button } from '../components/ui/Button';
import type { HistoryEntry } from '../types';
import { formatDateTime } from '../lib/utils';

export function HistoryPage() {
  const [entries, setEntries] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);

  const loadHistory = async () => {
    try {
      const data = await invoke<HistoryEntry[]>('get_history');
      setEntries(data ?? []);
    } catch {
      setEntries([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadHistory();
  }, []);

  const handleRemove = async (id: string) => {
    try {
      await invoke('remove_history', { id });
      setEntries((prev) => prev.filter((e) => e.id !== id));
    } catch (e) {
      console.error('Remove failed:', e);
    }
  };

  const handleClear = async () => {
    if (!confirm('Limpar todo o histórico?')) return;
    try {
      await invoke('clear_history');
      setEntries([]);
    } catch (e) {
      console.error('Clear failed:', e);
    }
  };

  if (loading) {
    return <div className="text-center text-[var(--text-secondary)]">Carregando...</div>;
  }

  return (
    <div className="mx-auto max-w-2xl space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-xl font-bold">Histórico</h2>
        {entries.length > 0 && (
          <Button variant="danger" onClick={handleClear}>
            🗑️ Limpar
          </Button>
        )}
      </div>

      {entries.length === 0 ? (
        <div className="rounded-lg border border-dashed border-[var(--border-color)] p-8 text-center text-[var(--text-secondary)]">
          Nenhuma partida no histórico
        </div>
      ) : (
        <div className="overflow-x-auto rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)]">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-[var(--border-color)] text-left text-xs text-[var(--text-secondary)]">
                <th className="px-4 py-3">Data</th>
                <th className="px-4 py-3">Times</th>
                <th className="px-4 py-3 text-center">Placar</th>
                <th className="px-4 py-3">Duração</th>
                <th className="px-4 py-3"></th>
              </tr>
            </thead>
            <tbody>
              {entries.map((entry) => (
                <tr key={entry.id} className="border-b border-[var(--border-color)] last:border-b-0">
                  <td className="px-4 py-3 text-[var(--text-secondary)]">
                    {formatDateTime(entry.finished_at)}
                  </td>
                  <td className="px-4 py-3 font-medium">
                    {entry.team_a_name} × {entry.team_b_name}
                  </td>
                  <td className="px-4 py-3 text-center">
                    <span className="font-bold text-neon-green">{entry.score_a}</span>
                    <span className="mx-1 text-[var(--text-secondary)]">×</span>
                    <span className="font-bold text-neon-green">{entry.score_b}</span>
                  </td>
                  <td className="px-4 py-3 text-[var(--text-secondary)]">
                    {Math.floor(entry.duration_secs / 60)}min
                  </td>
                  <td className="px-4 py-3">
                    <button
                      onClick={() => handleRemove(entry.id)}
                      className="text-[var(--text-secondary)] hover:text-neon-red transition-colors"
                      aria-label="Remover"
                    >
                      ✕
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

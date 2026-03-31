import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { HistoryEntry } from '../types';

export function HistoryPage() {
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<HistoryEntry[]>('get_history', { limit: 50 })
      .then(setHistory)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return <div className="p-8 text-gray-400">Carregando histórico...</div>;
  }

  return (
    <div className="max-w-4xl mx-auto p-8">
      <h2 className="text-xl font-bold text-gray-100 mb-6">📊 Histórico de Partidas</h2>

      {history.length === 0 ? (
        <div className="bg-gray-900 border border-gray-800 rounded-lg p-8 text-center text-gray-500">
          Nenhuma partida registrada ainda.
        </div>
      ) : (
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-gray-800 text-gray-400">
                <th className="text-left py-3 px-4 font-medium">Data</th>
                <th className="text-left py-3 px-4 font-medium">Time A</th>
                <th className="text-center py-3 px-4 font-medium">Placar</th>
                <th className="text-left py-3 px-4 font-medium">Time B</th>
                <th className="text-right py-3 px-4 font-medium">Duração</th>
              </tr>
            </thead>
            <tbody>
              {history.map((entry) => {
                const winner =
                  entry.score_a > entry.score_b ? entry.team_a_name :
                  entry.score_b > entry.score_a ? entry.team_b_name : null;
                const date = new Date(entry.finished_at).toLocaleDateString('pt-BR', {
                  day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit',
                });
                const mins = Math.floor(entry.duration_secs / 60);
                const secs = entry.duration_secs % 60;
                const duration = `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`;

                return (
                  <tr key={entry.id} className="border-b border-gray-800/50 hover:bg-gray-900/50">
                    <td className="py-3 px-4 text-gray-400">{date}</td>
                    <td className={`py-3 px-4 font-medium ${winner === entry.team_a_name ? 'text-green-400' : 'text-gray-200'}`}>
                      {entry.team_a_name}
                    </td>
                    <td className="py-3 px-4 text-center font-mono font-bold text-gray-100">
                      {entry.score_a} × {entry.score_b}
                    </td>
                    <td className={`py-3 px-4 font-medium ${winner === entry.team_b_name ? 'text-green-400' : 'text-gray-200'}`}>
                      {entry.team_b_name}
                    </td>
                    <td className="py-3 px-4 text-right text-gray-400">{duration}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

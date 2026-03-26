import { useState, useEffect, useCallback } from 'react';
import { motion } from 'framer-motion';
import { getMatchHistory, clearMatchHistory, type MatchRecord } from '../lib/tauri';

interface HistoryPageProps {
  onBack: () => void;
}

function formatDuration(secs: number): string {
  const m = Math.floor(secs / 60);
  const s = secs % 60;
  return `${m}:${s.toString().padStart(2, '0')}`;
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleDateString('pt-BR', { day: '2-digit', month: '2-digit', year: '2-digit', hour: '2-digit', minute: '2-digit' });
  } catch {
    return iso;
  }
}

export function HistoryPage({ onBack }: HistoryPageProps) {
  const [records, setRecords] = useState<MatchRecord[]>([]);
  const [loading, setLoading] = useState(true);

  const load = useCallback(async () => {
    setLoading(true);
    try {
      const history = await getMatchHistory();
      setRecords(history.reverse());
    } catch (e) {
      console.error('Failed to load history:', e);
    }
    setLoading(false);
  }, []);

  useEffect(() => { load(); }, [load]);

  const handleClear = async () => {
    if (!confirm('Limpar todo o histórico de partidas?')) return;
    try {
      await clearMatchHistory();
      setRecords([]);
    } catch (e) {
      console.error('Failed to clear history:', e);
    }
  };

  return (
    <div className="min-h-screen bg-[#0a0f1a] text-white flex flex-col items-center px-4 py-6 sm:px-6 sm:py-8">
      <motion.header initial={{ opacity: 0, y: -20 }} animate={{ opacity: 1, y: 0 }} className="w-full max-w-3xl mb-6 flex items-center justify-between">
        <button onClick={onBack} className="text-gray-400 hover:text-white transition-colors p-1" aria-label="Voltar">← Voltar</button>
        <h1 className="text-lg sm:text-xl font-bold"><span className="text-[#00ff88]">📋</span> Histórico</h1>
        {records.length > 0 && (
          <button onClick={handleClear} className="text-gray-500 hover:text-red-400 transition-colors text-sm" aria-label="Limpar histórico">Limpar</button>
        )}
      </motion.header>

      <main className="w-full max-w-3xl flex flex-col gap-3">
        {loading ? (
          <p className="text-gray-500 text-center py-8">Carregando...</p>
        ) : records.length === 0 ? (
          <div className="text-center py-16">
            <p className="text-gray-600 text-4xl mb-4">⚽</p>
            <p className="text-gray-500">Nenhuma partida registrada</p>
          </div>
        ) : (
          records.map((r, i) => {
            const winner = r.score_a > r.score_b ? 'a' : r.score_b > r.score_a ? 'b' : null;
            return (
              <motion.div
                key={r.id}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.05 }}
                className="bg-[#0d1117] border border-[#1e3a5f] rounded-xl p-4 flex items-center justify-between"
              >
                <div className="flex-1 min-w-0 text-right">
                  <span className={`font-bold truncate block ${winner === 'a' ? 'text-cyan-400' : 'text-gray-300'}`}>{r.team_a_name}</span>
                </div>
                <div className="flex-shrink-0 mx-3 text-center">
                  <span className="text-2xl font-black">
                    <span className={winner === 'a' ? 'text-cyan-400' : 'text-gray-400'}>{r.score_a}</span>
                    <span className="text-gray-600 mx-1">×</span>
                    <span className={winner === 'b' ? 'text-red-400' : 'text-gray-400'}>{r.score_b}</span>
                  </span>
                  {winner === null && <p className="text-xs text-amber-400 mt-0.5">Empate</p>}
                </div>
                <div className="flex-1 min-w-0 text-left">
                  <span className={`font-bold truncate block ${winner === 'b' ? 'text-red-400' : 'text-gray-300'}`}>{r.team_b_name}</span>
                </div>
                <div className="flex-shrink-0 ml-4 text-right">
                  <p className="text-xs text-gray-500">{formatDate(r.finished_at)}</p>
                  <p className="text-xs text-gray-600">⏱ {formatDuration(r.duration_secs)}</p>
                </div>
              </motion.div>
            );
          })
        )}
      </main>
    </div>
  );
}

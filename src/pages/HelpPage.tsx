import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { CommandHelp } from '../types';

export function HelpPage() {
  const [commands, setCommands] = useState<CommandHelp[]>([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState('');

  useEffect(() => {
    invoke<CommandHelp[]>('get_available_commands')
      .then(setCommands)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const filtered = filter
    ? commands.filter(c =>
        c.command.toLowerCase().includes(filter.toLowerCase()) ||
        c.description.toLowerCase().includes(filter.toLowerCase()) ||
        c.aliases.some(a => a.toLowerCase().includes(filter.toLowerCase()))
      )
    : commands;

  if (loading) {
    return <div className="p-8 text-gray-400">Carregando comandos...</div>;
  }

  return (
    <div className="max-w-2xl mx-auto p-8">
      <h2 className="text-xl font-bold text-gray-100 mb-6">❓ Comandos de Voz</h2>

      <input
        type="text"
        placeholder="Buscar comandos..."
        value={filter}
        onChange={e => setFilter(e.target.value)}
        className="w-full px-4 py-2 mb-6 bg-gray-900 border border-gray-800 rounded-lg text-gray-100 text-sm placeholder-gray-500 focus:border-blue-500 focus:outline-none"
      />

      <div className="space-y-3">
        {filtered.map((cmd) => (
          <div key={cmd.command} className="bg-gray-900 border border-gray-800 rounded-lg p-4">
            <div className="flex items-start justify-between">
              <h3 className="font-mono font-semibold text-blue-400 text-sm">{cmd.command}</h3>
              {cmd.aliases.length > 0 && (
                <div className="flex gap-1">
                  {cmd.aliases.map(alias => (
                    <span key={alias} className="px-2 py-0.5 bg-gray-800 text-gray-400 text-xs rounded">
                      {alias}
                    </span>
                  ))}
                </div>
              )}
            </div>
            <p className="text-gray-400 text-sm mt-1">{cmd.description}</p>
          </div>
        ))}

        {filtered.length === 0 && (
          <p className="text-center text-gray-500 py-8">Nenhum comando encontrado para "{filter}"</p>
        )}
      </div>

      <div className="mt-8 bg-gray-900 border border-gray-800 rounded-lg p-4">
        <h3 className="text-sm font-semibold text-gray-300 mb-2">💡 Dicas</h3>
        <ul className="text-sm text-gray-400 space-y-1">
          <li>• Fale o comando claramente após pressionar o botão de voz</li>
          <li>• Você pode usar o comando principal ou qualquer um dos aliases</li>
          <li>• "Gol do time A" e "Gol pro A" são equivalentes</li>
        </ul>
      </div>
    </div>
  );
}

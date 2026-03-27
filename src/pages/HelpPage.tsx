import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { CommandHelp } from '../types';

export function HelpPage() {
  const [commands, setCommands] = useState<CommandHelp[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<CommandHelp[]>('get_available_commands')
      .then(setCommands)
      .catch(() => setCommands([]))
      .finally(() => setLoading(false));
  }, []);

  if (loading) {
    return <div className="text-center text-[var(--text-secondary)]">Carregando...</div>;
  }

  return (
    <div className="mx-auto max-w-lg space-y-4">
      <h2 className="text-xl font-bold">Comandos de Voz</h2>
      <p className="text-sm text-[var(--text-secondary)]">
        Use estes comandos por voz (PTT) ou pelos botões na tela.
      </p>

      <div className="space-y-2">
        {commands.map((cmd) => (
          <div
            key={cmd.command}
            className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] p-4"
          >
            <div className="flex items-center gap-2">
              <span className="rounded-lg bg-neon-blue/10 px-2 py-1 font-mono text-sm font-bold text-neon-blue">
                {cmd.command}
              </span>
            </div>
            <p className="mt-1 text-sm text-[var(--text-secondary)]">{cmd.description}</p>
            {cmd.aliases.length > 0 && (
              <div className="mt-2 flex flex-wrap gap-1">
                {cmd.aliases.map((alias) => (
                  <span
                    key={alias}
                    className="rounded-full bg-[var(--bg-secondary)] px-2 py-0.5 text-xs text-[var(--text-secondary)]"
                  >
                    {alias}
                  </span>
                ))}
              </div>
            )}
          </div>
        ))}
      </div>

      {commands.length === 0 && (
        <div className="rounded-lg border border-dashed border-[var(--border-color)] p-8 text-center text-[var(--text-secondary)]">
          Nenhum comando disponível
        </div>
      )}
    </div>
  );
}

import { useState } from 'react';

export interface CommandLogEntry {
  timestamp: string;
  command: string;
  result: string;
}

interface CommandLogProps {
  entries: CommandLogEntry[];
}

export function CommandLog({ entries }: CommandLogProps) {
  const [isExpanded, setIsExpanded] = useState(false);
  const visible = isExpanded ? entries : entries.slice(-5);

  return (
    <div className="mt-6 text-center">
      {entries.length > 0 && (
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="text-xs text-gray-500 hover:text-gray-400 mb-2"
        >
          {isExpanded ? '↑ Recolher' : `↓ Log (${entries.length})`}
        </button>
      )}
      <div className="max-h-48 overflow-y-auto space-y-1 text-xs text-gray-500">
        {visible.map((entry, i) => (
          <p key={i}>
            <span className="text-gray-600">[{entry.timestamp}]</span>{' '}
            <span className="text-gray-300">{entry.command}</span>{' '}
            → {entry.result}
          </p>
        ))}
      </div>
    </div>
  );
}

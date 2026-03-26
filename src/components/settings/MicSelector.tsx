import { SettingsCard } from './SettingsCard';

interface MicSelectorProps {
  microphones: string[];
  selected: string | null;
  onChange: (device: string | null) => void;
}

/** Seletor de microfone com opção de padrão do sistema */
export function MicSelector({ microphones, selected, onChange }: MicSelectorProps) {
  return (
    <SettingsCard title="Microfone" icon="🎤">
      <select
        value={selected ?? '__default__'}
        onChange={(e) => onChange(e.target.value === '__default__' ? null : e.target.value)}
        className="w-full bg-[#0a0f1a] border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                   focus:outline-none focus:border-[#00ff88] focus:ring-1 focus:ring-[#00ff88]/30
                   transition-colors cursor-pointer"
      >
        <option value="__default__">Padrão do sistema</option>
        {microphones.map((mic) => (
          <option key={mic} value={mic}>
            {mic}
          </option>
        ))}
      </select>
    </SettingsCard>
  );
}

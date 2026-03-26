import type { STTProviderName } from '../../services/stt';

interface STTProviderSelectorProps {
  value: STTProviderName;
  onChange: (value: STTProviderName) => void;
}

const options: { value: STTProviderName; label: string; description: string }[] = [
  { value: 'auto', label: 'Automático', description: 'Usa Web Speech quando disponível, senão Whisper' },
  { value: 'web-speech', label: 'Web Speech (Browser)', description: 'Reconhecimento nativo do navegador' },
  { value: 'whisper', label: 'Whisper (Local)', description: 'Modelo local via Tauri backend' },
];

export function STTProviderSelector({ value, onChange }: STTProviderSelectorProps) {
  return (
    <div className="bg-gray-900/60 rounded-xl p-4 border border-gray-800">
      <h3 className="text-sm font-semibold text-white mb-3">
        🎤 Motor de Reconhecimento de Voz
      </h3>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value as STTProviderName)}
        className="w-full bg-gray-800 border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                   focus:outline-none focus:ring-2 focus:ring-[#00ff88]/50 focus:border-[#00ff88]/50
                   transition-colors cursor-pointer"
        aria-label="Motor de reconhecimento de voz"
      >
        {options.map((opt) => (
          <option key={opt.value} value={opt.value}>
            {opt.label}
          </option>
        ))}
      </select>
      <p className="text-xs text-gray-500 mt-2">
        {options.find((o) => o.value === value)?.description}
      </p>
    </div>
  );
}

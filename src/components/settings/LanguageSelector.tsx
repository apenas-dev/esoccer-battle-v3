import { SettingsCard } from './SettingsCard';

interface LanguageSelectorProps {
  value: string;
  onChange: (lang: string) => void;
}

const LANGUAGES = [
  { value: 'pt', label: 'Português' },
  { value: 'en', label: 'English' },
];

/** Seletor de idioma */
export function LanguageSelector({ value, onChange }: LanguageSelectorProps) {
  return (
    <SettingsCard title="Idioma" icon="🌐">
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="w-full bg-[#0a0f1a] border border-gray-700 rounded-lg px-3 py-2 text-sm text-white
                   focus:outline-none focus:border-[#00ff88] focus:ring-1 focus:ring-[#00ff88]/30
                   transition-colors cursor-pointer"
      >
        {LANGUAGES.map((lang) => (
          <option key={lang.value} value={lang.value}>
            {lang.label}
          </option>
        ))}
      </select>
    </SettingsCard>
  );
}

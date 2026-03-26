import { SettingsCard } from './SettingsCard';

interface ThemeToggleProps {
  value: string;
  onChange: (theme: string) => void;
}

const THEMES = [
  { value: 'dark', label: 'Dark', icon: '🌙' },
  { value: 'light', label: 'Light', icon: '☀️' },
];

/** Toggle de tema (dark/light) */
export function ThemeToggle({ value, onChange }: ThemeToggleProps) {
  return (
    <SettingsCard title="Tema" icon="🎨">
      <div className="flex gap-2">
        {THEMES.map((theme) => {
          const isActive = theme.value === value;
          return (
            <button
              key={theme.value}
              onClick={() => onChange(theme.value)}
              disabled={theme.value === 'light'}
              className={`flex-1 flex items-center justify-center gap-2 px-3 py-2 rounded-lg border text-sm
                         transition-colors ${
                           isActive
                             ? 'border-[#00ff88]/40 bg-[#00ff88]/10 text-[#00ff88]'
                             : 'border-gray-700 bg-[#0a0f1a] text-gray-400 hover:border-gray-600'
                         }
                         disabled:opacity-30 disabled:cursor-not-allowed`}
              title={theme.value === 'light' ? 'Em breve' : undefined}
            >
              <span>{theme.icon}</span>
              <span>{theme.label}</span>
            </button>
          );
        })}
      </div>
      <p className="text-[10px] text-gray-600 mt-1.5">Light theme em breve</p>
    </SettingsCard>
  );
}

import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { AppConfig, WhisperModel, Language, TimerMode } from '../types';
import { Button } from '../components/ui/Button';

const WHISPER_MODELS: WhisperModel[] = ['tiny', 'base', 'small'];
const LANGUAGES: { value: Language; label: string }[] = [
  { value: 'pt_br', label: 'Português (BR)' },
  { value: 'en', label: 'English' },
  { value: 'es', label: 'Español' },
];
const TIMER_MODES: { value: TimerMode; label: string }[] = [
  { value: 'countdown', label: 'Regressivo' },
  { value: 'count_up', label: 'Progressivo' },
];

export function SettingsPage() {
  const [config, setConfig] = useState<AppConfig | null>(null);
  const [micDevices, setMicDevices] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<AppConfig>('get_config').then(setConfig);
    invoke<string[]>('list_mic_devices').then(setMicDevices).catch(() => {});
  }, []);

  const update = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
    setConfig(prev => prev ? { ...prev, [key]: value } : prev);
    setSaved(false);
  };

  const handleSave = async () => {
    if (!config) return;
    setSaving(true);
    try {
      await invoke('update_config', { newConfig: config });
      setSaved(true);
    } catch (e) {
      console.error('Save failed:', e);
    } finally {
      setSaving(false);
    }
  };

  if (!config) {
    return <div className="p-8 text-gray-400">Carregando configurações...</div>;
  }

  return (
    <div className="max-w-2xl mx-auto p-8">
      <h2 className="text-xl font-bold text-gray-100 mb-6">⚙️ Configurações</h2>

      <div className="space-y-5">
        {/* Teams */}
        <fieldset className="bg-gray-900 border border-gray-800 rounded-lg p-5">
          <legend className="text-sm font-semibold text-gray-300 px-2">Times</legend>
          <div className="grid grid-cols-2 gap-4">
            <label className="block">
              <span className="text-xs text-gray-400">Nome do Time A</span>
              <input
                type="text"
                value={config.team_a_name}
                onChange={e => update('team_a_name', e.target.value)}
                className="mt-1 w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-gray-100 text-sm focus:border-blue-500 focus:outline-none"
              />
            </label>
            <label className="block">
              <span className="text-xs text-gray-400">Nome do Time B</span>
              <input
                type="text"
                value={config.team_b_name}
                onChange={e => update('team_b_name', e.target.value)}
                className="mt-1 w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-gray-100 text-sm focus:border-blue-500 focus:outline-none"
              />
            </label>
          </div>
        </fieldset>

        {/* Match */}
        <fieldset className="bg-gray-900 border border-gray-800 rounded-lg p-5">
          <legend className="text-sm font-semibold text-gray-300 px-2">Partida</legend>
          <div className="grid grid-cols-2 gap-4">
            <label className="block">
              <span className="text-xs text-gray-400">Duração (segundos)</span>
              <input
                type="number"
                min={30}
                max={3600}
                value={config.match_duration_secs}
                onChange={e => update('match_duration_secs', Number(e.target.value))}
                className="mt-1 w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-gray-100 text-sm focus:border-blue-500 focus:outline-none"
              />
            </label>
            <label className="block">
              <span className="text-xs text-gray-400">Modo do Timer</span>
              <select
                value={config.timer_mode}
                onChange={e => update('timer_mode', e.target.value as TimerMode)}
                className="mt-1 w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-gray-100 text-sm focus:border-blue-500 focus:outline-none"
              >
                {TIMER_MODES.map(m => (
                  <option key={m.value} value={m.value}>{m.label}</option>
                ))}
              </select>
            </label>
          </div>
        </fieldset>

        {/* Audio */}
        <fieldset className="bg-gray-900 border border-gray-800 rounded-lg p-5">
          <legend className="text-sm font-semibold text-gray-300 px-2">Áudio & Voz</legend>
          <div className="space-y-4">
            <label className="block">
              <span className="text-xs text-gray-400">Volume: {Math.round(config.volume * 100)}%</span>
              <input
                type="range"
                min={0}
                max={1}
                step={0.05}
                value={config.volume}
                onChange={e => update('volume', Number(e.target.value))}
                className="mt-1 w-full accent-blue-500"
              />
            </label>

            {micDevices.length > 0 && (
              <label className="block">
                <span className="text-xs text-gray-400">Microfone</span>
                <select
                  value={config.mic_device || ''}
                  onChange={e => update('mic_device', e.target.value || null)}
                  className="mt-1 w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-gray-100 text-sm focus:border-blue-500 focus:outline-none"
                >
                  <option value="">Padrão</option>
                  {micDevices.map(d => (
                    <option key={d} value={d}>{d}</option>
                  ))}
                </select>
              </label>
            )}

            <label className="block">
              <span className="text-xs text-gray-400">Modelo Whisper</span>
              <select
                value={config.whisper_model}
                onChange={e => update('whisper_model', e.target.value as WhisperModel)}
                className="mt-1 w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-gray-100 text-sm focus:border-blue-500 focus:outline-none"
              >
                {WHISPER_MODELS.map(m => (
                  <option key={m} value={m}>{m}</option>
                ))}
              </select>
            </label>

            <label className="block">
              <span className="text-xs text-gray-400">Idioma</span>
              <select
                value={config.language}
                onChange={e => update('language', e.target.value as Language)}
                className="mt-1 w-full px-3 py-2 bg-gray-800 border border-gray-700 rounded text-gray-100 text-sm focus:border-blue-500 focus:outline-none"
              >
                {LANGUAGES.map(l => (
                  <option key={l.value} value={l.value}>{l.label}</option>
                ))}
              </select>
            </label>
          </div>
        </fieldset>
      </div>

      <div className="mt-6 flex items-center gap-3">
        <Button onClick={handleSave} disabled={saving}>
          {saving ? 'Salvando...' : '💾 Salvar'}
        </Button>
        {saved && <span className="text-sm text-green-400">✓ Salvo com sucesso</span>}
      </div>
    </div>
  );
}

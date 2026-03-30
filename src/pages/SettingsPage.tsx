import { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useMatchState } from '../hooks/useMatchState';
import { Button } from '../components/ui/Button';
import type { AppConfig, WhisperModel, Language, TimerMode, Theme } from '../types';

export function SettingsPage() {
  const { config, loadConfig, updateConfig } = useMatchState();
  const [form, setForm] = useState<AppConfig | null>(null);
  const [micDevices, setMicDevices] = useState<string[]>([]);
  const [saving, setSaving] = useState(false);
  const [saved, setSaved] = useState(false);
  // BUG 4 FIX: Track whether form has been modified by the user
  const [isDirty, setIsDirty] = useState(false);

  useEffect(() => {
    loadConfig();
    invoke<string[]>('list_mic_devices').then(setMicDevices).catch(() => {});
  }, [loadConfig]);

  useEffect(() => {
    if (config && !form) {
      setForm({ ...config });
    }
    // BUG 4 FIX: Sync form when config changes AND user hasn't edited
    if (config && form && !isDirty) {
      setForm({ ...config });
    }
  }, [config, form, isDirty]);

  if (!form) {
    return <div className="text-center text-[var(--text-secondary)]">Carregando configurações...</div>;
  }

  const update = <K extends keyof AppConfig>(key: K, value: AppConfig[K]) => {
    setForm((prev) => (prev ? { ...prev, [key]: value } : prev));
    setIsDirty(true);
    setSaved(false);
  };

  const handleSave = async () => {
    setSaving(true);
    await updateConfig(form);
    setSaving(false);
    setSaved(true);
    setIsDirty(false);
    setTimeout(() => setSaved(false), 2000);
  };

  const inputClass =
    'rounded-lg border border-[var(--border-color)] bg-[var(--bg-card)] px-3 py-2 text-sm text-[var(--text-primary)] focus:border-neon-blue focus:outline-none focus:ring-1 focus:ring-neon-blue';

  return (
    <div className="mx-auto max-w-lg space-y-6">
      <h2 className="text-xl font-bold">Configurações</h2>

      {/* Team Names */}
      <section className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] p-4 space-y-3">
        <h3 className="font-semibold">Times</h3>
        <div>
          <label className="text-xs text-[var(--text-secondary)]">Nome Time A</label>
          <input
            className={inputClass + ' w-full'}
            value={form.team_a_name}
            onChange={(e) => update('team_a_name', e.target.value)}
          />
        </div>
        <div>
          <label className="text-xs text-[var(--text-secondary)]">Nome Time B</label>
          <input
            className={inputClass + ' w-full'}
            value={form.team_b_name}
            onChange={(e) => update('team_b_name', e.target.value)}
          />
        </div>
      </section>

      {/* Match */}
      <section className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] p-4 space-y-3">
        <h3 className="font-semibold">Partida</h3>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-[var(--text-secondary)]">Duração (segundos)</label>
            <input
              type="number"
              min={30}
              max={600}
              step={30}
              className={inputClass + ' w-full'}
              value={form.match_duration_secs}
              onChange={(e) => {
                const val = Number(e.target.value);
                if (!Number.isNaN(val) && val >= 30 && val <= 600) {
                  update('match_duration_secs', val);
                }
              }}
            />
          </div>
          <div>
            <label className="text-xs text-[var(--text-secondary)]">Modo Timer</label>
            <select
              className={inputClass + ' w-full'}
              value={form.timer_mode}
              onChange={(e) => update('timer_mode', e.target.value as TimerMode)}
            >
              <option value="countdown">Regressivo</option>
              <option value="countup">Progressivo</option>
            </select>
          </div>
        </div>
      </section>

      {/* Voice */}
      <section className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] p-4 space-y-3">
        <h3 className="font-semibold">Voz</h3>
        <div>
          <label className="text-xs text-[var(--text-secondary)]">Microfone</label>
          <select
            className={inputClass + ' w-full'}
            value={form.mic_device ?? ''}
            onChange={(e) => update('mic_device', e.target.value || null)}
          >
            <option value="">Padrão</option>
            {micDevices.map((d) => (
              <option key={d} value={d}>
                {d}
              </option>
            ))}
          </select>
        </div>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-[var(--text-secondary)]">Modelo Whisper</label>
            <select
              className={inputClass + ' w-full'}
              value={form.whisper_model}
              onChange={(e) => update('whisper_model', e.target.value as WhisperModel)}
            >
              <option value="tiny">Tiny</option>
              <option value="base">Base</option>
              <option value="small">Small</option>
            </select>
          </div>
          <div>
            <label className="text-xs text-[var(--text-secondary)]">Idioma</label>
            <select
              className={inputClass + ' w-full'}
              value={form.language}
              onChange={(e) => update('language', e.target.value as Language)}
            >
              <option value="pt_br">Português (BR)</option>
              <option value="en">English</option>
              <option value="es">Español</option>
            </select>
          </div>
        </div>
        <div>
          <label className="text-xs text-[var(--text-secondary)]">
            Sensibilidade Voz: {form.voice_threshold}
          </label>
          <input
            type="range"
            min="0"
            max="1"
            step="0.05"
            value={form.voice_threshold}
            onChange={(e) => update('voice_threshold', Number(e.target.value))}
            className="w-full"
          />
        </div>
      </section>

      {/* Appearance */}
      <section className="rounded-xl border border-[var(--border-color)] bg-[var(--bg-card)] p-4 space-y-3">
        <h3 className="font-semibold">Aparência</h3>
        <div className="grid grid-cols-2 gap-3">
          <div>
            <label className="text-xs text-[var(--text-secondary)]">Tema</label>
            <select
              className={inputClass + ' w-full'}
              value={form.theme}
              onChange={(e) => update('theme', e.target.value as Theme)}
            >
              <option value="dark">Dark</option>
              <option value="light">Light</option>
            </select>
          </div>
          <div>
            <label className="text-xs text-[var(--text-secondary)]">
              Volume: {Math.round(form.volume * 100)}%
            </label>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={form.volume}
              onChange={(e) => update('volume', Number(e.target.value))}
              className="w-full mt-1"
            />
          </div>
        </div>
      </section>

      {/* Save */}
      <div className="flex items-center gap-3">
        <Button variant="neon" onClick={handleSave} disabled={saving}>
          {saving ? 'Salvando...' : '💾 Salvar'}
        </Button>
        {saved && <span className="text-sm text-neon-green">✓ Salvo!</span>}
      </div>
    </div>
  );
}

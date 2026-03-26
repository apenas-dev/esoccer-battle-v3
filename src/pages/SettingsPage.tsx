import { useState, useEffect, useCallback, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  getSettings,
  setSettings,
  listMicrophone,
  listModels,
  downloadModel,
  type AppSettings,
  type WhisperModel,
} from '../lib/tauri';
import { listen } from '@tauri-apps/api/event';
import { MicSelector } from '../components/settings/MicSelector';
import { ModelDownloader } from '../components/settings/ModelDownloader';
import { LanguageSelector } from '../components/settings/LanguageSelector';
import { VoiceThresholdSlider } from '../components/settings/VoiceThresholdSlider';
import { ThemeToggle } from '../components/settings/ThemeToggle';
import { TeamNames } from '../components/settings/TeamNames';

// ── Tipos ─────────────────────────────────────────────

interface SettingsPageProps {
  onBack: () => void;
}

// ── Toast ─────────────────────────────────────────────

type ToastType = 'success' | 'error';

interface Toast {
  id: number;
  message: string;
  type: ToastType;
}

// ── Componente ────────────────────────────────────────

export function SettingsPage({ onBack }: SettingsPageProps) {
  // Settings state
  const [settings, setLocalSettings] = useState<AppSettings>({
    mic_device: null,
    model: '',
    language: 'pt',
    voice_threshold: 0.5,
    theme: 'dark',
    team_a_name: 'Time A',
    team_b_name: 'Time B',
  });

  // Dados externos
  const [microphones, setMicrophones] = useState<string[]>([]);
  const [models, setModels] = useState<WhisperModel[]>([]);


  // Download state
  const [downloading, setDownloading] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<Record<string, number>>({});
  const downloadUnlistenRef = useRef<(() => void) | null>(null);

  // Toast state
  const [toast, setToast] = useState<Toast | null>(null);
  const toastIdRef = useRef(0);

  // Debounce ref para updateSettings
  const saveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // ── Carregar dados iniciais ────────────────────────

  useEffect(() => {
    (async () => {
      try {
        const [s, mics, mdls] = await Promise.all([
          getSettings(),
          listMicrophone(),
          listModels(),
        ]);
        setLocalSettings(s);
        setMicrophones(mics.map((d) => d.name));
        setModels(mdls);
      } catch (err) {
        showToast('Erro ao carregar configurações', 'error');
        console.error(err);
      }
    })();
  }, []);

  // ── Helpers ─────────────────────────────────────────

  function showToast(message: string, type: ToastType) {
    const id = ++toastIdRef.current;
    setToast({ id, message, type });
    setTimeout(() => setToast((t) => (t?.id === id ? null : t)), 3000);
  }

  const updateSettings = useCallback(
    (partial: Partial<AppSettings>) => {
      const updated = { ...settings, ...partial };
      setLocalSettings(updated);

      // Debounce: salva no backend após 500ms sem mudanças
      if (saveTimerRef.current) clearTimeout(saveTimerRef.current);
      saveTimerRef.current = setTimeout(() => {
        setSettings(updated)
          .then(() => showToast('Salvo com sucesso!', 'success'))
          .catch(() => showToast('Erro ao salvar', 'error'));
      }, 500);
    },
    [settings],
  );

  const handleDownload = useCallback(
    async (modelName: string) => {
      // BUG-2/BUG-4: Impedir download duplicado
      if (downloading !== null) return;

      try {
        setDownloading(modelName);
        setDownloadProgress((prev) => ({ ...prev, [modelName]: 0 }));

        // BUG-1: Limpar listener anterior (se houver)
        if (downloadUnlistenRef.current) {
          downloadUnlistenRef.current();
          downloadUnlistenRef.current = null;
        }

        const channelName = await downloadModel(modelName);

        // BUG-1: Usar listener do canal específico com eventos de progress/done/error
        const unlisten = await listen<{
          type: 'progress' | 'done' | 'error';
          value: string | number;
        }>(channelName, (e) => {
          const payload = e.payload;
          switch (payload.type) {
            case 'progress':
              setDownloadProgress((prev) => ({
                ...prev,
                [modelName]: Number(payload.value),
              }));
              break;
            case 'done':
              setDownloadProgress((prev) => ({ ...prev, [modelName]: 100 }));
              setDownloading((d) => (d === modelName ? null : d));
              if (downloadUnlistenRef.current) {
                downloadUnlistenRef.current();
                downloadUnlistenRef.current = null;
              }
              listModels().then(setModels);
              showToast(`${modelName} baixado com sucesso!`, 'success');
              break;
            case 'error':
              setDownloading((d) => (d === modelName ? null : d));
              setDownloadProgress((prev) => ({ ...prev, [modelName]: 0 }));
              if (downloadUnlistenRef.current) {
                downloadUnlistenRef.current();
                downloadUnlistenRef.current = null;
              }
              showToast(`Erro ao baixar ${modelName}: ${payload.value}`, 'error');
              break;
          }
        });

        downloadUnlistenRef.current = unlisten;
      } catch (err) {
        setDownloading((d) => (d === modelName ? null : d));
        setDownloadProgress((prev) => ({ ...prev, [modelName]: 0 }));
        showToast(`Erro ao baixar ${modelName}`, 'error');
        console.error(err);
      }
    },
    [downloading],
  );

  // ── Render ──────────────────────────────────────────

  return (
    <div className="min-h-screen bg-[#0a0f1a] text-white flex flex-col items-center px-4 py-6 sm:px-6 sm:py-8">
      {/* Header */}
      <motion.header
        initial={{ opacity: 0, y: -20 }}
        animate={{ opacity: 1, y: 0 }}
        className="w-full max-w-3xl mb-6 flex items-center gap-3"
      >
        <button
          onClick={onBack}
          className="text-gray-400 hover:text-white transition-colors p-1"
          aria-label="Voltar"
        >
          ← Voltar
        </button>
        <h1 className="text-lg sm:text-xl font-bold tracking-tight">
          <span className="text-[#00ff88]">⚙️</span>{' '}
          <span>Configurações</span>
        </h1>
      </motion.header>

      {/* Conteúdo */}
      <main className="w-full max-w-3xl space-y-4">
        <MicSelector
          microphones={microphones}
          selected={settings.mic_device}
          onChange={(device) => updateSettings({ mic_device: device })}
        />

        <ModelDownloader
          models={models}
          activeModel={settings.model}
          downloading={downloading}
          downloadProgress={downloadProgress}
          onSelect={(model) => updateSettings({ model })}
          onDownload={handleDownload}
        />

        <LanguageSelector
          value={settings.language}
          onChange={(lang) => updateSettings({ language: lang })}
        />

        <VoiceThresholdSlider
          value={settings.voice_threshold}
          onChange={(threshold) => updateSettings({ voice_threshold: threshold })}
        />

        <ThemeToggle
          value={settings.theme}
          onChange={(theme) => updateSettings({ theme })}
        />

        <TeamNames
          teamA={settings.team_a_name}
          teamB={settings.team_b_name}
          onChangeTeamA={(name) => updateSettings({ team_a_name: name })}
          onChangeTeamB={(name) => updateSettings({ team_b_name: name })}
        />
      </main>

      {/* Toast */}
      <AnimatePresence>
        {toast && (
          <motion.div
            initial={{ opacity: 0, y: 40 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 40 }}
            className={`fixed bottom-6 left-1/2 -translate-x-1/2 px-4 py-2.5 rounded-lg text-sm font-medium
                        shadow-lg z-50 ${
                          toast.type === 'success'
                            ? 'bg-[#00ff88]/15 text-[#00ff88] border border-[#00ff88]/30'
                            : 'bg-red-500/15 text-red-400 border border-red-500/30'
                        }`}
          >
            {toast.type === 'success' ? '✅' : '❌'} {toast.message}
          </motion.div>
        )}
      </AnimatePresence>

      <footer className="mt-auto pt-8 pb-2">
        <p className="text-xs text-gray-700 text-center">
          E-Soccer Battle V3 · Configurações
        </p>
      </footer>
    </div>
  );
}

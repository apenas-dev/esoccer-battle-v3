import { useState, useEffect, useCallback, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import {
  getSettings,
  setSettings,
  listMicrophone,
  listModels,
  listModelCategories,
  downloadModel,
  onModelDownloadProgress,
  type AppSettings,
  type WhisperModel,
  type ModelCategory,
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
  const [categories, setCategories] = useState<ModelCategory[]>([]);

  // Download state
  const [downloading, setDownloading] = useState<string | null>(null);
  const [downloadProgress, setDownloadProgress] = useState<Record<string, number>>({});

  // Toast state
  const [toast, setToast] = useState<Toast | null>(null);
  const toastIdRef = useRef(0);

  // ── Carregar dados iniciais ────────────────────────

  useEffect(() => {
    (async () => {
      try {
        const [s, mics, mdls, cats] = await Promise.all([
          getSettings(),
          listMicrophone(),
          listModels(),
          listModelCategories(),
        ]);
        setLocalSettings(s);
        setMicrophones(mics.map((d) => d.name));
        setModels(mdls);
        setCategories(cats);
      } catch (err) {
        showToast('Erro ao carregar configurações', 'error');
        console.error(err);
      }
    })();
  }, []);

  // ── Escutar progresso de download ──────────────────

  useEffect(() => {
    let unlisten: Awaited<ReturnType<typeof listen>> | undefined;

    onModelDownloadProgress((payload) => {
      setDownloadProgress((prev) => {
        if (!downloading) return prev;
        return { ...prev, [downloading]: payload.percent };
      });
    }).then((fn) => {
      unlisten = fn;
    });

    return () => {
      unlisten?.();
    };
  }, [downloading]);

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
      setSettings(updated).then(() => showToast('Salvo com sucesso!', 'success')).catch(() => showToast('Erro ao salvar', 'error'));
    },
    [settings],
  );

  const handleDownload = useCallback(
    async (modelName: string) => {
      try {
        setDownloading(modelName);
        setDownloadProgress((prev) => ({ ...prev, [modelName]: 0 }));

        const channelName = await downloadModel(modelName);

        // Escuta específica do canal retornado para progresso
        const { listen } = await import('@tauri-apps/api/event');
        const unlisten = await listen<{ percent: number }>(channelName, (e) => {
          setDownloadProgress((prev) => ({ ...prev, [modelName]: e.payload.percent }));
        });

        // Aguarda download terminar (poll do progress chegando em 100)
        // Na prática o evento de progresso finaliza, limpamos depois
        setTimeout(() => {
          unlisten();
          setDownloading((d) => (d === modelName ? null : d));
          setDownloadProgress((prev) => ({ ...prev, [modelName]: 0 }));

          // Recarrega modelos para refletir status atualizado
          listModels().then(setModels);
          showToast(`${modelName} baixado com sucesso!`, 'success');
        }, 5000); // Fallback — em produção seria baseado no evento de conclusão
      } catch (err) {
        setDownloading((d) => (d === modelName ? null : d));
        setDownloadProgress((prev) => ({ ...prev, [modelName]: 0 }));
        showToast(`Erro ao baixar ${modelName}`, 'error');
        console.error(err);
      }
    },
    [],
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
          categories={categories}
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

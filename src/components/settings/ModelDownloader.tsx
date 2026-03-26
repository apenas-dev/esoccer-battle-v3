import { SettingsCard } from './SettingsCard';
import { motion } from 'framer-motion';
import type { WhisperModel } from '../../lib/tauri';

interface ModelDownloaderProps {
  models: WhisperModel[];
  activeModel: string;
  downloading: string | null;
  downloadProgress: Record<string, number>;
  onSelect: (model: string) => void;
  onDownload: (model: string) => void;
}

/** Formata bytes para legível (KB, MB, GB) */
function formatSize(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
}

export function ModelDownloader({
  models,
  activeModel,
  downloading,
  downloadProgress,
  onSelect,
  onDownload,
}: ModelDownloaderProps) {
  // Mostrar APENAS MediumWhisper
  const filteredModels = models.filter((m) => m.type === 'MediumWhisper');

  return (
    <SettingsCard title="Modelo Whisper" icon="📥">
      <div className="space-y-2">
        {filteredModels.map((model) => {
          const isActive = model.type === activeModel;
          const isDownloading = downloading === model.type;
          const progress = downloadProgress[model.type] ?? 0;

          return (
            <motion.div
              key={model.type}
              className={`flex items-center gap-3 p-2.5 rounded-lg border transition-colors cursor-pointer ${
                isActive
                  ? 'border-[#00ff88]/40 bg-[#00ff88]/5'
                  : 'border-gray-800 bg-[#0a0f1a] hover:border-gray-700'
              }`}
              onClick={() => model.is_downloaded && onSelect(model.type)}
            >
              <div
                className={`w-2.5 h-2.5 rounded-full flex-shrink-0 ${
                  isActive ? 'bg-[#00ff88] shadow-[0_0_8px_#00ff88]' : 'bg-gray-600'
                }`}
              />
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-white truncate">
                  {model.type_name || model.name}
                  {isActive && (
                    <span className="ml-2 text-[10px] text-[#00ff88] font-normal">ATIVO</span>
                  )}
                </p>
                <div className="flex gap-3 text-[11px] text-gray-500">
                  <span>💾 {formatSize(model.disk_usage)}</span>
                  <span>🧠 {formatSize(model.mem_usage)}</span>
                </div>
                {isDownloading && (
                  <div className="mt-1.5 w-full bg-gray-800 rounded-full h-1.5 overflow-hidden">
                    <motion.div
                      className="h-full bg-[#00ff88] rounded-full"
                      initial={{ width: 0 }}
                      animate={{ width: `${progress}%` }}
                      transition={{ duration: 0.3 }}
                    />
                    <p className="text-[10px] text-gray-400 mt-0.5">{Math.round(progress)}%</p>
                  </div>
                )}
              </div>
              {model.is_downloaded ? (
                <span className="text-xs text-[#00ff88] flex-shrink-0">✓</span>
              ) : (
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    onDownload(model.type);
                  }}
                  disabled={isDownloading}
                  className="text-xs px-2.5 py-1 rounded-md bg-[#00ff88]/10 text-[#00ff88]
                             hover:bg-[#00ff88]/20 disabled:opacity-40 disabled:cursor-not-allowed
                             transition-colors flex-shrink-0"
                >
                  {isDownloading ? `${Math.round(progress)}%` : 'Baixar'}
                </button>
              )}
            </motion.div>
          );
        })}
      </div>
    </SettingsCard>
  );
}

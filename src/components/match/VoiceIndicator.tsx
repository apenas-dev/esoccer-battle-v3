import type { VoiceStatus } from '../../types';

interface VoiceIndicatorProps {
  status: VoiceStatus;
  isListening: boolean;
  lastTranscript: string | null;
  onStart: () => void;
  onStop: () => void;
}

export function VoiceIndicator({ status, isListening, lastTranscript, onStart, onStop }: VoiceIndicatorProps) {
  return (
    <div className="mt-6 text-center">
      <button
        onMouseDown={onStart}
        onMouseUp={onStop}
        onTouchStart={onStart}
        onTouchEnd={onStop}
        className={`w-20 h-20 rounded-full text-2xl transition-all duration-150 select-none ${
          isListening
            ? 'bg-red-600 hover:bg-red-700 animate-pulse shadow-lg shadow-red-600/50'
            : 'bg-gray-700 hover:bg-gray-600'
        }`}
      >
        {isListening ? '🎤' : '🎙️'}
      </button>
      
      <p className="mt-2 text-sm text-gray-400">
        {isListening ? 'Ouvindo...' : 'Segure para falar'}
      </p>

      {status === 'processing' && (
        <p className="mt-1 text-sm text-yellow-400">⏳ Processando...</p>
      )}

      {status === 'error' && (
        <p className="mt-1 text-sm text-red-400">❌ Erro na transcrição</p>
      )}

      {lastTranscript && !isListening && (
        <p className="mt-2 text-xs text-gray-500 italic">
          Último: "{lastTranscript}"
        </p>
      )}
    </div>
  );
}

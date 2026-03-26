import { useState, useRef, useCallback, useMemo } from 'react';
import { loadModel, startRecording, stopRecording } from './lib/transcriber';

const styles: Record<string, React.CSSProperties> = {
  container: {
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: '100vh',
    padding: 24,
    fontFamily: '-apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    background: '#0f172a',
    color: '#e2e8f0',
  },
  title: {
    fontSize: 28,
    fontWeight: 700,
    marginBottom: 4,
  },
  subtitle: {
    fontSize: 14,
    color: '#94a3b8',
    marginBottom: 24,
  },
  progress: {
    padding: '8px 16px',
    background: '#1e293b',
    borderRadius: 8,
    marginBottom: 16,
    fontSize: 14,
    color: '#60a5fa',
    maxWidth: 400,
    textAlign: 'center',
  },
  placeholder: {
    color: '#64748b',
    fontStyle: 'italic',
  },
  button: {
    padding: '12px 32px',
    border: 'none',
    borderRadius: 8,
    color: '#fff',
    fontSize: 16,
    fontWeight: 600,
    transition: 'background-color 0.2s',
  },
};

function App() {
  const [status, setStatus] = useState<'idle' | 'loading' | 'ready' | 'recording'>('idle');
  const [transcription, setTranscription] = useState('');
  const [progress, setProgress] = useState('');
  const transcriptionRef = useRef('');
  const isRecording = status === 'recording';

  const transcriptionBoxStyle = useMemo(
    () => ({
      ...styles.transcriptionBox,
      border: isRecording ? '2px solid #ef4444' : '2px solid #334155',
    }) as React.CSSProperties,
    [isRecording]
  );

  const handleStart = useCallback(async () => {
    if (status === 'idle' || status === 'loading') {
      setStatus('loading');
      try {
        await loadModel((p) => setProgress(p));
        setStatus('ready');
      } catch (err) {
        setStatus('idle');
        setProgress(`Erro: ${err}`);
        return;
      }
    }

    setStatus('recording');
    await startRecording((text) => {
      transcriptionRef.current += text + ' ';
      setTranscription(transcriptionRef.current);
    });
  }, [status]);

  const handleStop = useCallback(() => {
    stopRecording();
    setStatus('ready');
  }, []);

  return (
    <div style={styles.container}>
      <h1 style={styles.title}>🎙️ Transcrição por Voz</h1>
      <p style={styles.subtitle}>Offline • Whisper WASM • Português</p>

      {status === 'loading' && progress && (
        <div style={styles.progress}>
          <span>{progress}</span>
        </div>
      )}

      <div style={transcriptionBoxStyle}>
        {transcription || (
          <span style={styles.placeholder}>
            {isRecording ? 'Escutando...' : 'Clique em Iniciar para começar'}
          </span>
        )}
      </div>

      <button
        onClick={isRecording ? handleStop : handleStart}
        style={{
          ...styles.button,
          backgroundColor: isRecording ? '#dc2626' : '#2563eb',
          cursor: status === 'loading' ? 'wait' : 'pointer',
        }}
        disabled={status === 'loading'}
      >
        {status === 'loading'
          ? '⏳ Carregando modelo...'
          : isRecording
            ? '⏹ Parar'
            : '▶ Iniciar Transcrição'}
      </button>

      {transcription && (
        <button
          onClick={() => {
            setTranscription('');
            transcriptionRef.current = '';
          }}
          style={{ ...styles.button, backgroundColor: '#6b7280', marginTop: 8 }}
        >
          🗑 Limpar
        </button>
      )}
    </div>
  );
}

export default App;

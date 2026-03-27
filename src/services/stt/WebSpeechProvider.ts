import type { ISTTProvider } from './ISTTProvider';

export class WebSpeechProvider implements ISTTProvider {
  readonly name = 'web-speech';

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private recognition: any = null;
  private transcriptResolve: ((value: string) => void) | null = null;
  private lang: string;

  constructor(lang: string = 'pt-BR') {
    this.lang = lang;
  }

  async isAvailable(): Promise<boolean> {
    return 'SpeechRecognition' in window || 'webkitSpeechRecognition' in window;
  }

  async start(): Promise<void> {
    if (!this.recognition) {
      this.createRecognition();
    }
    this.onStatusChange?.('listening');
    this.recognition?.start();
  }

  async stop(): Promise<string> {
    this.onStatusChange?.('processing');
    this.recognition?.stop();
    return new Promise<string>((resolve) => {
      this.transcriptResolve = resolve;
    });
  }

  cancel(): void {
    this.cleanup();
  }

  onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;

  private createRecognition(): void {
    const win = window as unknown as Record<string, new () => any>;
    const SpeechRecognitionCtor = win.SpeechRecognition || win.webkitSpeechRecognition;

    if (!SpeechRecognitionCtor) {
      console.warn('Web Speech API not available');
      return;
    }

    this.recognition = new SpeechRecognitionCtor();
    this.recognition.continuous = false;
    this.recognition.interimResults = false;
    this.recognition.lang = this.lang;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    this.recognition.onresult = (event: any) => {
      const transcript = event.results[0]?.[0]?.transcript?.trim() ?? '';
      this.transcriptResolve?.(transcript);
      this.transcriptResolve = null;
    };

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    this.recognition.onerror = (_event: any) => {
      this.transcriptResolve?.('');
      this.transcriptResolve = null;
    };

    this.recognition.onend = () => {
      this.onStatusChange?.('idle');
    };
  }

  private cleanup(): void {
    if (this.recognition) {
      try {
        this.recognition.abort();
      } catch {
        // ignore
      }
      this.recognition = null;
    }
    this.transcriptResolve?.('');
    this.transcriptResolve = null;
  }
}

import type { ISTTProvider } from './ISTTProvider';

export class WebSpeechProvider implements ISTTProvider {
  readonly name = 'web-speech';
  private recognition: SpeechRecognition | null = null;
  private lang: string;
  private _onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;

  constructor(lang: string = 'pt-BR') {
    this.lang = lang;
  }

  set onStatusChange(cb: (status: 'idle' | 'listening' | 'processing') => void) {
    this._onStatusChange = cb;
  }

  async isAvailable(): Promise<boolean> {
    return !!(window.SpeechRecognition || window.webkitSpeechRecognition);
  }

  async start(): Promise<void> {
    const SR = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SR) throw new Error('Web Speech API not available');
    
    this.recognition = new SR();
    this.recognition.lang = this.lang;
    this.recognition.continuous = false;
    this.recognition.interimResults = false;
    
    this.recognition.onstart = () => this._onStatusChange?.('listening');
    this.recognition.onresult = () => {
      this._onStatusChange?.('processing');
    };
    this.recognition.onerror = () => {
      this._onStatusChange?.('idle');
    };
    
    this.recognition.start();
    this._onStatusChange?.('listening');
  }

  async stop(): Promise<string> {
    return new Promise((resolve, reject) => {
      if (!this.recognition) return resolve('');
      this.recognition.onresult = (e: SpeechRecognitionEvent) => {
        const transcript = e.results[0]?.[0]?.transcript || '';
        resolve(transcript);
      };
      this.recognition.onerror = (e: SpeechRecognitionErrorEvent) => reject(e.error);
      this.recognition.stop();
      this._onStatusChange?.('idle');
    });
  }

  cancel(): void {
    this.recognition?.stop();
    this._onStatusChange?.('idle');
  }
}

import type { ISTTProvider, STTConfig } from './types';

type SpeechRecognitionLike = {
  new (): SpeechRecognitionInstance;
  prototype: SpeechRecognitionInstance;
};

interface SpeechRecognitionInstance extends EventTarget {
  lang: string;
  continuous: boolean;
  interimResults: boolean;
  maxAlternatives: number;
  start(): void;
  stop(): void;
  abort(): void;
  onresult: ((event: SpeechRecognitionEventLike) => void) | null;
  onerror: ((event: Event & { error: string }) => void) | null;
  onend: (() => void) | null;
}

interface SpeechRecognitionEventLike extends Event {
  readonly results: SpeechRecognitionResultListLike;
}

interface SpeechRecognitionResultListLike {
  readonly length: number;
  [index: number]: SpeechRecognitionResultLike;
}

interface SpeechRecognitionResultLike {
  readonly length: number;
  readonly isFinal: boolean;
  [index: number]: SpeechRecognitionAlternativeLike;
}

interface SpeechRecognitionAlternativeLike {
  readonly transcript: string;
  readonly confidence: number;
}

function getSpeechRecognition(): SpeechRecognitionLike | null {
  if (typeof window === 'undefined') return null;
  return (window as unknown as Record<string, unknown>).SpeechRecognition as SpeechRecognitionLike
    ?? (window as unknown as Record<string, unknown>).webkitSpeechRecognition as SpeechRecognitionLike
    ?? null;
}

export class WebSpeechProvider implements ISTTProvider {
  readonly name = 'web-speech';

  private recognition: SpeechRecognitionInstance | null = null;
  private resultCallbacks: Array<(text: string, isFinal: boolean) => void> = [];
  private errorCallbacks: Array<(error: Error) => void> = [];
  private restartOnEnd = false;

  isAvailable(): boolean {
    return getSpeechRecognition() !== null;
  }

  start(config: STTConfig): void {
    const Ctor = getSpeechRecognition();
    if (!Ctor) return;

    this.recognition = new Ctor();
    this.recognition.lang = config.language ?? 'pt-BR';
    this.recognition.continuous = config.continuous ?? false;
    this.recognition.interimResults = false;
    this.recognition.maxAlternatives = 1;

    this.recognition.onresult = (event: SpeechRecognitionEventLike) => {
      for (let i = 0; i < event.results.length; i++) {
        const result = event.results[i];
        const text = result[0].transcript.trim();
        if (text) this.emitResult(text, result.isFinal);
      }
    };

    this.recognition.onerror = (event) => {
      if (event.error !== 'aborted' && event.error !== 'no-speech') {
        this.emitError(new Error(`SpeechRecognition: ${event.error}`));
      }
    };

    this.recognition.onend = () => {
      if (this.restartOnEnd) {
        try { this.recognition?.start(); } catch { /* ignore */ }
      }
    };

    this.restartOnEnd = true;
    this.recognition.start();
  }

  stop(): void {
    this.restartOnEnd = false;
    this.recognition?.stop();
    this.recognition = null;
  }

  onResult(cb: (text: string, isFinal: boolean) => void): () => void {
    this.resultCallbacks.push(cb);
    return () => {
      this.resultCallbacks = this.resultCallbacks.filter((c) => c !== cb);
    };
  }

  onError(cb: (error: Error) => void): () => void {
    this.errorCallbacks.push(cb);
    return () => {
      this.errorCallbacks = this.errorCallbacks.filter((c) => c !== cb);
    };
  }

  private emitResult(text: string, isFinal: boolean): void {
    this.resultCallbacks.forEach((cb) => cb(text, isFinal));
  }

  private emitError(error: Error): void {
    this.errorCallbacks.forEach((cb) => cb(error));
  }
}

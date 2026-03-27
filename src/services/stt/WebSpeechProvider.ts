// =============================================================================
// WebSpeechProvider — STT via browser Web Speech API
// Responsibility: Capture audio and transcribe using native SpeechRecognition
// Dependencies: ISTTProvider
// =============================================================================

import type { ISTTProvider } from './ISTTProvider';

// -- Web Speech API type shims (not all browsers expose full TS types) ---------

type SpeechRecognitionCtor = {
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
  onaudiostart?: (() => void) | null;
  onaudioend?: (() => void) | null;
  onsoundstart?: (() => void) | null;
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

// -- Helpers ------------------------------------------------------------------

function getSpeechRecognitionCtor(): SpeechRecognitionCtor | null {
  if (typeof window === 'undefined') return null;
  const w = window as unknown as Record<string, unknown>;
  return (w.SpeechRecognition ?? w.webkitSpeechRecognition) as SpeechRecognitionCtor | null ?? null;
}

// -- Provider -----------------------------------------------------------------

export class WebSpeechProvider implements ISTTProvider {
  readonly name = 'web-speech';

  private recognition: SpeechRecognitionInstance | null = null;
  private readonly lang: string;
  private resolveStop: ((text: string) => void) | null = null;
  private rejectStop: ((err: Error) => void) | null = null;
  private finalTranscript = '';
  private cancelled = false;

  onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;

  constructor(lang = 'pt-BR') {
    this.lang = lang;
  }

  // -- ISTTProvider ----------------------------------------------------------

  async start(): Promise<void> {
    const Ctor = getSpeechRecognitionCtor();
    if (!Ctor) throw new Error('Web Speech API not available in this browser');

    this.cancelled = false;
    this.finalTranscript = '';

    this.recognition = new Ctor();
    this.recognition.lang = this.lang;
    this.recognition.continuous = true;
    this.recognition.interimResults = true;
    this.recognition.maxAlternatives = 1;

    this.recognition.onresult = (event: SpeechRecognitionEventLike) => {
      let interim = '';
      let final = '';
      for (let i = 0; i < event.results.length; i++) {
        const result = event.results[i];
        if (result.isFinal) {
          final += result[0].transcript.trim() + ' ';
        } else {
          interim += result[0].transcript;
        }
      }
      this.finalTranscript = (final + interim).trim();
    };

    this.recognition.onerror = (event) => {
      if (event.error === 'aborted') return;
      this.onStatusChange?.('idle');
      this.rejectStop?.(new Error(`SpeechRecognition error: ${event.error}`));
    };

    this.recognition.onend = () => {
      if (this.cancelled) return;
      // Auto-resume while actively listening
      if (!this.resolveStop) {
        try { this.recognition?.start(); } catch { /* already stopped */ }
      }
    };

    this.onStatusChange?.('listening');
    this.recognition.start();
  }

  async stop(): Promise<string> {
    this.cancelled = false;

    return new Promise<string>((resolve, reject) => {
      this.resolveStop = resolve;
      this.rejectStop = reject;

      this.onStatusChange?.('processing');

      // Force stop — onend will fire but resolveStop is set so it won't auto-resume
      this.recognition?.stop();

      // Safety timeout: resolve with whatever we have after 3s
      setTimeout(() => {
        if (this.resolveStop) {
          const text = this.finalTranscript;
          this.cleanup();
          resolve(text);
        }
      }, 3000);
    });
  }

  cancel(): void {
    this.cancelled = true;
    this.cleanup();
  }

  async isAvailable(): Promise<boolean> {
    return getSpeechRecognitionCtor() !== null;
  }

  // -- Private ----------------------------------------------------------------

  private cleanup(): void {
    try { this.recognition?.abort(); } catch { /* ignore */ }
    this.recognition = null;
    this.onStatusChange?.('idle');
    if (this.resolveStop) {
      this.resolveStop(this.finalTranscript);
      this.resolveStop = null;
      this.rejectStop = null;
    }
  }
}

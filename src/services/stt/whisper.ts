import { isTauri, startRecording, stopRecordingAndTranscribe, onVoiceText, onCommandUnknown } from '../../lib/tauri';
import type { ISTTProvider, STTConfig } from './types';

export class WhisperProvider implements ISTTProvider {
  readonly name = 'whisper';

  private resultCallbacks: Array<(text: string, isFinal: boolean) => void> = [];
  private errorCallbacks: Array<(error: Error) => void> = [];
  private unlistens: Array<(() => void) | Promise<() => void>> = [];
  private started = false;

  isAvailable(): boolean {
    return isTauri();
  }

  start(_config: STTConfig): void {
    if (this.started) return;
    this.started = true;

    this.unlistens.push(
      onVoiceText(({ text }) => this.emitResult(text, true)),
      onCommandUnknown(({ text }) => this.emitResult(text, true)),
    );

    startRecording().catch((e) => this.emitError(e instanceof Error ? e : new Error(String(e))));
  }

  stop(): void {
    if (!this.started) return;
    this.started = false;

    stopRecordingAndTranscribe().catch(() => {});
    this.unlistens.forEach((u) => {
      if (typeof u === 'function') u();
      else u.then((fn) => fn());
    });
    this.unlistens = [];
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

import type { ISTTProvider } from './ISTTProvider';
import { invoke } from '@tauri-apps/api/core';

export class WhisperProvider implements ISTTProvider {
  readonly name = 'whisper';

  /** BUG 4 FIX: Store model and language for future backend use. */
  readonly model: string;
  readonly language: string;

  /** Track whether we have an active listening session so cancel can stop it. */
  private _isListening = false;

  constructor(model: string = 'base', language: string = 'pt-BR') {
    this.model = model;
    this.language = language;
  }

  async isAvailable(): Promise<boolean> {
    return true;
  }

  async start(): Promise<void> {
    this._isListening = true;
    this.onStatusChange?.('listening');
    await invoke('start_listening');
  }

  async stop(): Promise<string> {
    this._isListening = false;
    this.onStatusChange?.('processing');
    const transcript = await invoke<string>('stop_listening');
    return transcript ?? '';
  }

  /**
   * Cancel an in-flight listening session.
   * Uses stop_listening as the sole backend call — cancel_listening will be
   * added by the backend team later and can be wired up here then.
   */
  cancel(): void {
    if (!this._isListening) {
      this.onStatusChange?.('idle');
      return;
    }
    this._isListening = false;
    this.onStatusChange?.('idle');
    // Fire-and-forget: tell the backend to stop capturing audio.
    invoke('stop_listening').catch(() => {});
  }

  onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;
}

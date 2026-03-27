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

  /** BUG 5 FIX: Actually stop/cancel the backend listening session. */
  async cancel(): Promise<void> {
    if (!this._isListening) {
      this.onStatusChange?.('idle');
      return;
    }
    this._isListening = false;
    this.onStatusChange?.('idle');
    try {
      // Prefer explicit cancel command if available; fall back to stop_listening.
      await invoke('cancel_listening').catch(() => invoke('stop_listening'));
    } catch {
      // Silently ignore — status is already reset.
    }
  }

  onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;
}

import { invoke } from '@tauri-apps/api/core';
import type { ISTTProvider } from './ISTTProvider';

export class WhisperProvider implements ISTTProvider {
  readonly name = 'whisper';
  private _onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;

  constructor() {}

  set onStatusChange(cb: (status: 'idle' | 'listening' | 'processing') => void) {
    this._onStatusChange = cb;
  }

  async isAvailable(): Promise<boolean> {
    return true; // Always available via Tauri backend
  }

  async start(): Promise<void> {
    this._onStatusChange?.('listening');
    await invoke('start_listening');
  }

  async stop(): Promise<string> {
    this._onStatusChange?.('processing');
    const transcript = await invoke<string>('stop_listening');
    this._onStatusChange?.('idle');
    return transcript;
  }

  cancel(): void {
    this._onStatusChange?.('idle');
  }
}

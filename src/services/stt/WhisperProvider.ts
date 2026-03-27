import type { ISTTProvider } from './ISTTProvider';
import { invoke } from '@tauri-apps/api/core';

export class WhisperProvider implements ISTTProvider {
  readonly name = 'whisper';

  constructor(_model: string = 'base', _language: string = 'pt-BR') {
    void _model;
    void _language;
  }

  async isAvailable(): Promise<boolean> {
    return true;
  }

  async start(): Promise<void> {
    this.onStatusChange?.('listening');
    await invoke('start_listening');
  }

  async stop(): Promise<string> {
    this.onStatusChange?.('processing');
    const transcript = await invoke<string>('stop_listening');
    return transcript ?? '';
  }

  cancel(): void {
    this.onStatusChange?.('idle');
  }

  onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;
}

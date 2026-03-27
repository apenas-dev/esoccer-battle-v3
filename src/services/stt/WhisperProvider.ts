// =============================================================================
// WhisperProvider — STT via Tauri backend (whisper-rs)
// Responsibility: Capture audio through Tauri invoke and return transcript
// Dependencies: ISTTProvider, @tauri-apps/api/core
// =============================================================================

import type { ISTTProvider } from './ISTTProvider';
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

interface VoiceTextPayload {
  text: string;
}

export class WhisperProvider implements ISTTProvider {
  readonly name = 'whisper';

  /** @internal model name, reserved for future invoke params */
  readonly model: string;

  /** @internal language code, reserved for future invoke params */
  readonly language: string;

  private transcript = '';
  private unlisten: UnlistenFn | null = null;
  private recording = false;

  onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;

  constructor(model = 'base', language = 'pt-BR') {
    this.model = model;
    this.language = language;
  }

  // -- ISTTProvider ----------------------------------------------------------

  async start(): Promise<void> {
    this.transcript = '';

    // Listen for real-time voice text events while recording
    this.unlisten = await listen<VoiceTextPayload>('voice_text', (event) => {
      this.transcript = event.payload.text;
    });

    await invoke('start_recording');
    this.recording = true;
    this.onStatusChange?.('listening');
  }

  async stop(): Promise<string> {
    if (!this.recording) return '';
    this.recording = false;
    this.onStatusChange?.('processing');

    try {
      await invoke('stop_recording_and_transcribe');
    } catch {
      // Recording may already be stopped; use whatever transcript we collected
    }

    await this.cleanup();
    return this.transcript;
  }

  cancel(): void {
    if (!this.recording) return;
    this.recording = false;

    // Fire-and-forget — don't block cancel()
    invoke('stop_recording_and_transcribe').catch(() => {});

    this.cleanup();
  }

  async isAvailable(): Promise<boolean> {
    return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
  }

  // -- Private ----------------------------------------------------------------

  private async cleanup(): Promise<void> {
    await this.unlisten?.();
    this.unlisten = null;
    this.onStatusChange?.('idle');
  }
}

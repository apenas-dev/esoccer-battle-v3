// =============================================================================
// ISTTProvider — Contract for any STT (Speech-to-Text) provider
// Responsibility: Define the interface; zero implementation
// Dependencies: None
// =============================================================================

export interface ISTTProvider {
  /** Provider identifier (for debug/display) */
  readonly name: string;

  /** Start capture + transcription */
  start(): Promise<void>;

  /** Stop capture and return the transcript */
  stop(): Promise<string>;

  /** Cancel without returning a transcript */
  cancel(): void;

  /** Check if the provider is available on this platform */
  isAvailable(): Promise<boolean>;

  /** Optional status change callback */
  onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;
}

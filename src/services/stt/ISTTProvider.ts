export interface ISTTProvider {
  readonly name: string;
  start(): Promise<void>;
  stop(): Promise<string>;
  cancel(): void;
  isAvailable(): Promise<boolean>;
  onStatusChange?: (status: 'idle' | 'listening' | 'processing') => void;
}

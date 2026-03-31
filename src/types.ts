// --- Enums (mirror Rust) ---
export type GamePhase = 'idle' | 'playing' | 'paused' | 'finished';
export type TimerMode = 'countdown' | 'count_up';
export type WhisperModel = 'tiny' | 'base' | 'small';
export type Language = 'pt_br' | 'en' | 'es';
export type Theme = 'dark' | 'light';

// --- State ---
export interface MatchConfig {
  team_a_name: string;
  team_b_name: string;
  duration_secs: number;
  timer_mode: TimerMode;
}

export interface MatchState {
  phase: GamePhase;
  config: MatchConfig;
  score_a: number;
  score_b: number;
  elapsed_secs: number;
  started_at: number | null;
  paused_elapsed_secs: number;
  match_id: string;
}

export interface AppConfig {
  mic_device: string | null;
  whisper_model: WhisperModel;
  language: Language;
  voice_threshold: number;
  team_a_name: string;
  team_b_name: string;
  theme: Theme;
  match_duration_secs: number;
  timer_mode: TimerMode;
  volume: number;
}

// --- History ---
export interface HistoryEntry {
  id: string;
  match_id: string;
  team_a_name: string;
  team_b_name: string;
  score_a: number;
  score_b: number;
  duration_secs: number;
  finished_at: string;
}

// --- Commands ---
export interface CommandHelp {
  command: string;
  description: string;
  aliases: string[];
}

// --- Voice ---
export type VoiceStatus = 'idle' | 'listening' | 'processing' | 'error';

export interface VoiceEvent {
  status: VoiceStatus;
  transcript?: string;
  error?: string;
}

// --- Event Payloads ---
export interface ScoreChangedPayload { score_a: number; score_b: number; }
export interface MatchFinishedPayload { score_a: number; score_b: number; }

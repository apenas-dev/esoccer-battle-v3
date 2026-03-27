// --- Enums (espelho Rust) ---

export type GamePhase = 'idle' | 'playing' | 'paused' | 'finished';
export type PlayingSubPhase = 'normal' | 'challenge';
export type TimerMode = 'countdown' | 'countup';
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
  sub_phase: PlayingSubPhase;
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

// --- Command Help ---

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

// --- Tauri Event Payloads ---

export interface ScoreChangedPayload {
  score_a: number;
  score_b: number;
}

export interface TimeUpdatedPayload {
  elapsed_secs: number;
  display: string;
}

export interface MatchFinishedPayload {
  score_a: number;
  score_b: number;
}

// --- Command Log ---

export interface CommandLogEntry {
  id: string;
  timestamp: Date;
  command: string;
  source: 'voice' | 'button';
  success: boolean;
}

// --- Page ---

export type Page = 'match' | 'settings' | 'history' | 'help';

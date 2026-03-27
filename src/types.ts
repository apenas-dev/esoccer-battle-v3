// =============================================================================
// E-Soccer Battle V3 — Shared Types (mirror of Rust backend)
// Responsibility: Define ALL TypeScript types matching backend models
// Dependencies: None (pure domain types)
// =============================================================================

// --- Enums (mirror Rust) ---

/** Match lifecycle phase (state machine) */
export type GamePhase = 'idle' | 'playing' | 'paused' | 'finished';

/** Sub-state during Playing phase */
export type PlayingSubPhase = 'normal' | 'challenge';

/** Timer display mode */
export type TimerMode = 'countdown' | 'countup';

/** Whisper model size */
export type WhisperModel = 'tiny' | 'base' | 'small';

/** UI language */
export type Language = 'pt_br' | 'en' | 'es';

/** App theme */
export type Theme = 'dark' | 'light';

// --- Match State ---

/** Immutable match configuration (set before start) */
export interface MatchConfig {
  team_a_name: string;
  team_b_name: string;
  duration_secs: number;
  timer_mode: TimerMode;
}

/** Complete match state (immutable, updated via with_* builders in Rust) */
export interface MatchState {
  phase: GamePhase;
  sub_phase: PlayingSubPhase;
  config: MatchConfig;
  score_a: number;
  score_b: number;
  elapsed_secs: number;
  started_at: number | null; // timestamp millis
  paused_elapsed_secs: number; // accumulated when paused
  match_id: string; // UUID
}

// --- App Config ---

/** Persistent application settings */
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

/** Single entry in match history */
export interface HistoryEntry {
  id: string;
  match_id: string;
  team_a_name: string;
  team_b_name: string;
  score_a: number;
  score_b: number;
  duration_secs: number;
  finished_at: string; // ISO 8601
}

// --- Command Help ---

/** Voice command documentation */
export interface CommandHelp {
  command: string;
  description: string;
  aliases: string[];
}

// --- Voice Pipeline ---

/** Voice pipeline status */
export type VoiceStatus = 'idle' | 'listening' | 'processing' | 'error';

/** Voice event payload (from Tauri) */
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

export interface PhaseChangedPayload {
  phase: GamePhase;
  sub_phase: PlayingSubPhase;
}

// --- Command Log (frontend-only) ---

export interface CommandLogEntry {
  id: string;
  timestamp: Date;
  command: string;
  source: 'voice' | 'button';
  success: boolean;
}

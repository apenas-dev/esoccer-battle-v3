import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// ── Helpers ───────────────────────────────────────────

export function isTauri(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}

// ── Types ─────────────────────────────────────────────

export interface MatchState {
  status: 'idle' | 'playing' | 'challenge' | 'finished';
  score_a: number;
  score_b: number;
  elapsed_seconds: number;
  team_a_name: string;
  team_b_name: string;
  last_command: string | null;
}

export interface DeviceResult {
  name: string;
}

export interface AppSettings {
  mic_device: string | null;
  model: string;
  language: string;
  voice_threshold: number;
  theme: string;
  team_a_name: string;
  team_b_name: string;
}

export interface ModelInfo {
  name: string;
  size?: string;
}

export interface TimerTickPayload {
  elapsed_seconds: number;
}

export interface VoiceTextPayload {
  text: string;
}

// ── Commands ──────────────────────────────────────────

export async function startMatch(): Promise<void> {
  return invoke('start_match');
}

export async function endMatch(): Promise<void> {
  return invoke('end_match');
}

export async function goalA(): Promise<void> {
  return invoke('goal_a');
}

export async function goalB(): Promise<void> {
  return invoke('goal_b');
}

export async function restart(): Promise<void> {
  return invoke('restart');
}

export async function challenge(): Promise<void> {
  return invoke('challenge');
}

export async function resolveChallenge(): Promise<void> {
  return invoke('resolve_challenge');
}

export async function getMatchState(): Promise<MatchState> {
  return invoke('get_match_state');
}

export async function startListening(deviceName?: string, model?: string): Promise<void> {
  return invoke('start_listening', { deviceName, model });
}

export async function stopListening(): Promise<void> {
  return invoke('stop_listening');
}

export async function listMicrophone(): Promise<DeviceResult[]> {
  return invoke('list_microphone');
}

export async function getSettings(): Promise<AppSettings> {
  return invoke('get_settings');
}

export async function setSettings(settings: AppSettings): Promise<void> {
  return invoke('set_settings', { settings });
}

export async function downloadModel(model: string): Promise<string> {
  return invoke('download_model', { model });
}

export async function listModels(): Promise<ModelInfo[]> {
  return invoke('list_models');
}

// ── Events ────────────────────────────────────────────

export function onMatchStateChanged(cb: (state: MatchState) => void): Promise<UnlistenFn> {
  return listen<MatchState>('match_state_changed', (e) => cb(e.payload));
}

export function onTimerTick(cb: (payload: TimerTickPayload) => void): Promise<UnlistenFn> {
  return listen<TimerTickPayload>('timer_tick', (e) => cb(e.payload));
}

export function onVoiceText(cb: (payload: VoiceTextPayload) => void): Promise<UnlistenFn> {
  return listen<VoiceTextPayload>('voice_text', (e) => cb(e.payload));
}

export function onCommandUnknown(cb: (payload: VoiceTextPayload) => void): Promise<UnlistenFn> {
  return listen<VoiceTextPayload>('command_unknown', (e) => cb(e.payload));
}

export function onModelDownloadProgress(cb: (payload: { percent: number }) => void): Promise<UnlistenFn> {
  return listen<{ percent: number }>('model_download_progress', (e) => cb(e.payload));
}

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write as _;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use tauri::{Emitter, Listener, Manager, State};

mod audio;
mod buffer;
mod capture;
mod configuration;
mod game;
mod parser;
mod match_history;
mod on_demand_transcriber;
mod transcriber;

// ── Shared state types ───────────────────────────────────────────────────

type Settings = Arc<Mutex<configuration::AppSettings>>;
type MatchState = Arc<Mutex<game::MatchState>>;
type TimerHandle = Mutex<Option<TimerGuard>>;

/// Combined handle that keeps both the transcriber and the audio stream alive.
/// The `_stream` field is critical — dropping it would stop audio capture.
struct VoicePipelineHandle {
    pipeline: transcriber::VoicePipeline,
    _stream: capture::AudioStream,
}

type VoicePipelineState = Mutex<Option<VoicePipelineHandle>>;

/// State for push-to-talk recording (captures audio on demand).
type RecordingState = Arc<Mutex<Option<capture::AudioStream>>>;

/// Handle to the background timer thread.
struct TimerGuard {
    shutdown: Arc<AtomicBool>,
    thread_handle: thread::JoinHandle<()>,
}

impl TimerGuard {
    fn stop(self) {
        self.shutdown.store(true, Ordering::Relaxed);
        let _ = self.thread_handle.join();
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn project_directory() -> directories::ProjectDirs {
    directories::ProjectDirs::from("com.esoccer", "ESoccer", "ESoccerBattle")
        .expect("Cannot use app directory")
}

/// Spawn the 1-second timer thread. Returns when `match_state.status != Playing`.
fn spawn_timer(app: tauri::AppHandle, match_state: MatchState) -> TimerGuard {
    use std::time::Instant;

    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = shutdown.clone();

    let handle = thread::Builder::new()
        .name("match-timer".into())
        .spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                if shutdown_clone.load(Ordering::Relaxed) {
                    break;
                }

                // Measure how long the previous iteration took (lock + emit)
                // and compensate the sleep so ticks stay near 1 s.
                let processing_elapsed = last_tick.elapsed();
                let sleep_duration = Duration::from_secs(1).saturating_sub(processing_elapsed);
                if !sleep_duration.is_zero() {
                    thread::sleep(sleep_duration);
                }
                last_tick = Instant::now();

                let elapsed = {
                    let mut state = match match_state.lock() {
                        Ok(g) => g,
                        Err(e) => {
                            tracing::warn!("[timer] Lock poisoned: {e}");
                            let _ = app.emit("timer_error", format!("Lock poisoned: {e}"));
                            break;
                        }
                    };
                    game::tick(&mut state);
                    state.elapsed_seconds
                };

                if let Err(e) = app.emit("timer_tick", serde_json::json!({ "elapsed_seconds": elapsed })) {
                    tracing::warn!("[timer] Failed to emit: {e}");
                }

                // Stop ticking if not playing (check after tick since start_match sets Playing).
                let is_playing = match_state
                    .lock()
                    .map(|s| s.status == game::MatchStatus::Playing)
                    .unwrap_or(false);

                if !is_playing {
                    break;
                }
            }
        })
        .expect("Failed to spawn timer thread");

    TimerGuard {
        shutdown,
        thread_handle: handle,
    }
}

fn stop_timer(timer: &TimerHandle) {
    let mut guard = match timer.lock() {
        Ok(g) => g,
        Err(e) => {
            tracing::warn!("Timer lock poisoned: {e}");
            return;
        }
    };
    if let Some(t) = guard.take() {
        t.stop();
    }
}

fn emit_state(app: &tauri::AppHandle, state: &game::MatchState) {
    if let Err(e) = app.emit("match_state_changed", state.clone()) {
        tracing::warn!("Failed to emit match_state_changed: {e}");
    }
}

fn execute_command(
    app: &tauri::AppHandle,
    match_state: &MatchState,
    pipeline_state: &VoicePipelineState,
    timer_state: &TimerHandle,
    settings: &Settings,
    cmd: parser::GameCommand,
) {
    let state = {
        let mut guard = match match_state.lock() {
            Ok(g) => g,
            Err(e) => {
                tracing::warn!("MatchState lock poisoned: {e}");
                return;
            }
        };
        match cmd {
            parser::GameCommand::StartMatch => game::start_match(&mut guard),
            parser::GameCommand::Restart => game::restart(&mut guard),
            parser::GameCommand::EndMatch => {
                // Clone state BEFORE ending so we can save the record
                let pre_end = guard.clone();
                game::end_match(&mut guard);
                save_match_record(&pre_end);
            }
            parser::GameCommand::Challenge => game::challenge(&mut guard),
            parser::GameCommand::GoalA => game::goal_a(&mut guard),
            parser::GameCommand::GoalB => game::goal_b(&mut guard),
            parser::GameCommand::PauseMatch => { game::pause_match(&mut guard).ok(); }
            parser::GameCommand::ResumeMatch => { game::resume_match(&mut guard).ok(); }
            parser::GameCommand::ResolveChallenge => game::resolve_challenge(&mut guard),
        }
        guard.clone()
    };

    // Play sound
    match cmd {
        parser::GameCommand::GoalA | parser::GameCommand::GoalB => {
            audio::play_sound(audio::GameSound::Goal);
        }
        parser::GameCommand::StartMatch => {
            audio::play_sound(audio::GameSound::WhistleStart);
        }
        parser::GameCommand::EndMatch => {
            audio::play_sound(audio::GameSound::WhistleEnd);
        }
        parser::GameCommand::Restart => {
            audio::play_sound(audio::GameSound::SixMeters);
        }
        parser::GameCommand::Challenge => {
            audio::play_sound(audio::GameSound::Challenge);
        }
        parser::GameCommand::PauseMatch | parser::GameCommand::ResumeMatch | parser::GameCommand::ResolveChallenge => {
            // no specific sound
        }
    }

    // Handle timer and voice pipeline lifecycle per command
    match cmd {
        parser::GameCommand::EndMatch => {
            tracing::info!("[voice] EndMatch via voice — stopping pipeline and timer");
            stop_listening_inner(pipeline_state);
            stop_timer(timer_state);
        }
        parser::GameCommand::Restart => {
            tracing::info!("[voice] Restart via voice — stopping pipeline and timer, then re-spawning both");
            stop_listening_inner(pipeline_state);
            stop_timer(timer_state);

            // Re-spawn voice pipeline
            let (mic_device, model_str) = {
                let s = match settings.lock() {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::warn!("[voice] Restart: settings lock poisoned: {e}");
                        emit_state(app, &state);
                        return;
                    }
                };
                (s.mic_device.clone(), s.model.clone())
            };
            match transcriber::Model::from_str_friendly(&model_str) {
                Some(model) if model.is_downloaded() => {
                    if let Err(e) = start_listening_inner(app, pipeline_state, mic_device, model) {
                        tracing::warn!("[voice] Restart: failed to restart voice pipeline: {e}");
                    }
                }
                _ => {
                    tracing::info!("[voice] Restart: model not available, skipping voice pipeline restart");
                }
            }

            // Re-spawn timer
            let guard = spawn_timer(app.clone(), match_state.clone());
            if let Ok(mut t) = timer_state.lock() {
                *t = Some(guard);
            }
        }
        parser::GameCommand::ResumeMatch | parser::GameCommand::ResolveChallenge => {
            tracing::info!("[voice] {:?} via voice — re-spawning timer", cmd);
            stop_timer(timer_state);
            let guard = spawn_timer(app.clone(), match_state.clone());
            if let Ok(mut t) = timer_state.lock() {
                *t = Some(guard);
            }
        }
        _ => {}
    }

    emit_state(app, &state);
}

// ── Tauri commands ───────────────────────────────────────────────────────

#[tauri::command]
fn start_match(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
    settings: State<'_, Settings>,
) -> Result<(), String> {
    let (team_a, team_b) = {
        let s = settings
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        (s.team_a_name.clone(), s.team_b_name.clone())
    };

    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        *state = game::new_match(&team_a, &team_b);
        game::start_match(&mut state);
    }

    // Stop any existing timer
    stop_timer(&timer);

    // Spawn new timer
    let guard = spawn_timer(app.clone(), match_state.inner().clone());
    {
        let mut t = timer
            .lock()
            .map_err(|e| format!("Timer lock poisoned: {e}"))?;
        *t = Some(guard);
    }

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);

    // Play start whistle
    audio::play_sound(audio::GameSound::WhistleStart);

    Ok(())
}

#[tauri::command]
fn save_match_record(state: &game::MatchState) {
    let record = match_history::MatchRecord {
        id: uuid::Uuid::new_v4().to_string(),
        team_a_name: state.team_a_name.clone(),
        team_b_name: state.team_b_name.clone(),
        score_a: state.score_a,
        score_b: state.score_b,
        duration_secs: state.elapsed_seconds as u32,
        finished_at: chrono::Utc::now().to_rfc3339(),
    };
    if let Err(e) = match_history::save_match(&record) {
        tracing::warn!("[match_history] Failed to save: {e}");
    }
}

#[tauri::command]
fn end_match(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
    pipeline: State<'_, VoicePipelineState>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::end_match(&mut state);
    }

    stop_timer(&timer);
    audio::play_sound(audio::GameSound::WhistleEnd);

    // Auto-stop voice pipeline (non-fatal)
    stop_listening_inner(pipeline.inner());

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();

    // Save match record to history
    save_match_record(&state);

    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn goal_a(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::goal_a(&mut state);
    }
    audio::play_sound(audio::GameSound::Goal);

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn goal_b(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::goal_b(&mut state);
    }
    audio::play_sound(audio::GameSound::Goal);

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn restart(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
    pipeline: State<'_, VoicePipelineState>,
    settings: State<'_, Settings>,
) -> Result<(), String> {
    stop_timer(&timer);

    // Stop and restart voice pipeline to prevent mic freeze
    stop_listening_inner(pipeline.inner());
    let (mic_device, model_str) = {
        let s = settings
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        (s.mic_device.clone(), s.model.clone())
    };
    match transcriber::Model::from_str_friendly(&model_str) {
        Some(model) if model.is_downloaded() => {
            if let Err(e) = start_listening_inner(&app, pipeline.inner(), mic_device, model) {
                tracing::warn!("[restart] Failed to restart voice pipeline: {e}");
            }
        }
        _ => {}
    }

    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::restart(&mut state);
    }
    audio::play_sound(audio::GameSound::SixMeters);

    // Re-spawn timer since we're still Playing
    let guard = spawn_timer(app.clone(), match_state.inner().clone());
    {
        let mut t = timer
            .lock()
            .map_err(|e| format!("Timer poisoned: {e}"))?;
        *t = Some(guard);
    }

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn challenge(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::challenge(&mut state);
    }
    audio::play_sound(audio::GameSound::Challenge);

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn resolve_challenge(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::resolve_challenge(&mut state);
    }

    // Stop any existing timer before re-spawning
    stop_timer(&timer);

    // Re-spawn timer since we're back to Playing
    let guard = spawn_timer(app.clone(), match_state.inner().clone());
    {
        let mut t = timer
            .lock()
            .map_err(|e| format!("Timer lock poisoned: {e}"))?;
        *t = Some(guard);
    }

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn set_score_a(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    score: u32,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::set_score_a(&mut state, score);
    }
    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn set_score_b(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    score: u32,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::set_score_b(&mut state, score);
    }
    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn undo_goal(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::undo_goal(&mut state)?;
    }
    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn pause_match(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::pause_match(&mut state)?;
    }

    stop_timer(&timer);

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}

#[tauri::command]
fn resume_match(
    app: tauri::AppHandle,
    match_state: State<'_, MatchState>,
    timer: State<'_, TimerHandle>,
) -> Result<(), String> {
    {
        let mut state = match_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        game::resume_match(&mut state)?;
    }

    // Stop any existing timer then re-spawn
    stop_timer(&timer);
    let guard = spawn_timer(app.clone(), match_state.inner().clone());
    {
        let mut t = timer
            .lock()
            .map_err(|e| format!("Timer lock poisoned: {e}"))?;
        *t = Some(guard);
    }

    let state = match_state
        .lock()
        .map_err(|e| format!("Lock poisoned: {e}"))?
        .clone();
    emit_state(&app, &state);
    Ok(())
}


// ── Score adjustment commands ────────────────────────────────────────────

#[tauri::command]
fn get_match_state(match_state: State<'_, MatchState>) -> Result<game::MatchState, String> {
    match_state
        .lock()
        .map(|s| s.clone())
        .map_err(|e| format!("Lock poisoned: {e}"))
}

// ── Voice pipeline commands ──────────────────────────────────────────────

/// Shared logic used by both `start_listening` command and `start_match_cmd`.
fn start_listening_inner(
    app: &tauri::AppHandle,
    pipeline_state: &VoicePipelineState,
    device_name: Option<String>,
    model: transcriber::Model,
) -> Result<(), String> {
    // Stop any existing pipeline first.
    {
        let mut guard = pipeline_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        if guard.is_some() {
            let old = guard.take().ok_or("Failed to take old pipeline")?;
            old.pipeline.stop().map_err(|e| format!("Failed to stop previous pipeline: {e}"))?;
        }
    }

    tracing::info!(
        "[voice] Starting capture: device={}",
        device_name.as_deref().unwrap_or("default")
    );

    let stream = match capture::start_capture(device_name) {
        Ok(s) => {
            tracing::info!("[voice] ✅ Capture started");
            s
        }
        Err(e) => {
            tracing::error!("[voice] ❌ Voice pipeline failed: {e}");
            return Err(e);
        }
    };

    let audio_buffer = stream.buffer.clone();

    let voice_pipeline = match transcriber::VoicePipeline::start(app.clone(), audio_buffer, model) {
        Ok(p) => {
            tracing::info!("[voice] ✅ Transcriber started");
            p
        }
        Err(e) => {
            tracing::error!("[voice] ❌ Voice pipeline failed: {e}");
            return Err(e);
        }
    };

    {
        let mut guard = pipeline_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        *guard = Some(VoicePipelineHandle {
            pipeline: voice_pipeline,
            _stream: stream,
        });
    }

    tracing::info!("[voice] ✅ Voice pipeline fully operational");
    Ok(())
}

/// Internal stop helper that takes `&VoicePipelineState` (not State wrapper).
fn stop_listening_inner(pipeline_state: &VoicePipelineState) {
    let mut guard = match pipeline_state.lock() {
        Ok(g) => g,
        Err(e) => {
            tracing::warn!("[voice] Pipeline lock poisoned: {e}");
            return;
        }
    };
    if let Some(handle) = guard.take() {
        tracing::info!("[voice] Stopping voice pipeline");
        if let Err(e) = handle.pipeline.stop() {
            tracing::warn!("[voice] Failed to stop pipeline: {e}");
        }
        // handle._stream dropped here — stops capture cleanly
    }
}

#[tauri::command]
fn start_listening(
    app: tauri::AppHandle,
    pipeline: State<'_, VoicePipelineState>,
    device_name: Option<String>,
    model: transcriber::Model,
) -> Result<(), String> {
    start_listening_inner(&app, pipeline.inner(), device_name, model)
}

#[tauri::command]
fn stop_listening(pipeline: State<'_, VoicePipelineState>) -> Result<(), String> {
    stop_listening_inner(pipeline.inner());
    Ok(())
}

#[tauri::command]
fn list_microphone() -> Vec<capture::DeviceResult> {
    capture::list_microphone()
}

// ── Push-to-talk commands ─────────────────────────────────────────────────

#[tauri::command]
fn start_recording(
    app: tauri::AppHandle,
    recording_state: State<'_, RecordingState>,
    settings: State<'_, Settings>,
) -> Result<(), String> {
    let (mic_device, model_str) = {
        let s = settings
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        (s.mic_device.clone(), s.model.clone())
    };

    // Verify model is downloaded
    let model = transcriber::Model::from_str_friendly(&model_str)
        .ok_or_else(|| format!("Unknown model '{model_str}'"))?;
    if !model.is_downloaded() {
        return Err(format!("Model '{}' is not downloaded", model_str));
    }

    // Stop any previous recording
    {
        let mut guard = recording_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        if let Some(_old) = guard.take() {
            tracing::info!("[ptt] Stopped previous recording");
            // _old dropped here — capture stops cleanly
        }
    }

    tracing::info!(
        "[ptt] Starting capture: device={}",
        mic_device.as_deref().unwrap_or("default")
    );

    let stream = capture::start_capture(mic_device)
        .map_err(|e| format!("Failed to start capture: {e}"))?;

    tracing::info!("[ptt] ✅ Capture started");

    {
        let mut guard = recording_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        *guard = Some(stream);
    }

    let _ = app.emit(
        "recording_state",
        serde_json::json!({ "status": "recording" }),
    );

    tracing::info!("[ptt] ✅ Recording state emitted");
    Ok(())
}

#[tauri::command]
fn stop_recording_and_transcribe(
    app: tauri::AppHandle,
    recording_state: State<'_, RecordingState>,
    settings: State<'_, Settings>,
) -> Result<(), String> {
    let (model_str,) = {
        let s = settings
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        (s.model.clone(),)
    };

    let model = transcriber::Model::from_str_friendly(&model_str)
        .ok_or_else(|| format!("Unknown model '{model_str}'"))?;

    // Take the stream out (stops capture)
    let stream = {
        let mut guard = recording_state
            .lock()
            .map_err(|e| format!("Lock poisoned: {e}"))?;
        guard.take()
    };

    let stream = match stream {
        Some(s) => {
            tracing::info!("[ptt] Stopping capture, draining buffer…");
            s
        }
        None => {
            tracing::warn!("[ptt] stop_recording called but no active recording");
            let _ = app.emit(
                "recording_state",
                serde_json::json!({ "status": "idle" }),
            );
            return Ok(());
        }
    };

    let audio = stream.drain_buffer();

    // stream is dropped here — capture fully stopped

    if audio.len() < 16_000 {
        tracing::info!(
            "[ptt] Audio too short ({} samples < 16000) — ignoring",
            audio.len()
        );
        let _ = app.emit("voice_empty", serde_json::json!({}));
        let _ = app.emit(
            "recording_state",
            serde_json::json!({ "status": "idle" }),
        );
        return Ok(());
    }

    let _ = app.emit(
        "recording_state",
        serde_json::json!({ "status": "processing" }),
    );

    tracing::info!(
        "[ptt] Transcribing {} samples (lang={})",
        audio.len(),
        model.default_language()
    );

    let text = on_demand_transcriber::transcribe_once(
        &audio,
        &model,
        model.default_language(),
    )
    .map_err(|e| format!("Transcription failed: {e}"))?;

    let _ = app.emit(
        "voice_text",
        serde_json::json!({ "text": text }),
    );

    let _ = app.emit(
        "recording_state",
        serde_json::json!({ "status": "idle" }),
    );

    tracing::info!("[ptt] ✅ Transcription emitted, back to idle");
    Ok(())
}

#[tauri::command]
fn get_recording_state(recording_state: State<'_, RecordingState>) -> Result<serde_json::Value, String> {
    let is_recording = recording_state
        .lock()
        .map(|g| g.is_some())
        .unwrap_or(false);

    let status = if is_recording { "recording" } else { "idle" };
    Ok(serde_json::json!({ "status": status }))
}

// ── Settings commands ────────────────────────────────────────────────────

#[tauri::command]
fn get_settings(settings: State<'_, Settings>) -> Result<configuration::AppSettings, String> {
    settings
        .lock()
        .map(|s| s.clone())
        .map_err(|e| format!("Lock poisoned: {e}"))
}

#[tauri::command]
fn set_settings(
    settings_state: State<'_, Settings>,
    settings: configuration::AppSettings,
) -> Result<(), String> {
    if let Err(e) = configuration::save(&settings) {
        tracing::warn!("Failed to save settings: {e}");
    }
    settings_state
        .lock()
        .map(|mut s| *s = settings)
        .map_err(|e| format!("Lock poisoned: {e}"))
}

// ── Match history commands ───────────────────────────────────────────────

#[tauri::command]
fn get_match_history() -> Result<Vec<match_history::MatchRecord>, String> {
    match_history::load_history()
}

#[tauri::command]
fn clear_match_history() -> Result<(), String> {
    match_history::clear_history()
}

// ── Model management commands ────────────────────────────────────────────

#[tauri::command]
fn download_model(app: tauri::AppHandle, model: transcriber::Model) -> Result<String, String> {
    let channel_name = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("Time error: {e}"))?
        .as_nanos()
        .to_string();

    let ch = channel_name.clone();
    tauri::async_runtime::spawn(async move {
        let response = reqwest::get(model.download_url())
            .await
            .and_then(|r| r.error_for_status())
            .map_err(|e| format!("Download failed: {e}"));

        let mut response = match response {
            Ok(r) => r,
            Err(e) => {
                let _ = app.emit(&ch, serde_json::json!({"type": "error", "value": e}));
                return;
            }
        };

        let file = match std::fs::File::create(model.path()) {
            Ok(f) => f,
            Err(e) => {
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": format!("Create file: {e}")}),
                );
                return;
            }
        };

        let mut file = std::io::BufWriter::new(file);
        let mut downloaded_size = 0u64;
        let predicted_size =
            response.content_length().unwrap_or((model.disk_usage() * 1_000_000) as u64);

        while let Some(chunk) = match response.chunk().await {
            Ok(Some(c)) => Some(c),
            Ok(None) => None,
            Err(e) => {
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": format!("Stream error: {e}")}),
                );
                None
            }
        } {
            if file.write_all(&chunk).is_err() {
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": "Write failed"}),
                );
                return;
            }
            downloaded_size += chunk.len() as u64;

            let percent = (downloaded_size as f32 / predicted_size as f32) * 100.0;
            let _ = app.emit(
                &ch,
                serde_json::json!({
                    "type": "progress",
                    "value": percent.min(100.0),
                }),
            );
        }

        drop(file);

        // Verify file size (5% tolerance for rounding / gzip differences)
        let min_size = ((model.disk_usage() * 1_000_000) as f64 * 0.95) as u64;
        match std::fs::metadata(model.path()) {
            Ok(meta) if meta.len() >= min_size => {}
            Ok(meta) => {
                let _ = std::fs::remove_file(model.path());
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": format!(
                        "Downloaded file too small: {} bytes (expected >= {})", meta.len(), min_size
                    )}),
                );
                return;
            }
            Err(e) => {
                let _ = app.emit(
                    &ch,
                    serde_json::json!({"type": "error", "value": format!("Failed to verify file: {e}")}),
                );
                return;
            }
        }

        let _ = app.emit(&ch, serde_json::json!({"type": "done", "value": ""}));
    });

    Ok(channel_name)
}

#[tauri::command]
fn list_models() -> Vec<serde_json::Value> {
    use strum::IntoEnumIterator as _;
    transcriber::Model::iter()
        .map(|model| {
            serde_json::json!({
                "type": model,
                "name": model.name(),
                "mem_usage": model.average_memory_usage(),
                "disk_usage": model.disk_usage(),
                "is_downloaded": model.is_downloaded(),
                "can_run": model.can_run(),
                "category": model.category(),
                "type_name": model.model_type().name(),
            })
        })
        .collect()
}

#[tauri::command]
fn list_model_categories() -> Vec<serde_json::Value> {
    use strum::IntoEnumIterator as _;
    transcriber::Category::iter()
        .map(|c| serde_json::json!({ "type": c, "name": c.name() }))
        .collect()
}

// ── Voice → Game integration (event listener) ────────────────────────────

fn setup_voice_to_game(app: &tauri::AppHandle) {
    let app_handle = app.clone();
    tracing::info!("[voice-setup] Event listener registered for voice_text");
    app.listen("voice_text", move |event: tauri::Event| {
        let text = event.payload();

        // Parse JSON payload: {"text": "..."}
        let trimmed = match serde_json::from_str::<serde_json::Value>(text) {
            Ok(v) => v["text"].as_str().unwrap_or("").trim().to_string(),
            Err(_) => text.trim().to_string(),
        };
        if trimmed.is_empty() {
            return;
        }

        tracing::info!("[voice→game] Received: \"{trimmed}\"");

        // Parse command
        match parser::parse_command(&trimmed) {
            Some(cmd) => {
                tracing::info!("[voice→game] Parsed command: {cmd:?}");
                let match_state: MatchState = app_handle.state::<MatchState>().inner().clone();
                let pipeline_state = app_handle.state::<VoicePipelineState>().inner();
                let timer_state = app_handle.state::<TimerHandle>().inner();
                let settings_state = app_handle.state::<Settings>().inner();
                execute_command(&app_handle, &match_state, pipeline_state, timer_state, settings_state, cmd);
            }
            None => {
                tracing::info!("[voice→game] Unknown command: \"{trimmed}\"");
                if let Err(e) = app_handle.emit("command_unknown", serde_json::json!({ "text": trimmed })) {
                    tracing::warn!("Failed to emit command_unknown: {e}");
                }
            }
        }
    });
}

// ── Main ─────────────────────────────────────────────────────────────────

fn main() {
    tracing_subscriber::fmt::init();

    let settings: Settings =
        Arc::new(Mutex::new(configuration::AppSettings::load_or_default()));

    let match_state: MatchState = Arc::new(Mutex::new(game::new_match("Time A", "Time B")));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(settings)
        .manage(match_state)
        .manage(Mutex::new(None::<VoicePipelineHandle>))
        .manage(Mutex::new(None::<TimerGuard>))
        .manage(Arc::new(Mutex::new(None::<capture::AudioStream>)))
        .setup(|app| {
            setup_voice_to_game(&app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_match,
            end_match,
            goal_a,
            goal_b,
            set_score_a,
            set_score_b,
            undo_goal,
            restart,
            challenge,
            resolve_challenge,
            pause_match,
            resume_match,
            get_match_state,
            get_match_history,
            clear_match_history,
            start_listening,
            stop_listening,
            list_microphone,
            get_settings,
            set_settings,
            download_model,
            list_models,
            list_model_categories,
            start_recording,
            stop_recording_and_transcribe,
            get_recording_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

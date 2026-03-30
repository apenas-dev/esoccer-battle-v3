use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tauri::{AppHandle, Emitter};

use crate::game::{MatchState, TimerMode};

/// Manages a single background timer thread that emits `time-updated` events.
pub struct TimerManager {
    /// Sender to signal the timer thread to stop.
    stop_tx: Mutex<Option<Sender<()>>>,
    /// Handle to the running timer thread (so we can join on drop).
    thread_handle: Mutex<Option<JoinHandle<()>>>,
    /// Whether the timer is currently running.
    running: Arc<AtomicBool>,
}

impl TimerManager {
    pub fn new() -> Self {
        Self {
            stop_tx: Mutex::new(None),
            thread_handle: Mutex::new(None),
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Start the timer. Does nothing if already running.
    ///
    /// `shared_state` allows the timer to auto-finish the match (CountDown mode)
    /// by calling `match_service::process(End)` and updating state atomically.
    pub fn start(
        &self,
        app: AppHandle,
        initial_elapsed: u64,
        duration_secs: u64,
        timer_mode: TimerMode,
        shared_state: Arc<Mutex<MatchState>>,
    ) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        self.stop_tx.lock().unwrap().replace(stop_tx);
        self.running.store(true, Ordering::SeqCst);

        let running_flag = self.running.clone();

        let handle = thread::spawn(move || {
            let mut elapsed = initial_elapsed;
            let is_countdown = timer_mode == TimerMode::Countdown;

            loop {
                // Check for stop signal with 1-second timeout
                match stop_rx.recv_timeout(std::time::Duration::from_secs(1)) {
                    Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => {
                        // Stop signal received or channel closed
                        break;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Normal tick — increment and emit
                        elapsed += 1;

                        let display = if is_countdown {
                            let remaining = duration_secs.saturating_sub(elapsed);
                            format!("{:02}:{:02}", remaining / 60, remaining % 60)
                        } else {
                            format!("{:02}:{:02}", elapsed / 60, elapsed % 60)
                        };

                        // Update shared state elapsed_secs so get_state() always reflects real time
                        if let Ok(mut state_lock) = shared_state.lock() {
                            state_lock.elapsed_secs = elapsed;
                        }

                        let _ = app.emit(
                            "time-updated",
                            serde_json::json!({
                                "elapsed_secs": elapsed,
                                "display": display,
                                "remaining_secs": if is_countdown {
                                    Some(duration_secs.saturating_sub(elapsed))
                                } else {
                                    None
                                },
                            }),
                        );

                        // Check time up for countdown mode
                        if is_countdown && elapsed >= duration_secs {
                            tracing::info!(
                                "Timer reached duration ({}s), auto-finishing match",
                                duration_secs
                            );

                            // --- Auto-finish: atomically transition state to Finished ---
                            // We read current state, process End command, and write back
                            // all within a single lock to prevent races.
                            let dispatch_actions = {
                                let mut state_lock = match shared_state.lock() {
                                    Ok(lock) => lock,
                                    Err(_) => {
                                        tracing::error!("Failed to lock state for auto-finish");
                                        break;
                                    }
                                };
                                let current = (*state_lock).clone();
                                let result =
                                    crate::match_service::process(&current, crate::command::GameCommand::End);
                                *state_lock = result.new_state.clone();
                                result.actions
                            };

                            // Emit time-up event
                            let _ = app.emit(
                                "time-up",
                                serde_json::json!({
                                    "elapsed_secs": elapsed,
                                    "display": "00:00",
                                }),
                            );

                            // Dispatch the End actions (sounds, phase-changed, save match, etc.)
                            if let Err(e) =
                                crate::action_dispatcher::dispatch(dispatch_actions, &app)
                            {
                                tracing::error!("Failed to dispatch auto-finish actions: {}", e);
                            }

                            break;
                        }
                    }
                }
            }

            running_flag.store(false, Ordering::SeqCst);
        });

        *self.thread_handle.lock().unwrap() = Some(handle);
        tracing::info!(
            "Timer started (initial elapsed: {}s, mode: {:?}, duration: {}s)",
            initial_elapsed,
            timer_mode,
            duration_secs
        );
    }

    /// Stop the timer. Waits for the thread to finish.
    pub fn stop(&self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }

        // Send stop signal
        if let Ok(mut guard) = self.stop_tx.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(());
            }
        }

        // Wait for thread to finish (with timeout)
        if let Ok(mut handle_guard) = self.thread_handle.lock() {
            if let Some(handle) = handle_guard.take() {
                // Give the thread a moment to stop cleanly
                let _ = handle.join();
            }
        }

        tracing::info!("Timer stopped");
    }

    /// Check if timer is running
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }
}

impl Default for TimerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TimerManager {
    fn drop(&mut self) {
        // Ensure timer is stopped on drop
        if let Ok(mut guard) = self.stop_tx.lock() {
            if let Some(tx) = guard.take() {
                let _ = tx.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timer_manager_new_not_running() {
        let tm = TimerManager::new();
        assert!(!tm.is_running());
    }

    #[test]
    fn timer_manager_stop_when_not_running() {
        let tm = TimerManager::new();
        tm.stop(); // Should not panic
        assert!(!tm.is_running());
    }
}

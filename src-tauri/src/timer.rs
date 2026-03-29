use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use tauri::{AppHandle, Emitter};

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
    pub fn start(&self, app: AppHandle, initial_elapsed: u64, duration_secs: u64) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }

        let (stop_tx, stop_rx) = mpsc::channel::<()>();
        self.stop_tx.lock().unwrap().replace(stop_tx);
        self.running.store(true, Ordering::SeqCst);

        let running_flag = self.running.clone();
        // Clone the AtomicBool so the thread can clear it on exit

        let handle = thread::spawn(move || {
            let mut elapsed = initial_elapsed;
            let timer_mode_countdown = duration_secs > 0; // non-zero duration means countdown

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

                        let remaining = duration_secs.saturating_sub(elapsed);
                        let display = if timer_mode_countdown {
                            format!("{:02}:{:02}", remaining / 60, remaining % 60)
                        } else {
                            format!("{:02}:{:02}", elapsed / 60, elapsed % 60)
                        };

                        let _ = app.emit(
                            "time-updated",
                            serde_json::json!({
                                "elapsed_secs": elapsed,
                                "display": display,
                                "remaining_secs": if timer_mode_countdown { Some(remaining) } else { None },
                            }),
                        );

                        // Check time up for countdown mode
                        if timer_mode_countdown && elapsed >= duration_secs {
                            tracing::info!("Timer reached duration ({}s), auto-finishing match", duration_secs);
                            // Auto-finish: execute End command
                            let app_clone = app.clone();
                            tauri::async_runtime::block_on(async {
                                let _ = app_clone.emit("time-up", serde_json::json!({
                                    "elapsed_secs": elapsed,
                                    "display": "00:00",
                                }));
                            });
                            break;
                        }
                    }
                }
            }

            running_flag.store(false, Ordering::SeqCst);
        });

        *self.thread_handle.lock().unwrap() = Some(handle);
        tracing::info!("Timer started (initial elapsed: {}s)", initial_elapsed);
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

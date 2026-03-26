//! Chunk extraction from the shared audio ring buffer.
//!
//! Extracts audio chunks with overlap for continuous transcription.

use crate::capture::AudioBuffer;

/// Default chunk duration in seconds (~40 000 samples at 16 kHz).
pub const DEFAULT_CHUNK_SECS: f32 = 2.5;

/// Default overlap in seconds (~8 000 samples at 16 kHz).
pub const DEFAULT_OVERLAP_SECS: f32 = 0.5;

/// Target sample rate (must match capture.rs).
const SAMPLE_RATE: u32 = 16_000;

/// Extract the next audio chunk from the ring buffer.
///
/// * `buffer` — shared ring buffer (newest samples at the back).
/// * `chunk_duration_secs` — how many seconds of audio to include.
/// * `overlap_secs` — how many trailing seconds of the **previous** chunk
///   to keep for continuity.
///
/// Returns `None` when the buffer doesn't yet contain enough new data
/// (i.e. fewer than `chunk - overlap` seconds have accumulated since the
/// last extraction).
///
/// Samples that have been fully consumed are drained from the front of
/// the buffer so they are never re-transcribed.
pub fn extract_chunk(
    buffer: &AudioBuffer,
    chunk_duration_secs: f32,
    overlap_secs: f32,
) -> Option<Vec<f32>> {
    let chunk_samples = (chunk_duration_secs * SAMPLE_RATE as f32) as usize;
    let overlap_samples = (overlap_secs * SAMPLE_RATE as f32) as usize;

    // The minimum new data we need before we can produce a chunk.
    // We keep `overlap_samples` from the previous extraction, so we only
    // need `chunk_samples - overlap_samples` new ones.
    let min_new = chunk_samples.saturating_sub(overlap_samples);

    let mut buf = buffer.lock().ok()?;
    if buf.len() < min_new {
        return None;
    }

    // Drain samples up to `chunk_samples - overlap_samples` from the front.
    // These are "old" samples that no longer belong to any future chunk.
    let drain_count = if buf.len() >= chunk_samples {
        // Buffer is large enough: we can take a full chunk and drain
        // the portion that won't be needed by the next extraction.
        buf.len() - overlap_samples
    } else {
        // Buffer has between `min_new` and `chunk_samples`.
        // Don't drain more than we have; take everything available.
        0
    };

    let consumed: Vec<f32> = buf.drain(..drain_count).collect();

    // The chunk is the last `chunk_samples` of what remains (or all of it
    // if buffer was smaller). Prepend consumed samples for overlap context.
    let mut chunk = consumed;
    let take = buf.len().min(chunk_samples.saturating_sub(chunk.len()));
    chunk.extend(buf.iter().take(take).copied());

    if chunk.is_empty() {
        return None;
    }

    Some(chunk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::{Arc, Mutex};

    fn make_buffer(samples: &[f32]) -> AudioBuffer {
        let dq: VecDeque<f32> = samples.iter().copied().collect();
        Arc::new(Mutex::new(dq))
    }

    #[test]
    fn extract_returns_none_when_buffer_too_small() {
        // 1 second at 16 kHz = 16 000 samples — not enough for 2.5s chunk
        let buf = make_buffer(&vec![0.0; 16_000]);
        assert!(extract_chunk(&buf, 2.5, 0.5).is_none());
    }

    #[test]
    fn extract_returns_chunk_when_enough_data() {
        // 3 seconds — enough for a 2.5s chunk with 0.5s overlap
        let samples: Vec<f32> = (0..48_000).map(|i| i as f32).collect();
        let buf = make_buffer(&samples);
        let chunk = extract_chunk(&buf, 2.5, 0.5).unwrap();
        // chunk should be up to 40 000 samples
        assert!(chunk.len() <= 40_000);
        assert!(chunk.len() > 20_000);
    }

    #[test]
    fn extract_drains_old_samples() {
        let samples: Vec<f32> = (0..80_000).map(|i| i as f32).collect();
        let buf = make_buffer(&samples);

        let chunk1 = extract_chunk(&buf, 2.5, 0.5).unwrap();
        let len_after_first = buf.lock().unwrap().len();
        assert!(len_after_first < 80_000, "buffer should shrink after extraction");

        // Fill enough data for another extraction (need >= 32k new samples)
        let len_now = buf.lock().unwrap().len();
        for i in 0..40_000 {
            buf.lock().unwrap().push_back((80_000 + i) as f32);
        }
        let total = buf.lock().unwrap().len();
        assert!(total >= 32_000, "need enough data for second extraction, got {total}");

        let chunk2 = extract_chunk(&buf, 2.5, 0.5).unwrap();
        // The second chunk should start from a later offset (no full re-transcription)
        assert_ne!(chunk1[0], chunk2[0]);
    }
}

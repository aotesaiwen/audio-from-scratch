//! Lesson 05: Frames, chopping audio into analysable pieces.
//! See `lessons/05-frames/README.md`.
//!
//! This lesson is about borrowing: frames are *views* into the signal, not
//! copies, and some functions change the signal in place.
//! Check with `cargo test l05`, then run `cargo run --example lesson05`.

use crate::aliasing::zero_crossings;
use crate::level::{amplitude_to_db, rms};

/// How many whole frames of `frame_len` samples, starting every `hop`
/// samples, fit in a signal of `n_samples`? (No padding: a partial frame at
/// the end doesn't count.)
pub fn num_frames(n_samples: usize, frame_len: usize, hop: usize) -> usize {
    // Careful: usize can't go negative. `n_samples - frame_len` panics if
    // n_samples is smaller, so check that case first.
    todo!("l05: num_frames")
}

/// Split a signal into overlapping frames. Each frame is a slice that
/// *borrows* from `x`: no samples are copied.
pub fn frames(x: &[f64], frame_len: usize, hop: usize) -> Vec<&[f64]> {
    // Frame i is `&x[start..end]`. Push each one onto a Vec.
    todo!("l05: frames")
}

/// The loudness of each frame: its RMS, in dB.
pub fn frame_rms_db(x: &[f64], frame_len: usize, hop: usize) -> Vec<f64> {
    todo!("l05: frame_rms_db")
}

/// The zero-crossing rate of each frame: crossings divided by frame length.
/// Noisy sounds like "s" and "f" cross zero far more often than vowels.
pub fn frame_zcr(x: &[f64], frame_len: usize, hop: usize) -> Vec<f64> {
    todo!("l05: frame_zcr")
}

/// Multiply every sample by `gain`, changing the signal *in place*.
/// Note the `&mut`: the caller lends you their data and lets you modify it.
pub fn apply_gain(x: &mut [f64], gain: f64) {
    // `for v in x.iter_mut() { *v = ...; }`: the `*` writes through the reference.
    todo!("l05: apply_gain")
}

/// Fade in over the first `fade_len` samples and out over the last
/// `fade_len`, in place. Gain ramps linearly: sample i of the fade-in is
/// multiplied by i / fade_len (so the very first sample becomes 0), and the
/// fade-out mirrors it. If the signal is shorter than two fades, use
/// `fade_len = len / 2`. Fades stop clicks when audio starts or stops.
pub fn fade(x: &mut [f64], fade_len: usize) {
    todo!("l05: fade")
}

/// Add two signals sample by sample. If one is longer, its tail is kept.
pub fn mix(a: &[f64], b: &[f64]) -> Vec<f64> {
    // `a.get(i)` returns `Some(&value)` or `None` past the end, and
    // `.unwrap_or(&0.0)` supplies a default.
    todo!("l05: mix")
}

/// Cut silence from the start and end, like `librosa.effects.trim`.
///
/// Compute each frame's RMS in dB. Frames within `top_db` of the loudest
/// frame count as sound. Return the slice of `x` from the start of the first
/// loud frame to the end of the last one. If there are no frames at all,
/// return an empty slice.
///
/// The return type `&[f64]` borrows from `x`, so the trimmed audio is not
/// copied. Rust works out that the output borrows from the only input reference.
pub fn trim_silence(x: &[f64], frame_len: usize, hop: usize, top_db: f64) -> &[f64] {
    // `.position(|v| ...)` finds the first index where a condition holds;
    // `.rposition(...)` finds the last. Both return an Option.
    todo!("l05: trim_silence")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::sine;
    use crate::testutil::*;

    #[test]
    fn l05_num_frames() {
        assert_eq!(num_frames(10, 4, 2), 4); // [0..4] [2..6] [4..8] [6..10]
        assert_eq!(num_frames(11, 4, 2), 4); // the last sample doesn't make a frame
        assert_eq!(num_frames(3, 4, 2), 0);
        assert_eq!(num_frames(4, 4, 1), 1);
        assert_eq!(num_frames(16_000, 400, 160), 98); // 25 ms frames, 10 ms hop at 16 kHz
    }

    #[test]
    fn l05_frames_are_views() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let f = frames(&x, 4, 3);
        assert_eq!(f.len(), 3);
        assert_eq!(f[0], &[0.0, 1.0, 2.0, 3.0]);
        assert_eq!(f[1], &[3.0, 4.0, 5.0, 6.0]);
        assert_eq!(f[2], &[6.0, 7.0, 8.0, 9.0]);
        // Same memory as x, not a copy:
        assert!(std::ptr::eq(&f[1][0], &x[3]));
    }

    #[test]
    fn l05_frame_rms_db() {
        let db = frame_rms_db(&[0.5; 100], 20, 10);
        assert_eq!(db.len(), 9);
        for v in db {
            assert_close(v, -6.0206, 1e-4);
        }
    }

    #[test]
    fn l05_frame_zcr() {
        let x = [1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, 1.0];
        assert_all_close(&frame_zcr(&x, 4, 4), &[0.75, 0.0], 1e-12);
    }

    #[test]
    fn l05_apply_gain() {
        let mut x = vec![1.0, -2.0, 0.5];
        apply_gain(&mut x, 2.0);
        assert_eq!(x, vec![2.0, -4.0, 1.0]);
    }

    #[test]
    fn l05_fade() {
        let mut x = vec![1.0; 10];
        fade(&mut x, 4);
        assert_all_close(&x, &[0.0, 0.25, 0.5, 0.75, 1.0, 1.0, 0.75, 0.5, 0.25, 0.0], 1e-12);
        let mut short = vec![1.0; 3];
        fade(&mut short, 100); // must not panic
        assert_close(short[0], 0.0, 1e-12);
    }

    #[test]
    fn l05_mix() {
        assert_eq!(mix(&[1.0, 2.0, 3.0], &[0.5, 0.5]), vec![1.5, 2.5, 3.0]);
        assert_eq!(mix(&[], &[1.0]), vec![1.0]);
    }

    #[test]
    fn l05_trim_silence() {
        let sr = 16_000;
        let tone = sine(300.0, 0.5, sr, 0.5);
        let mut x = vec![0.0; 8_000];
        x.extend_from_slice(&tone);
        x.extend_from_slice(&[0.0; 8_000]);
        let trimmed = trim_silence(&x, 400, 160, 60.0);
        assert!(
            trimmed.len() >= tone.len() && trimmed.len() <= tone.len() + 800,
            "trimmed length {} should be close to the tone's {}",
            trimmed.len(),
            tone.len()
        );
        assert!(trim_silence(&[0.1; 10], 400, 160, 60.0).is_empty());
    }
}

//! Lesson 13: Pitch tracking. See `lessons/13-pitch/README.md`.
//!
//! Find F0 by asking "how far must I shift this frame until it lines up
//! with itself?" That's the autocorrelation method, the same family as
//! Praat's "To Pitch (ac)".
//! Check with `cargo test l13`, then run `cargo run --example lesson13`.

use crate::frames::frames;

/// Pitch-tracking settings. The defaults follow Praat's (floor 75 Hz,
/// ceiling 600 Hz, 10 ms steps, frames three floor-periods long).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchConfig {
    /// Lowest F0 to look for (Hz). Sets the longest lag searched.
    pub fmin: f64,
    /// Highest F0 to look for (Hz). Sets the shortest lag searched.
    pub fmax: f64,
    /// A frame counts as voiced if its best normalised autocorrelation
    /// peak reaches this (0 to 1).
    pub voicing_threshold: f64,
    pub frame_seconds: f64,
    pub hop_seconds: f64,
}

impl Default for PitchConfig {
    fn default() -> Self {
        PitchConfig { fmin: 75.0, fmax: 600.0, voicing_threshold: 0.5, frame_seconds: 0.04, hop_seconds: 0.01 }
    }
}

/// r[lag] = Σ_n x[n] · x[n + lag], for lag = 0 ..= max_lag.
/// Lags at or beyond the frame length have nothing to overlap: give them 0.
pub fn autocorrelation(x: &[f64], max_lag: usize) -> Vec<f64> {
    // For each lag, pair up x[..len−lag] with x[lag..] using zip.
    todo!("l13: autocorrelation")
}

/// Normalised autocorrelation: the correlation between `a = x[..len−lag]`
/// and `b = x[lag..]`, divided by √(energy(a) · energy(b)).
///
/// It's 1.0 when the shifted frame lines up perfectly with the original,
/// whatever the loudness, and it doesn't shrink at long lags the way raw
/// autocorrelation does. Lags with zero energy, or beyond the frame: 0.
pub fn normalized_autocorrelation(x: &[f64], max_lag: usize) -> Vec<f64> {
    todo!("l13: normalized_autocorrelation")
}

/// Fit a parabola through y[i−1], y[i], y[i+1] and return its peak as
/// (position, value). Turns a whole-sample peak into a sub-sample one.
///     p = 0.5 · (y[i−1] − y[i+1]) / (y[i−1] − 2·y[i] + y[i+1])
///     position = i + p,   value = y[i] − 0.25 · (y[i−1] − y[i+1]) · p
/// At either end of `y`, or if the three points are in a straight line,
/// return (i, y[i]) unchanged.
pub fn parabolic_peak(y: &[f64], i: usize) -> (f64, f64) {
    todo!("l13: parabolic_peak")
}

/// The heart of the pitch tracker. Returns `Some((period_in_samples,
/// strength))` for a voiced frame, or `None`.
///
/// 1. Lags to search: from floor(sr / fmax) (at least 1) to ceil(sr / fmin).
///    The frame must be longer than the largest lag + 1, otherwise None.
/// 2. Compute the normalised autocorrelation r up to the largest lag + 1.
/// 3. `best` = the largest r in the search range. Below the voicing
///    threshold: None.
/// 4. Pick the *first* lag in range that is a local peak (r[lag] ≥ both
///    neighbours) and reaches 0.9 · best. Taking the first, not the
///    biggest, avoids octave errors: a voice that repeats every T samples
///    also repeats every 2T, 3T, ...
/// 5. Refine it with `parabolic_peak` and return (position, value).
pub fn periodicity(frame: &[f64], sample_rate: u32, cfg: &PitchConfig) -> Option<(f64, f64)> {
    // `Iterator::find` returns an Option, and `?` returns None early if it's
    // None. That's how Option and `?` work together.
    todo!("l13: periodicity")
}

/// F0 (Hz) of one frame, or None if unvoiced.
pub fn estimate_f0(frame: &[f64], sample_rate: u32, cfg: &PitchConfig) -> Option<f64> {
    // `option.map(|value| ...)` transforms what's inside a Some.
    todo!("l13: estimate_f0")
}

/// Run `estimate_f0` on every frame: one Option<f64> per hop.
pub fn pitch_track(x: &[f64], sample_rate: u32, cfg: &PitchConfig) -> Vec<Option<f64>> {
    todo!("l13: pitch_track")
}

/// Mean and sample standard deviation (dividing by n − 1, as Praat does)
/// of the voiced frames, like Praat's "Get mean" / "Get standard deviation".
/// None if fewer than two frames are voiced.
pub fn mean_and_std(track: &[Option<f64>]) -> Option<(f64, f64)> {
    // `track.iter().flatten()` skips the Nones and unwraps the Somes.
    todo!("l13: mean_and_std")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::sine;
    use crate::synth::{voice, white_noise, VoiceParams};
    use crate::testutil::*;

    fn middle_frame(x: &[f64], sample_rate: u32) -> &[f64] {
        let len = (0.04 * sample_rate as f64) as usize;
        &x[x.len() / 2..x.len() / 2 + len]
    }

    #[test]
    fn l13_autocorrelation() {
        assert_all_close(&autocorrelation(&[1.0, 2.0, 3.0], 4), &[14.0, 8.0, 3.0, 0.0, 0.0], 1e-12);
    }

    #[test]
    fn l13_normalized_autocorrelation() {
        let x = sine(100.0, 0.3, 8_000, 0.1); // period: 80 samples
        let r = normalized_autocorrelation(&x, 100);
        assert_close(r[0], 1.0, 1e-12);
        assert_close(r[80], 1.0, 1e-9);
        assert_close(r[40], -1.0, 1e-9); // half a period: perfectly out of step
        assert_eq!(normalized_autocorrelation(&[0.0; 10], 3), vec![0.0; 4]);
    }

    #[test]
    fn l13_parabolic_peak() {
        let y: Vec<f64> = (0..6).map(|x| 5.0 - (x as f64 - 2.3).powi(2)).collect();
        let (pos, val) = parabolic_peak(&y, 2);
        assert_close(pos, 2.3, 1e-12);
        assert_close(val, 5.0, 1e-12);
        assert_eq!(parabolic_peak(&y, 0), (0.0, y[0]));
    }

    #[test]
    fn l13_sine_f0() {
        let x = sine(220.0, 0.5, 16_000, 0.2);
        let f0 = estimate_f0(middle_frame(&x, 16_000), 16_000, &PitchConfig::default()).expect("should be voiced");
        assert_close(f0, 220.0, 0.5);
    }

    #[test]
    fn l13_voice_f0_without_octave_errors() {
        for f0 in [95.0, 150.0, 210.0] {
            let v = voice(&VoiceParams { f0, ..Default::default() });
            let est = estimate_f0(middle_frame(&v.samples, v.sample_rate), v.sample_rate, &PitchConfig::default())
                .expect("should be voiced");
            assert_close(est, f0, 0.5);
        }
    }

    #[test]
    fn l13_missing_fundamental() {
        // Harmonics 2, 3 and 4 of 200 Hz, with no energy at 200 Hz itself.
        // The waveform still repeats every 5 ms, and that's what we hear.
        let (a, b, c) = (sine(400.0, 0.3, 16_000, 0.2), sine(600.0, 0.3, 16_000, 0.2), sine(800.0, 0.3, 16_000, 0.2));
        let x: Vec<f64> = (0..a.len()).map(|i| a[i] + b[i] + c[i]).collect();
        let f0 = estimate_f0(middle_frame(&x, 16_000), 16_000, &PitchConfig::default()).expect("should be voiced");
        assert_close(f0, 200.0, 1.0);
    }

    #[test]
    fn l13_noise_is_unvoiced() {
        let x = white_noise(4_000, 0.3, 42);
        assert_eq!(estimate_f0(middle_frame(&x, 16_000), 16_000, &PitchConfig::default()), None);
    }

    #[test]
    fn l13_frame_too_short() {
        assert_eq!(estimate_f0(&[0.1; 50], 16_000, &PitchConfig::default()), None);
    }

    #[test]
    fn l13_pitch_track() {
        let v = voice(&VoiceParams { f0: 120.0, seconds: 0.5, sample_rate: 16_000, ..Default::default() });
        let mut x = vec![0.0; 8_000]; // 0.5 s of silence first
        x.extend_from_slice(&v.samples);
        let track = pitch_track(&x, 16_000, &PitchConfig::default());
        assert_eq!(track.len(), 1 + (16_000 - 640) / 160);
        assert!(track[..40].iter().all(|f| f.is_none()), "silence must be unvoiced");
        for f in track[60..90].iter() {
            assert_close(f.expect("voice must be voiced"), 120.0, 1.0);
        }
    }

    #[test]
    fn l13_mean_and_std() {
        let (mean, std) = mean_and_std(&[Some(100.0), None, Some(110.0), Some(120.0)]).unwrap();
        assert_close(mean, 110.0, 1e-12);
        assert_close(std, 10.0, 1e-12);
        assert_eq!(mean_and_std(&[None, Some(1.0)]), None);
    }
}

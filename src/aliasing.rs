//! Lesson 04: Nyquist and aliasing. See `lessons/04-aliasing/README.md`.
//!
//! Check with `cargo test l04`, then run `cargo run --example lesson04` and
//! listen to what comes out.

use crate::signal::{num_samples, sine};
use std::f64::consts::PI;

/// The Nyquist frequency: half the sample rate. The highest frequency a
/// sampled signal can represent unambiguously.
pub fn nyquist(sample_rate: u32) -> f64 {
    todo!("l04: nyquist")
}

/// The frequency you actually get (between 0 and Nyquist) when you sample a
/// sine of `freq` Hz at `sample_rate`.
///
/// Sampling can't tell `freq` apart from `freq + k·sample_rate` for any whole
/// number k, or from its mirror image `sample_rate − freq`. So: wrap `freq`
/// into [0, sample_rate), then fold anything above Nyquist back down.
pub fn alias_frequency(freq: f64, sample_rate: u32) -> f64 {
    // `a.rem_euclid(b)` is the remainder of a / b, always in [0, b), even
    // for negative a (unlike `%`). An `if` in Rust is an expression:
    //     let y = if cond { a } else { b };
    todo!("l04: alias_frequency")
}

/// Count sign changes between neighbouring samples. Treat 0.0 as positive
/// (librosa does the same), so [0.0, 1.0] has no crossing.
pub fn zero_crossings(x: &[f64]) -> usize {
    // `x.windows(2)` walks overlapping pairs [x0,x1], [x1,x2], ...
    // `.filter(|w| ...)` keeps the ones where the condition is true, and
    // `.count()` counts what's left.
    todo!("l04: zero_crossings")
}

/// Estimate a sine's frequency by counting zero crossings: a sine crosses
/// zero twice per cycle.
pub fn estimate_frequency(x: &[f64], sample_rate: u32) -> f64 {
    todo!("l04: estimate_frequency")
}

/// A square wave made the naive way: +1 when a sine is ≥ 0, −1 otherwise.
/// Its sharp corners contain harmonics far above Nyquist, and they alias.
pub fn naive_square(freq: f64, sample_rate: u32, seconds: f64) -> Vec<f64> {
    todo!("l04: naive_square")
}

/// A square wave is the sum of its odd harmonics (k = 1, 3, 5, ...) with
/// amplitude 4/(πk). How many of them lie strictly below Nyquist?
pub fn odd_harmonics_below_nyquist(freq: f64, sample_rate: u32) -> usize {
    // A `while` loop over k = 1, 3, 5, ... is the clearest way.
    todo!("l04: odd_harmonics_below_nyquist")
}

/// Build a square wave from its first `n_harmonics` odd harmonics:
/// Σ (4/(πk)) · sin(2π·k·freq·t) for k = 1, 3, 5, ... (n_harmonics terms).
/// Stop below Nyquist and nothing aliases: a "band-limited" square wave.
pub fn square_from_harmonics(freq: f64, sample_rate: u32, seconds: f64, n_harmonics: usize) -> Vec<f64> {
    // `.iter_mut()` gives mutable references, so you can write
    //     for (o, v) in out.iter_mut().zip(&h) { *o += v; }
    todo!("l04: square_from_harmonics")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn l04_nyquist() {
        assert_close(nyquist(44_100), 22_050.0, 1e-12);
        assert_close(nyquist(16_000), 8_000.0, 1e-12);
    }

    #[test]
    fn l04_alias_frequency() {
        assert_close(alias_frequency(1_000.0, 8_000), 1_000.0, 1e-9); // fine: below Nyquist
        assert_close(alias_frequency(5_000.0, 8_000), 3_000.0, 1e-9); // folds back
        assert_close(alias_frequency(8_000.0, 8_000), 0.0, 1e-9); // looks like DC
        assert_close(alias_frequency(9_000.0, 8_000), 1_000.0, 1e-9); // wraps around
        assert_close(alias_frequency(15_000.0, 8_000), 1_000.0, 1e-9);
    }

    /// Nothing to implement here. This test already passes. Lesson 04's README
    /// asks you to explain *why* it passes.
    #[test]
    fn l04_a_sine_and_its_mirror_are_indistinguishable() {
        let a = sine(1_000.0, 1.0, 8_000, 0.01);
        let b = sine(7_000.0, 1.0, 8_000, 0.01);
        for (x, y) in a.iter().zip(&b) {
            assert_close(*x, -*y, 1e-9);
        }
    }

    #[test]
    fn l04_zero_crossings() {
        assert_eq!(zero_crossings(&[1.0, -1.0, 1.0]), 2);
        assert_eq!(zero_crossings(&[0.0, 1.0, 2.0]), 0);
        assert_eq!(zero_crossings(&[-1.0, 0.0, -1.0]), 2);
        assert_eq!(zero_crossings(&[]), 0);
    }

    #[test]
    fn l04_estimate_frequency() {
        assert_close(estimate_frequency(&sine(440.0, 1.0, 44_100, 1.0), 44_100), 440.0, 1.0);
        // A 7 kHz sine sampled at 8 kHz *is* a 1 kHz sine, as far as the
        // samples are concerned.
        assert_close(estimate_frequency(&sine(7_000.0, 1.0, 8_000, 1.0), 8_000), 1_000.0, 2.0);
    }

    #[test]
    fn l04_naive_square() {
        // 7 samples per cycle, so no sample lands exactly on a zero crossing
        let x = naive_square(1_000.0, 7_000, 0.001);
        assert_eq!(x, vec![1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0]);
    }

    #[test]
    fn l04_odd_harmonics_below_nyquist() {
        assert_eq!(odd_harmonics_below_nyquist(1_000.0, 8_000), 2); // 1k, 3k
        assert_eq!(odd_harmonics_below_nyquist(100.0, 44_100), 110); // 100 Hz .. 21.9 kHz
        assert_eq!(odd_harmonics_below_nyquist(5_000.0, 8_000), 0);
    }

    #[test]
    fn l04_square_from_harmonics() {
        let one = square_from_harmonics(100.0, 8_000, 0.05, 1);
        assert_all_close(&one, &sine(100.0, 4.0 / PI, 8_000, 0.05), 1e-12);
        // With many harmonics it approaches ±1 (plus Gibbs ripple near edges).
        let sq = square_from_harmonics(50.0, 48_000, 0.02, 200);
        assert_close(sq[240], 1.0, 0.02); // a quarter-cycle in
        assert_close(sq[720], -1.0, 0.02); // three quarters in
    }
}

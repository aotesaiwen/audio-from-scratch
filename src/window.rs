//! Lesson 10: Windows and spectral leakage. See `lessons/10-windows/README.md`.
//!
//! Check with `cargo test l10`, then run `cargo run --example lesson10`.

use crate::level::amplitude_to_db;
use std::f64::consts::PI;

/// The rectangular window: all ones, i.e. "no window". Given.
pub fn rectangular(n: usize) -> Vec<f64> {
    vec![1.0; n]
}

/// The Hann window, periodic form: w[i] = 0.5 − 0.5·cos(2π·i/n).
/// ("Periodic" divides by n, not n − 1. That's the version for spectral
/// analysis, and what librosa uses.)
pub fn hann(n: usize) -> Vec<f64> {
    (0..n).map(|i| 0.5 - 0.5 * (2.0 * PI * i as f64 / n as f64).cos()).collect()
}

/// The Hamming window, periodic form: w[i] = 0.54 − 0.46·cos(2π·i/n).
pub fn hamming(n: usize) -> Vec<f64> {
    (0..n).map(|i| 0.54 - 0.46 * (2.0 * PI * i as f64 / n as f64).cos()).collect()
}

/// Multiply a frame by a window, sample by sample. Panics if the lengths differ.
pub fn apply_window(x: &[f64], w: &[f64]) -> Vec<f64> {
    assert_eq!(x.len(), w.len(), "frame and window lengths differ");
    x.iter().zip(w).map(|(a, b)| a * b).collect()
}

/// The mean of the window. A windowed sine's spectral peak is this much
/// smaller than an unwindowed one's (0.5 for Hann).
pub fn coherent_gain(w: &[f64]) -> f64 {
    w.iter().sum::<f64>() / w.len() as f64
}

/// How badly a single tone has leaked across the spectrum: the biggest
/// magnitude more than `guard` bins away from the peak, relative to the
/// peak, in dB. −20 dB means bad leakage; −100 dB means essentially none.
pub fn max_sidelobe_db(magnitudes: &[f64], guard: usize) -> f64 {
    let (peak_bin, peak) = magnitudes
        .iter()
        .copied()
        .enumerate()
        .max_by(|a, b| a.1.total_cmp(&b.1))
        .expect("empty spectrum");
    let (lo, hi) = (peak_bin.saturating_sub(guard), peak_bin + guard);
    let worst = magnitudes
        .iter()
        .enumerate()
        .filter(|&(i, _)| i < lo || i > hi)
        .map(|(_, &m)| m)
        .fold(0.0, f64::max);
    amplitude_to_db(worst / peak)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fft::rfft;
    use crate::signal::sine;
    use crate::testutil::*;

    fn magnitudes(x: &[f64]) -> Vec<f64> {
        rfft(x, x.len()).iter().map(|c| c.abs()).collect()
    }

    #[test]
    fn l10_hann_values() {
        assert_all_close(&hann(4), &[0.0, 0.5, 1.0, 0.5], 1e-12);
    }

    #[test]
    fn l10_hamming_values() {
        let w = hamming(8);
        assert_close(w[0], 0.08, 1e-12);
        assert_close(w[4], 1.0, 1e-12);
    }

    #[test]
    fn l10_periodic_symmetry() {
        let w = hann(16);
        for i in 1..16 {
            assert_close(w[i], w[16 - i], 1e-12);
        }
    }

    #[test]
    fn l10_apply_window_and_gain() {
        assert_all_close(&apply_window(&[2.0, 2.0, 2.0, 2.0], &hann(4)), &[0.0, 1.0, 2.0, 1.0], 1e-12);
        assert_close(coherent_gain(&hann(1024)), 0.5, 1e-12);
        assert_close(coherent_gain(&rectangular(10)), 1.0, 1e-12);
    }

    #[test]
    fn l10_on_bin_tone_does_not_leak() {
        // 1000 Hz at 8 kHz with N = 1024: exactly bin 128.
        let x = sine(1_000.0, 1.0, 8_000, 1024.0 / 8_000.0);
        assert!(max_sidelobe_db(&magnitudes(&x), 1) < -100.0);
    }

    #[test]
    fn l10_off_bin_tone_leaks_without_a_window() {
        // Halfway between bins 128 and 129: the worst case.
        let x = sine(1_003.906_25, 1.0, 8_000, 1024.0 / 8_000.0);
        let leak = max_sidelobe_db(&magnitudes(&x), 3);
        assert!(leak > -20.0, "expected heavy leakage, got {leak:.1} dB");
    }

    #[test]
    fn l10_hann_tames_leakage() {
        let x = sine(1_003.906_25, 1.0, 8_000, 1024.0 / 8_000.0);
        let leak = max_sidelobe_db(&magnitudes(&apply_window(&x, &hann(x.len()))), 3);
        assert!(leak < -35.0, "expected little leakage, got {leak:.1} dB");
    }
}

//! Lesson 06: Convolution and FIR filters. See `lessons/06-fir-filters/README.md`.
//!
//! Check with `cargo test l06`, then run `cargo run --example lesson06`.

use crate::level::{amplitude_to_db, rms};
use crate::signal::sine;

/// Full convolution: y[n] = Σ_k h[k] · x[n − k].
/// The output has `x.len() + h.len() − 1` samples. Empty input: empty output.
pub fn convolve(x: &[f64], h: &[f64]) -> Vec<f64> {
    // Read the README section on the two ways to write this loop. One of
    // them needs `n - k` on usizes, which panics when k > n.
    todo!("l06: convolve")
}

/// Filter `x` through the FIR filter `h`, keeping the output the same length
/// as the input (the first `x.len()` samples of the full convolution).
pub fn fir_filter(x: &[f64], h: &[f64]) -> Vec<f64> {
    todo!("l06: fir_filter")
}

/// An n-tap moving average: n copies of 1/n.
pub fn moving_average_kernel(n: usize) -> Vec<f64> {
    todo!("l06: moving_average_kernel")
}

/// Pre-emphasis: y[n] = x[n] − alpha · x[n−1], with y[0] = x[0].
/// The first step of classic MFCC and LPC pipelines (alpha ≈ 0.97). It
/// boosts the high frequencies, which voiced speech is weak in.
pub fn pre_emphasis(x: &[f64], alpha: f64) -> Vec<f64> {
    // This is just an FIR filter. What is its kernel h?
    todo!("l06: pre_emphasis")
}

/// The impulse response of a single echo: the original sound, then a copy
/// `delay_samples` later, scaled by `decay`. Length `delay_samples + 1`.
pub fn echo_kernel(delay_samples: usize, decay: f64) -> Vec<f64> {
    todo!("l06: echo_kernel")
}

/// Measure a filter's gain (dB) at one frequency, by experiment: play a
/// 0.5 s sine through it and compare output RMS to input RMS over the second
/// half (skipping the start, while the filter is "warming up").
///
/// `filter` can be any function or closure that takes a signal and returns
/// a signal: `measure_gain_db(|x| fir_filter(x, &h), 1000.0, 16_000)`.
pub fn measure_gain_db(filter: impl Fn(&[f64]) -> Vec<f64>, freq: f64, sample_rate: u32) -> f64 {
    // Call the closure like a function: `let y = filter(&x);`
    todo!("l06: measure_gain_db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn l06_convolve_by_hand() {
        assert_all_close(&convolve(&[1.0, 2.0, 3.0], &[0.0, 1.0, 0.5]), &[0.0, 1.0, 2.5, 4.0, 1.5], 1e-12);
        assert!(convolve(&[], &[1.0]).is_empty());
    }

    #[test]
    fn l06_convolve_with_impulse_is_identity() {
        let x = [0.3, -0.1, 0.7, 0.2];
        assert_all_close(&convolve(&x, &[1.0]), &x, 1e-12);
        // A delayed impulse delays the signal.
        assert_all_close(&convolve(&x, &[0.0, 0.0, 1.0]), &[0.0, 0.0, 0.3, -0.1, 0.7, 0.2], 1e-12);
    }

    #[test]
    fn l06_convolution_commutes() {
        let (a, b) = ([1.0, -2.0, 0.5, 4.0], [0.25, 0.5, -1.0]);
        assert_all_close(&convolve(&a, &b), &convolve(&b, &a), 1e-12);
    }

    #[test]
    fn l06_fir_filter_keeps_length() {
        let y = fir_filter(&[1.0, 2.0, 3.0], &[0.0, 1.0, 0.5]);
        assert_all_close(&y, &[0.0, 1.0, 2.5], 1e-12);
    }

    #[test]
    fn l06_kernels() {
        assert_all_close(&moving_average_kernel(4), &[0.25; 4], 1e-12);
        assert_all_close(&echo_kernel(3, 0.5), &[1.0, 0.0, 0.0, 0.5], 1e-12);
    }

    #[test]
    fn l06_pre_emphasis() {
        assert_all_close(&pre_emphasis(&[1.0, 1.0, 1.0], 0.97), &[1.0, 0.03, 0.03], 1e-12);
        assert_all_close(&pre_emphasis(&[0.0, 1.0, 0.0], 0.5), &[0.0, 1.0, -0.5], 1e-12);
    }

    #[test]
    fn l06_moving_average_is_a_lowpass() {
        let h = moving_average_kernel(5);
        assert_close(measure_gain_db(|x| fir_filter(x, &h), 100.0, 8_000), 0.0, 0.2);
        // A 5-sample average exactly cancels a wave whose period is 5 samples.
        assert!(measure_gain_db(|x| fir_filter(x, &h), 1_600.0, 8_000) < -40.0);
    }

    #[test]
    fn l06_pre_emphasis_is_a_highpass() {
        let low = measure_gain_db(|x| pre_emphasis(x, 0.97), 50.0, 16_000);
        let high = measure_gain_db(|x| pre_emphasis(x, 0.97), 4_000.0, 16_000);
        assert!(low < -25.0, "50 Hz should be cut hard, got {low:.1} dB");
        assert_close(high, 2.88, 0.2);
    }
}

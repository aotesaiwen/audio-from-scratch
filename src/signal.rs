//! Lesson 01: Sound is a list of numbers. See `lessons/01-sampling/README.md`.
//!
//! Check with `cargo test l01`, then run `cargo run --example lesson01`.

use std::f64::consts::PI;

/// How many samples make up `seconds` of audio at `sample_rate` Hz?
/// Round to the nearest whole sample.
pub fn num_samples(sample_rate: u32, seconds: f64) -> usize {
    (sample_rate as f64 * seconds).round() as usize
}

/// The time, in seconds, of each of the first `n` samples:
/// `[0, 1/sr, 2/sr, ...]`.
pub fn time_axis(sample_rate: u32, n: usize) -> Vec<f64> {
    let mut t = Vec::with_capacity(n);
    for i in 0..n {
        t.push(i as f64 / sample_rate as f64);
    }
    t
}

/// How many samples one cycle of a `freq` Hz wave lasts. Usually not a
/// whole number: a 440 Hz cycle at 44.1 kHz is 100.227... samples long.
pub fn samples_per_cycle(freq: f64, sample_rate: u32) -> f64 {
    sample_rate as f64 / freq
}

/// A sine wave lasting `seconds`:  x[n] = amplitude · sin(2π · freq · t),
/// where t = n / sample_rate is the time of sample n.
pub fn sine(freq: f64, amplitude: f64, sample_rate: u32, seconds: f64) -> Vec<f64> {
    let n = num_samples(sample_rate, seconds);
    let mut x = Vec::with_capacity(n);
    for i in 0..n {
        let t = i as f64 / sample_rate as f64;
        x.push(amplitude * (2.0 * PI * freq * t).sin());
    }
    x
}

/// Several sines added together, each with the same amplitude: a chord.
/// The result is as long as a single `sine` of that duration.
pub fn chord(freqs: &[f64], amplitude: f64, sample_rate: u32, seconds: f64) -> Vec<f64> {
    let mut out = vec![0.0; num_samples(sample_rate, seconds)];
    for &f in freqs {
        let s = sine(f, amplitude, sample_rate, seconds);
        for i in 0..out.len() {
            out[i] += s[i];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn l01_num_samples() {
        assert_eq!(num_samples(16_000, 1.0), 16_000);
        assert_eq!(num_samples(44_100, 0.5), 22_050);
        assert_eq!(num_samples(8_000, 0.0), 0);
        assert_eq!(num_samples(44_100, 0.01), 441);
    }

    #[test]
    fn l01_time_axis() {
        assert_all_close(&time_axis(4, 4), &[0.0, 0.25, 0.5, 0.75], 1e-12);
        assert!(time_axis(8_000, 0).is_empty());
    }

    #[test]
    fn l01_samples_per_cycle() {
        assert_close(samples_per_cycle(100.0, 8_000), 80.0, 1e-12);
        assert_close(samples_per_cycle(440.0, 44_100), 100.227_272, 1e-5);
    }

    #[test]
    fn l01_sine_length_and_start() {
        let x = sine(440.0, 1.0, 44_100, 0.5);
        assert_eq!(x.len(), 22_050);
        assert_close(x[0], 0.0, 1e-12); // sin(0) = 0
    }

    #[test]
    fn l01_sine_at_quarter_sample_rate() {
        // Four samples per cycle: 0, peak, 0, trough, 0, peak, ...
        let x = sine(4_000.0, 1.0, 16_000, 0.0005);
        assert_all_close(&x, &[0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0], 1e-9);
    }

    #[test]
    fn l01_sine_peak_is_amplitude() {
        let x = sine(441.0, 0.3, 44_100, 1.0);
        let mut biggest = 0.0f64;
        for &v in &x {
            biggest = biggest.max(v.abs());
        }
        assert_close(biggest, 0.3, 1e-9);
    }

    #[test]
    fn l01_chord_is_a_sum_of_sines() {
        let c = chord(&[220.0, 330.0], 0.25, 8_000, 0.1);
        let a = sine(220.0, 0.25, 8_000, 0.1);
        let b = sine(330.0, 0.25, 8_000, 0.1);
        assert_eq!(c.len(), a.len());
        for i in 0..c.len() {
            assert_close(c[i], a[i] + b[i], 1e-12);
        }
    }
}

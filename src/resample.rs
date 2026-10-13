//! Lesson 15 (bonus): Resampling. See `lessons/15-resampling/README.md`.
//!
//! What `torchaudio.transforms.Resample(44100, 16000)` does, and why you
//! can't just throw samples away.
//! Check with `cargo test l15`, then run `cargo run --example lesson15`.

use std::f64::consts::PI;

/// Keep every `factor`-th sample and drop the rest. Fast and wrong: anything
/// above the new Nyquist frequency aliases.
pub fn decimate_naive(x: &[f64], factor: usize) -> Vec<f64> {
    x.iter().step_by(factor).copied().collect()
}

/// Number of output samples: round(len · sr_out / sr_in).
pub fn output_len(len: usize, sr_in: u32, sr_out: u32) -> usize {
    (len as f64 * sr_out as f64 / sr_in as f64).round() as usize
}

/// Resample by drawing straight lines between input samples. Output sample
/// i sits at input position t = i · sr_in / sr_out; blend the two input
/// samples either side of t. Past the last sample, hold the last value.
pub fn resample_linear(x: &[f64], sr_in: u32, sr_out: u32) -> Vec<f64> {
    if x.is_empty() {
        return Vec::new();
    }
    let step = sr_in as f64 / sr_out as f64;
    let last = x.len() - 1;
    (0..output_len(x.len(), sr_in, sr_out))
        .map(|i| {
            let t = i as f64 * step;
            let k = (t.floor() as usize).min(last);
            let frac = t - k as f64;
            let (a, b) = (x[k], x[(k + 1).min(last)]);
            a + (b - a) * frac
        })
        .collect()
}

/// The normalised sinc function, sin(πx)/(πx), with sinc(0) = 1.
pub fn sinc(x: f64) -> f64 {
    if x.abs() < 1e-12 { 1.0 } else { (PI * x).sin() / (PI * x) }
}

/// Band-limited resampling with a Hann-windowed sinc kernel.
///
/// Output sample i sits at input position t = i · sr_in / sr_out, and is
/// a weighted sum of nearby input samples x[k]:
///     y[i] = Σ_k x[k] · cutoff · sinc(cutoff · (t − k)) · w(t − k)
/// where
///  - cutoff = 0.99 · min(1, sr_out / sr_in), as a fraction of the input
///    Nyquist. When downsampling it drops below 1, which low-passes the
///    signal so nothing aliases. (0.99 leaves a little room, like torchaudio's rolloff.)
///  - the kernel spans |t − k| < half_width, where
///    half_width = zero_crossings / cutoff input samples
///  - w(d) = 0.5 + 0.5·cos(π·d / half_width), a Hann taper to zero at the edges.
///
/// Skip k outside the signal (treat those samples as zero).
pub fn resample_sinc(x: &[f64], sr_in: u32, sr_out: u32, zero_crossings: usize) -> Vec<f64> {
    if x.is_empty() {
        return Vec::new();
    }
    let ratio = sr_out as f64 / sr_in as f64;
    let cutoff = 0.99 * ratio.min(1.0);
    let half_width = zero_crossings as f64 / cutoff;
    (0..output_len(x.len(), sr_in, sr_out))
        .map(|i| {
            let t = i as f64 / ratio;
            let k_min = (t - half_width).ceil().max(0.0) as usize;
            let k_max = ((t + half_width).floor() as usize).min(x.len() - 1);
            (k_min..=k_max)
                .map(|k| {
                    let d = t - k as f64;
                    let w = 0.5 + 0.5 * (PI * d / half_width).cos();
                    x[k] * cutoff * sinc(cutoff * d) * w
                })
                .sum()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fft::rfft;
    use crate::level::amplitude_to_db;
    use crate::signal::sine;
    use crate::testutil::*;
    use crate::window::{apply_window, hann};

    /// Magnitude at `freq`, relative to the magnitude at `reference`, in dB.
    fn relative_db(x: &[f64], sr: u32, freq: f64, reference: f64) -> f64 {
        let n = 8192;
        let spec = rfft(&apply_window(&x[..n], &hann(n)), n);
        let bin = |f: f64| (f * n as f64 / sr as f64).round() as usize;
        amplitude_to_db(spec[bin(freq)].abs() / spec[bin(reference)].abs())
    }

    #[test]
    fn l15_decimate_naive() {
        let x: Vec<f64> = (0..10).map(|i| i as f64).collect();
        assert_eq!(decimate_naive(&x, 3), vec![0.0, 3.0, 6.0, 9.0]);
    }

    #[test]
    fn l15_output_len() {
        assert_eq!(output_len(48_000, 48_000, 16_000), 16_000);
        assert_eq!(output_len(16_000, 16_000, 44_100), 44_100);
        assert_eq!(output_len(100, 44_100, 16_000), 36);
    }

    #[test]
    fn l15_linear_keeps_a_ramp_straight() {
        let ramp: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let up = resample_linear(&ramp, 1_000, 3_000);
        for (i, &v) in up.iter().enumerate().take(290) {
            assert_close(v, i as f64 / 3.0, 1e-9);
        }
    }

    #[test]
    fn l15_sinc() {
        assert_close(sinc(0.0), 1.0, 1e-12);
        assert_close(sinc(1.0), 0.0, 1e-12);
        assert_close(sinc(-3.0), 0.0, 1e-12);
        assert_close(sinc(0.5), 2.0 / PI, 1e-12);
    }

    #[test]
    fn l15_sinc_upsampling_preserves_a_sine() {
        let x = sine(1_000.0, 0.5, 16_000, 0.1);
        let y = resample_sinc(&x, 16_000, 48_000, 16);
        let expected = sine(1_000.0, 0.5, 48_000, 0.1);
        assert_eq!(y.len(), expected.len());
        // Away from the edges, where the kernel runs off the signal:
        for i in 600..y.len() - 600 {
            assert_close(y[i], expected[i], 0.01);
        }
    }

    #[test]
    fn l15_naive_decimation_aliases_but_sinc_does_not() {
        // 1 kHz (keep) + 10 kHz (above the new 8 kHz Nyquist), at 48 kHz.
        let x: Vec<f64> = sine(1_000.0, 0.5, 48_000, 1.0)
            .iter()
            .zip(sine(10_000.0, 0.5, 48_000, 1.0))
            .map(|(a, b)| a + b)
            .collect();
        // At 16 kHz, 10 kHz would alias to 16 − 10 = 6 kHz.
        let naive = decimate_naive(&x, 3);
        let proper = resample_sinc(&x, 48_000, 16_000, 16);
        assert!(relative_db(&naive, 16_000, 6_000.0, 1_000.0) > -3.0);
        assert!(relative_db(&proper, 16_000, 6_000.0, 1_000.0) < -40.0);
    }
}

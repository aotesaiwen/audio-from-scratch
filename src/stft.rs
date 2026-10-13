//! Lesson 11: The STFT and spectrograms. See `lessons/11-stft/README.md`.
//!
//! Frames (lesson 05) + a window (lesson 10) + the FFT (lesson 09).
//! Check with `cargo test l11`, then run `cargo run --example lesson11`.

use crate::complex::Complex;
use crate::fft::rfft;
use crate::frames::frames;
use crate::window::{apply_window, hann};

/// STFT settings, named as in librosa.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StftConfig {
    /// Samples per frame (and FFT size). A power of two.
    pub n_fft: usize,
    /// Samples between the starts of consecutive frames.
    pub hop_length: usize,
    /// Pad n_fft/2 zeros at both ends so frame i is *centred* on sample
    /// i·hop_length (librosa's default).
    pub center: bool,
}

/// `Default` lets you write `StftConfig::default()`, or override just some
/// fields: `StftConfig { n_fft: 512, ..Default::default() }`.
impl Default for StftConfig {
    fn default() -> Self {
        // Use librosa's defaults: n_fft 2048, hop_length 512, center true.
        todo!("l11: StftConfig::default")
    }
}

/// The short-time Fourier transform: for each frame, multiply by a Hann
/// window and take the rfft. Returns `[frame][bin]`, n_fft/2 + 1 bins per frame.
pub fn stft(x: &[f64], cfg: &StftConfig) -> Vec<Vec<Complex>> {
    // If cfg.center, build a padded copy first: n_fft/2 zeros, x, n_fft/2 zeros.
    todo!("l11: stft")
}

/// |STFT|²: the power spectrogram, `[frame][bin]`.
pub fn power_spectrogram(x: &[f64], cfg: &StftConfig) -> Vec<Vec<f64>> {
    todo!("l11: power_spectrogram")
}

/// Power to decibels, like `librosa.power_to_db(S, ref=np.max, top_db=...)`:
/// 10·log10(p / max), so the loudest cell is 0 dB, then anything below
/// −top_db is raised to −top_db. Floor p at 1e-10 before taking the log.
pub fn power_to_db(spec: &[Vec<f64>], top_db: f64) -> Vec<Vec<f64>> {
    // `spec.iter().flatten()` walks every value in every row.
    todo!("l11: power_to_db")
}

/// The time (s) each frame is centred on (assuming `center: true`).
pub fn frame_times(n_frames: usize, hop_length: usize, sample_rate: u32) -> Vec<f64> {
    todo!("l11: frame_times")
}

/// The frequency (Hz) of each of the n_fft/2 + 1 bins.
pub fn bin_frequencies(n_fft: usize, sample_rate: u32) -> Vec<f64> {
    todo!("l11: bin_frequencies")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::sine;
    use crate::synth::chirp;
    use crate::testutil::*;

    fn argmax(v: &[f64]) -> usize {
        (0..v.len()).max_by(|&a, &b| v[a].total_cmp(&v[b])).unwrap()
    }

    #[test]
    fn l11_default_matches_librosa() {
        let cfg = StftConfig::default();
        assert_eq!((cfg.n_fft, cfg.hop_length, cfg.center), (2048, 512, true));
    }

    #[test]
    fn l11_shape_centered() {
        let s = stft(&vec![0.0; 16_000], &StftConfig::default());
        assert_eq!(s.len(), 32, "1 + 16000/512 frames");
        assert_eq!(s[0].len(), 1025);
    }

    #[test]
    fn l11_shape_uncentered() {
        let cfg = StftConfig { center: false, ..Default::default() };
        assert_eq!(stft(&vec![0.0; 16_000], &cfg).len(), 28, "1 + (16000 − 2048)/512 frames");
    }

    #[test]
    fn l11_sine_lands_in_its_bin() {
        let cfg = StftConfig { n_fft: 512, hop_length: 128, center: true };
        let spec = power_spectrogram(&sine(1_000.0, 0.5, 16_000, 0.5), &cfg);
        // bin spacing 16000/512 = 31.25 Hz, so 1000 Hz is bin 32
        assert_eq!(argmax(&spec[spec.len() / 2]), 32);
    }

    #[test]
    fn l11_chirp_rises() {
        let cfg = StftConfig { n_fft: 512, hop_length: 128, center: true };
        let spec = power_spectrogram(&chirp(200.0, 6_000.0, 0.5, 16_000, 1.0), &cfg);
        let n = spec.len();
        let (a, b, c) = (argmax(&spec[n / 4]), argmax(&spec[n / 2]), argmax(&spec[3 * n / 4]));
        assert!(a < b && b < c, "peak bins should rise over time: {a}, {b}, {c}");
    }

    #[test]
    fn l11_power_to_db() {
        let db = power_to_db(&[vec![1.0, 0.1], vec![1e-12, 0.01]], 80.0);
        assert_all_close(&db[0], &[0.0, -10.0], 1e-9);
        assert_all_close(&db[1], &[-80.0, -20.0], 1e-9);
    }

    #[test]
    fn l11_axes() {
        assert_all_close(&frame_times(3, 512, 16_000), &[0.0, 0.032, 0.064], 1e-12);
        assert_all_close(&bin_frequencies(8, 16_000), &[0.0, 2_000.0, 4_000.0, 6_000.0, 8_000.0], 1e-9);
    }
}

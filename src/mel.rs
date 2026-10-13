//! Lesson 12: Mel spectrograms and MFCCs. See `lessons/12-mel-mfcc/README.md`.
//!
//! The standard inputs to speech classifiers, built from parts you now own.
//! Check with `cargo test l12`, then run `cargo run --example lesson12`.

use crate::stft::{bin_frequencies, power_spectrogram, StftConfig};
use std::f64::consts::PI;

/// Hz to mel, HTK formula: 2595 · log10(1 + hz/700).
/// (librosa's default is the slightly different "Slaney" mel scale; pass
/// `htk=True` to librosa to get this one.)
pub fn hz_to_mel(hz: f64) -> f64 {
    todo!("l12: hz_to_mel")
}

/// Mel to Hz: the inverse of `hz_to_mel`.
pub fn mel_to_hz(mel: f64) -> f64 {
    todo!("l12: mel_to_hz")
}

/// A bank of `n_mels` triangular filters, `[filter][bin]`, each row as long
/// as one STFT frame (n_fft/2 + 1).
///
/// Space n_mels + 2 edge points evenly on the *mel* scale from fmin to fmax,
/// and convert them back to Hz. Filter m rises linearly from 0 at edge m to
/// 1 at edge m+1, then falls back to 0 at edge m+2. Evaluate each triangle
/// at every bin's frequency (no normalisation: peaks are 1).
/// This matches `librosa.filters.mel(..., htk=True, norm=None)`.
pub fn mel_filterbank(n_mels: usize, n_fft: usize, sample_rate: u32, fmin: f64, fmax: f64) -> Vec<Vec<f64>> {
    // For a bin at frequency f and a triangle (lo, centre, hi):
    //     rising  = (f − lo) / (centre − lo)
    //     falling = (hi − f) / (hi − centre)
    //     weight  = the smaller of the two, but never below 0
    todo!("l12: mel_filterbank")
}

/// Apply a filterbank to every frame of a power spectrogram:
/// out[frame][m] = Σ_bin fb[m][bin] · power[frame][bin].
/// That's a matrix product, written as nested iterators.
pub fn apply_filterbank(power: &[Vec<f64>], fb: &[Vec<f64>]) -> Vec<Vec<f64>> {
    todo!("l12: apply_filterbank")
}

/// Mel power spectrogram, `[frame][mel band]`, with fmin = 0 and
/// fmax = Nyquist (librosa's defaults).
pub fn mel_spectrogram(x: &[f64], sample_rate: u32, cfg: &StftConfig, n_mels: usize) -> Vec<Vec<f64>> {
    todo!("l12: mel_spectrogram")
}

/// DCT-II with orthonormal scaling (scipy's `norm="ortho"`), first `n_out`
/// coefficients:
///     X[k] = s_k · Σ_n x[n] · cos(π·k·(2n + 1) / (2N))
/// with s_0 = √(1/N) and s_k = √(2/N) for k > 0.
pub fn dct2(x: &[f64], n_out: usize) -> Vec<f64> {
    todo!("l12: dct2")
}

/// MFCCs, `[frame][coefficient]`: mel spectrogram, then 10·log10 (floor the
/// power at 1e-10 first), then the DCT of each frame, keeping `n_mfcc`.
pub fn mfcc(x: &[f64], sample_rate: u32, cfg: &StftConfig, n_mels: usize, n_mfcc: usize) -> Vec<Vec<f64>> {
    todo!("l12: mfcc")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::sine;
    use crate::synth::Rng;
    use crate::testutil::*;

    #[test]
    fn l12_mel_scale() {
        assert_close(hz_to_mel(0.0), 0.0, 1e-12);
        assert_close(hz_to_mel(700.0), 2595.0 * 2f64.log10(), 1e-9);
        assert_close(hz_to_mel(1_000.0), 999.99, 0.05); // ~1000 mel at 1 kHz, by design
        assert_close(mel_to_hz(hz_to_mel(1_234.5)), 1_234.5, 1e-9);
    }

    #[test]
    fn l12_filterbank_shape() {
        let fb = mel_filterbank(40, 512, 16_000, 0.0, 8_000.0);
        assert_eq!(fb.len(), 40);
        assert!(fb.iter().all(|row| row.len() == 257));
        assert!(fb.iter().flatten().all(|&w| (0.0..=1.0).contains(&w)));
    }

    #[test]
    fn l12_filters_climb_in_frequency() {
        let fb = mel_filterbank(40, 512, 16_000, 0.0, 8_000.0);
        let peaks: Vec<usize> = fb
            .iter()
            .map(|row| (0..row.len()).max_by(|&a, &b| row[a].total_cmp(&row[b])).unwrap())
            .collect();
        assert!(peaks.windows(2).all(|w| w[0] <= w[1]), "peak bins: {peaks:?}");
        // Mel spacing: the top filter is much wider than the bottom one.
        let width = |row: &Vec<f64>| row.iter().filter(|&&w| w > 0.0).count();
        assert!(width(&fb[39]) > 5 * width(&fb[1]));
    }

    #[test]
    fn l12_triangles_overlap_to_one() {
        // Between the first and last filter centres, each bin is covered by
        // exactly two triangles whose weights add up to 1.
        let (n_mels, n_fft, sr) = (20, 1024, 16_000);
        let fb = mel_filterbank(n_mels, n_fft, sr, 0.0, 8_000.0);
        let step = hz_to_mel(8_000.0) / (n_mels + 1) as f64;
        let (first, last) = (mel_to_hz(step), mel_to_hz(step * n_mels as f64));
        for (bin, f) in bin_frequencies(n_fft, sr).into_iter().enumerate() {
            if f >= first && f <= last {
                let total: f64 = fb.iter().map(|row| row[bin]).sum();
                assert_close(total, 1.0, 1e-9);
            }
        }
    }

    #[test]
    fn l12_apply_filterbank() {
        let out = apply_filterbank(&[vec![2.0, 4.0]], &[vec![1.0, 0.0], vec![0.5, 0.5]]);
        assert_eq!(out, vec![vec![2.0, 3.0]]);
    }

    #[test]
    fn l12_dct_known_values() {
        // scipy.fft.dct([1, 2, 3, 4], norm="ortho")
        assert_all_close(&dct2(&[1.0, 2.0, 3.0, 4.0], 4), &[5.0, -2.230_442_497, 0.0, -0.158_512_668], 1e-8);
        assert_all_close(&dct2(&[1.0; 8], 3), &[8f64.sqrt(), 0.0, 0.0], 1e-12);
    }

    #[test]
    fn l12_dct_preserves_energy() {
        let mut rng = Rng::new(3);
        let x: Vec<f64> = (0..16).map(|_| rng.gaussian()).collect();
        let energy = |v: &[f64]| v.iter().map(|a| a * a).sum::<f64>();
        assert_close(energy(&dct2(&x, 16)), energy(&x), 1e-9);
    }

    #[test]
    fn l12_tone_lands_in_its_mel_band() {
        let (sr, n_mels) = (16_000, 40);
        let cfg = StftConfig { n_fft: 512, hop_length: 160, center: true };
        let mel = mel_spectrogram(&sine(1_000.0, 0.5, sr, 0.5), sr, &cfg, n_mels);
        let frame = &mel[mel.len() / 2];
        let loudest = (0..n_mels).max_by(|&a, &b| frame[a].total_cmp(&frame[b])).unwrap();
        // Which band is centred nearest 1 kHz?
        let step = hz_to_mel(8_000.0) / (n_mels + 1) as f64;
        let nearest = (0..n_mels)
            .min_by(|&a, &b| {
                let da = (mel_to_hz(step * (a + 1) as f64) - 1_000.0).abs();
                let db = (mel_to_hz(step * (b + 1) as f64) - 1_000.0).abs();
                da.total_cmp(&db)
            })
            .unwrap();
        assert!(loudest.abs_diff(nearest) <= 1, "loudest band {loudest}, expected about {nearest}");
    }

    #[test]
    fn l12_mfcc_shape() {
        let cfg = StftConfig { n_fft: 512, hop_length: 160, center: true };
        let m = mfcc(&sine(300.0, 0.5, 16_000, 1.0), 16_000, &cfg, 40, 13);
        assert_eq!(m.len(), 1 + 16_000 / 160);
        assert!(m.iter().all(|frame| frame.len() == 13));
    }
}

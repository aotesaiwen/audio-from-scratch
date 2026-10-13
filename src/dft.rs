//! Lesson 08 (part 2): The discrete Fourier transform. See `lessons/08-dft/README.md`.
//!
//! Check with `cargo test l08`, then run `cargo run --example lesson08`.

use crate::complex::Complex;
use std::f64::consts::PI;

/// Real samples to complex numbers with zero imaginary part.
pub fn to_complex(x: &[f64]) -> Vec<Complex> {
    todo!("l08: to_complex")
}

/// The DFT, straight from the definition:
///     X[k] = Σ_n x[n] · e^(−j·2π·k·n/N)    for k = 0 .. N−1
/// Each X[k] measures how much of the frequency "k cycles per N samples"
/// the signal contains. Two nested loops, so N² multiplications.
pub fn dft(x: &[Complex]) -> Vec<Complex> {
    // Tip: the angle only matters modulo 2π, so use `(k * n) % big_n` before
    // converting to f64. That keeps the angle small and the result accurate.
    todo!("l08: dft")
}

/// The inverse DFT: x[n] = (1/N) · Σ_k X[k] · e^(+j·2π·k·n/N).
/// Same as the DFT with the sign of the angle flipped, then divided by N.
pub fn idft(spectrum: &[Complex]) -> Vec<Complex> {
    todo!("l08: idft")
}

/// The frequency in Hz of DFT bin `k`, for an `n`-point DFT.
pub fn bin_frequency(k: usize, n: usize, sample_rate: u32) -> f64 {
    todo!("l08: bin_frequency")
}

/// |X[k]| for k = 0 ..= N/2: the magnitude spectrum of a real signal. The
/// upper half of the DFT mirrors the lower half for real input, so it adds
/// no information.
pub fn magnitude_spectrum(x: &[f64]) -> Vec<f64> {
    todo!("l08: magnitude_spectrum")
}

/// The frequency of the strongest component, ignoring bin 0 (DC).
pub fn peak_frequency(x: &[f64], sample_rate: u32) -> f64 {
    // To find the index of the largest value, track (best_index, best_value)
    // in a loop, or look up `Iterator::max_by` and `f64::total_cmp`.
    todo!("l08: peak_frequency")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::sine;
    use crate::testutil::*;

    #[test]
    fn l08_dft_of_impulse_is_flat() {
        let x = to_complex(&[1.0, 0.0, 0.0, 0.0]);
        for c in dft(&x) {
            assert_close(c.re, 1.0, 1e-12);
            assert_close(c.im, 0.0, 1e-12);
        }
    }

    #[test]
    fn l08_dft_of_constant_is_all_dc() {
        let spec = dft(&to_complex(&[1.0; 8]));
        assert_close(spec[0].re, 8.0, 1e-9);
        for c in &spec[1..] {
            assert_close(c.abs(), 0.0, 1e-9);
        }
    }

    #[test]
    fn l08_dft_finds_a_cosine() {
        // A cosine making exactly 3 cycles in 32 samples lands in bins 3 and 29.
        let x: Vec<f64> = (0..32).map(|n| (2.0 * PI * 3.0 * n as f64 / 32.0).cos()).collect();
        let spec = dft(&to_complex(&x));
        for (k, c) in spec.iter().enumerate() {
            let expected = if k == 3 || k == 29 { 16.0 } else { 0.0 };
            assert_close(c.abs(), expected, 1e-9);
        }
    }

    #[test]
    fn l08_idft_undoes_dft() {
        let x = to_complex(&[0.3, -1.0, 2.5, 0.0, 0.7]);
        let back = idft(&dft(&x));
        for (a, b) in back.iter().zip(&x) {
            assert_close(a.re, b.re, 1e-9);
            assert_close(a.im, b.im, 1e-9);
        }
    }

    #[test]
    fn l08_bin_frequency() {
        assert_close(bin_frequency(100, 800, 8_000), 1_000.0, 1e-9);
        assert_close(bin_frequency(1, 1024, 16_000), 15.625, 1e-9);
    }

    #[test]
    fn l08_magnitude_spectrum_length() {
        assert_eq!(magnitude_spectrum(&[0.0; 16]).len(), 9);
        assert_eq!(magnitude_spectrum(&[0.0; 15]).len(), 8);
    }

    #[test]
    fn l08_peak_frequency() {
        // 0.5 s at 8 kHz: 4000 samples, so bins are 2 Hz apart.
        let x: Vec<f64> = sine(440.0, 1.0, 8_000, 0.5)
            .iter()
            .zip(sine(1_000.0, 0.3, 8_000, 0.5))
            .map(|(a, b)| a + b + 0.5) // + a DC offset, which must be ignored
            .collect();
        assert_close(peak_frequency(&x, 8_000), 440.0, 1e-9);
    }
}

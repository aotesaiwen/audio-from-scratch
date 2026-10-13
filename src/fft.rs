//! Lesson 09: The FFT. See `lessons/09-fft/README.md`.
//!
//! Same answer as `dft`, but in N·log2(N) steps instead of N².
//! Check with `cargo test l09`, then run `cargo run --example lesson09`.

use crate::complex::Complex;
use std::f64::consts::PI;

/// Radix-2 Cooley–Tukey FFT. The length must be a power of two.
///
/// Split x into its even- and odd-indexed samples, FFT each half
/// (recursively), then combine. For k in 0 .. N/2, with
/// W = e^(−j·2π·k/N):
///     X[k]       = E[k] + W·O[k]
///     X[k + N/2] = E[k] − W·O[k]
/// The recursion stops at N = 1, where the FFT of one sample is itself.
pub fn fft(x: &[Complex]) -> Vec<Complex> {
    let n = x.len();
    assert!(n.is_power_of_two(), "fft length must be a power of two, got {n}");
    if n == 1 {
        return vec![x[0]];
    }
    let even: Vec<Complex> = x.iter().step_by(2).copied().collect();
    let odd: Vec<Complex> = x.iter().skip(1).step_by(2).copied().collect();
    let (e, o) = (fft(&even), fft(&odd));
    let mut out = vec![Complex::ZERO; n];
    for k in 0..n / 2 {
        let t = Complex::expj(-2.0 * PI * k as f64 / n as f64) * o[k];
        out[k] = e[k] + t;
        out[k + n / 2] = e[k] - t;
    }
    out
}

/// Inverse FFT, by the conjugation trick: ifft(X) = conj(fft(conj(X))) / N.
/// No new algorithm needed.
pub fn ifft(spectrum: &[Complex]) -> Vec<Complex> {
    let scale = 1.0 / spectrum.len() as f64;
    let conjugated: Vec<Complex> = spectrum.iter().map(|c| c.conj()).collect();
    fft(&conjugated).iter().map(|c| c.conj() * scale).collect()
}

/// FFT of a real signal, padded with zeros (or cut) to `n_fft` samples,
/// returning only bins 0 ..= n_fft/2. This is what `np.fft.rfft` does, and
/// what the STFT will call for every frame.
pub fn rfft(x: &[f64], n_fft: usize) -> Vec<Complex> {
    let mut buf = vec![Complex::ZERO; n_fft];
    for (b, &v) in buf.iter_mut().zip(x) {
        *b = Complex::new(v, 0.0);
    }
    let mut spec = fft(&buf);
    spec.truncate(n_fft / 2 + 1);
    spec
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dft::{dft, to_complex};
    use crate::synth::Rng;
    use crate::testutil::*;

    fn random_signal(n: usize, seed: u64) -> Vec<Complex> {
        let mut rng = Rng::new(seed);
        (0..n).map(|_| Complex::new(rng.bipolar(), rng.bipolar())).collect()
    }

    #[test]
    fn l09_fft_of_one_sample() {
        assert_eq!(fft(&[Complex::new(0.5, -1.0)]), vec![Complex::new(0.5, -1.0)]);
    }

    #[test]
    fn l09_fft_matches_dft() {
        for n in [2, 4, 8, 64, 256] {
            let x = random_signal(n, n as u64);
            for (a, b) in fft(&x).iter().zip(dft(&x)) {
                assert_close(a.re, b.re, 1e-9);
                assert_close(a.im, b.im, 1e-9);
            }
        }
    }

    #[test]
    fn l09_ifft_undoes_fft() {
        let x = random_signal(1024, 7);
        for (a, b) in ifft(&fft(&x)).iter().zip(&x) {
            assert_close(a.re, b.re, 1e-12);
            assert_close(a.im, b.im, 1e-12);
        }
    }

    #[test]
    #[should_panic(expected = "power of two")]
    fn l09_rejects_other_lengths() {
        fft(&to_complex(&[1.0, 2.0, 3.0]));
    }

    #[test]
    fn l09_rfft_pads_and_truncates() {
        let spec = rfft(&[1.0], 8); // an impulse, zero-padded to 8
        assert_eq!(spec.len(), 5);
        for c in &spec {
            assert_close(c.re, 1.0, 1e-12);
            assert_close(c.im, 0.0, 1e-12);
        }
        let long: Vec<f64> = (0..10).map(|i| i as f64).collect();
        let cut = rfft(&long, 8); // only the first 8 samples are used
        assert_close(cut[0].re, 28.0, 1e-9); // 0+1+...+7
    }
}

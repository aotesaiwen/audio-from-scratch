//! Lesson 16 (bonus): LPC and McAdams voice anonymisation.
//! See `lessons/16-lpc-mcadams/README.md`.
//!
//! The source–filter model of speech, and McAdams voice anonymisation,
//! built from parts you made earlier: autocorrelation, FIR filtering,
//! complex numbers.
//! Check with `cargo test l16`, then run `cargo run --example lesson16`.

use crate::complex::Complex;
use crate::fir::fir_filter;
use crate::frames::num_frames;
use crate::level::peak;
use crate::pitch::autocorrelation;
use crate::window::hann;
use std::f64::consts::PI;

/// Solve for prediction coefficients from autocorrelation values
/// r[0..=order] with the Levinson–Durbin recursion.
///
/// Returns (a, error) with a[0] = 1, such that the prediction error
/// e[n] = Σ_j a[j] · x[n − j] is as small as possible. `error` is the
/// energy left in e. If r[0] is 0 (silence), return a = [1, 0, 0, ...] and 0.
///
/// The recursion (i = 1 ..= order), starting from err = r[0]:
///     acc = r[i] + Σ_{j=1}^{i−1} a[j] · r[i − j]
///     k   = −acc / err                       (the "reflection coefficient")
///     a[j] += k · a_old[i − j]   for j in 1..i (use a copy of a from before this step)
///     a[i] = k
///     err  = err · (1 − k²)
pub fn levinson_durbin(r: &[f64], order: usize) -> (Vec<f64>, f64) {
    let mut a = vec![0.0; order + 1];
    a[0] = 1.0;
    let mut err = r[0];
    if err <= 0.0 {
        return (a, 0.0);
    }
    for i in 1..=order {
        let mut acc = r[i];
        for j in 1..i {
            acc += a[j] * r[i - j];
        }
        let k = -acc / err;
        let old = a.clone();
        for j in 1..i {
            a[j] = old[j] + k * old[i - j];
        }
        a[i] = k;
        err *= 1.0 - k * k;
        if err <= 0.0 {
            break;
        }
    }
    (a, err)
}

/// LPC coefficients of a frame (autocorrelation method).
pub fn lpc(frame: &[f64], order: usize) -> Vec<f64> {
    levinson_durbin(&autocorrelation(frame, order), order).0
}

/// The prediction error (the "residual", or excitation): filter x through
/// A(z), an FIR filter whose taps are the LPC coefficients themselves.
pub fn residual(x: &[f64], a: &[f64]) -> Vec<f64> {
    fir_filter(x, a)
}

/// Undo `residual`: run e through the all-pole filter 1/A(z),
///     y[n] = e[n] − Σ_{j=1}^{p} a[j] · y[n − j]
/// (skipping terms with n − j < 0).
pub fn synthesize(e: &[f64], a: &[f64]) -> Vec<f64> {
    let mut y = vec![0.0; e.len()];
    for n in 0..e.len() {
        let mut v = e[n];
        for j in 1..a.len().min(n + 1) {
            v -= a[j] * y[n - j];
        }
        y[n] = v;
    }
    y
}

/// Given: evaluate the polynomial z^p + a[1]·z^(p−1) + ... + a[p] at z
/// (Horner's method). Its roots are the poles of 1/A(z).
pub fn eval_poly(a: &[f64], z: Complex) -> Complex {
    let mut v = Complex::ZERO;
    for &c in a {
        v = v * z + Complex::new(c, 0.0);
    }
    v
}

/// All p roots of z^p + a[1]·z^(p−1) + ... + a[p] (a[0] must be 1), by the
/// Durand–Kerner method: guess p different starting points, then
/// repeatedly move each guess z_i to
///     z_i − P(z_i) / Π_{j≠i} (z_i − z_j)
/// until nothing moves more than 1e-12 (or 500 rounds have passed).
/// Start from z_i = (0.4 + 0.9j)^i, i = 0 .. p−1.
pub fn poly_roots(a: &[f64]) -> Vec<Complex> {
    let p = a.len() - 1;
    let seed = Complex::new(0.4, 0.9);
    let mut roots = Vec::with_capacity(p);
    let mut z = Complex::new(1.0, 0.0);
    for _ in 0..p {
        roots.push(z);
        z = z * seed;
    }
    for _ in 0..500 {
        let mut biggest_step: f64 = 0.0;
        for i in 0..p {
            let mut denom = Complex::new(1.0, 0.0);
            for j in 0..p {
                if j != i {
                    denom = denom * (roots[i] - roots[j]);
                }
            }
            let step = eval_poly(a, roots[i]) / denom;
            roots[i] = roots[i] - step;
            biggest_step = biggest_step.max(step.abs());
        }
        if biggest_step < 1e-12 {
            break;
        }
    }
    roots
}

/// Multiply out Π (z − root) and return the real parts of the coefficients,
/// highest power first (so a[0] = 1). Inverse of `poly_roots` for real
/// polynomials, whose complex roots come in conjugate pairs.
pub fn poly_from_roots(roots: &[Complex]) -> Vec<f64> {
    let mut c = vec![Complex::new(1.0, 0.0)];
    for &r in roots {
        let mut next = vec![Complex::ZERO; c.len() + 1];
        for (k, &ck) in c.iter().enumerate() {
            next[k] += ck;
            next[k + 1] = next[k + 1] - r * ck;
        }
        c = next;
    }
    c.iter().map(|v| v.re).collect()
}

/// The McAdams transformation: move every complex pole's angle φ (its
/// resonance frequency) to φ^alpha, keeping its radius (its bandwidth).
/// Poles with |imag| ≤ 1e-9 count as real and stay put. Clamp new angles to
/// at most π. For a pole below the real axis, φ is negative: use −(|φ|^alpha),
/// so conjugate pairs stay conjugate. alpha = 1 changes nothing.
pub fn mcadams(a: &[f64], alpha: f64) -> Vec<f64> {
    let moved: Vec<Complex> = poly_roots(a)
        .into_iter()
        .map(|z| {
            if z.im.abs() <= 1e-9 {
                return z;
            }
            let phi = z.arg();
            let new_phi = phi.abs().powf(alpha).min(PI) * phi.signum();
            Complex::from_polar(z.abs(), new_phi)
        })
        .collect();
    poly_from_roots(&moved)
}

/// Anonymise speech with the standard McAdams recipe:
/// - frames of 20 ms every 10 ms, LPC order 20
/// - analysis/synthesis window: w = √(hann(frame_len) / K), where
///   K = Σ hann / hop (so the overlapped windows add back to 1)
/// - for each frame: windowed = frame · w; a = lpc(windowed); e = residual;
///   a' = mcadams(a, alpha); out = synthesize(e, a') · w; overlap-add out
///   into the output at the frame's position
/// - finally scale the whole output so its peak is 1 (if it isn't silent).
pub fn mcadams_anonymize(x: &[f64], sample_rate: u32, alpha: f64) -> Vec<f64> {
    let sr = sample_rate as f64;
    let frame_len = (0.02 * sr) as usize;
    let hop = (0.01 * sr) as usize;
    let h = hann(frame_len);
    let k = h.iter().sum::<f64>() / hop as f64;
    let w: Vec<f64> = h.iter().map(|v| (v / k).sqrt()).collect();
    let mut out = vec![0.0; x.len()];
    for f in 0..num_frames(x.len(), frame_len, hop) {
        let start = f * hop;
        let frame: Vec<f64> = x[start..start + frame_len].iter().zip(&w).map(|(s, wv)| s * wv).collect();
        let a = lpc(&frame, 20);
        let e = residual(&frame, &a);
        let y = synthesize(&e, &mcadams(&a, alpha));
        for (i, (v, wv)) in y.iter().zip(&w).enumerate() {
            out[start + i] += v * wv;
        }
    }
    let p = peak(&out);
    if p > 0.0 {
        for v in out.iter_mut() {
            *v /= p;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{voice, Rng, VoiceParams};
    use crate::testutil::*;

    #[test]
    fn l16_levinson_recovers_an_ar2_process() {
        // x[n] = 1.3·x[n−1] − 0.6·x[n−2] + noise, i.e. A(z) = 1 − 1.3z⁻¹ + 0.6z⁻²
        let mut rng = Rng::new(5);
        let mut x = vec![0.0; 20_000];
        for n in 2..x.len() {
            x[n] = 1.3 * x[n - 1] - 0.6 * x[n - 2] + rng.gaussian();
        }
        let a = lpc(&x, 2);
        assert_all_close(&a, &[1.0, -1.3, 0.6], 0.03);
    }

    #[test]
    fn l16_levinson_on_silence() {
        assert_eq!(levinson_durbin(&[0.0, 0.0, 0.0], 2), (vec![1.0, 0.0, 0.0], 0.0));
    }

    #[test]
    fn l16_residual_then_synthesize_is_identity() {
        let mut rng = Rng::new(9);
        let x: Vec<f64> = (0..500).map(|_| rng.gaussian()).collect();
        let a = [1.0, -0.9, 0.2];
        assert_all_close(&synthesize(&residual(&x, &a), &a), &x, 1e-9);
    }

    #[test]
    fn l16_poly_roots_known() {
        // (z − 0.5)(z + 0.25) = z² − 0.25z − 0.125
        let mut roots = poly_roots(&[1.0, -0.25, -0.125]);
        roots.sort_by(|p, q| p.re.total_cmp(&q.re));
        assert_close(roots[0].re, -0.25, 1e-9);
        assert_close(roots[1].re, 0.5, 1e-9);
        // z² − 2r·cos(θ)z + r² has roots r·e^(±jθ)
        let (r, theta) = (0.9f64, 0.5f64);
        for z in poly_roots(&[1.0, -2.0 * r * theta.cos(), r * r]) {
            assert_close(z.abs(), r, 1e-9);
            assert_close(z.arg().abs(), theta, 1e-9);
        }
    }

    #[test]
    fn l16_roots_round_trip() {
        let v = voice(&VoiceParams { f0: 120.0, seconds: 0.1, sample_rate: 16_000, snr_db: Some(30.0), ..Default::default() });
        let frame: Vec<f64> = v.samples[400..720].iter().zip(hann(320)).map(|(s, w)| s * w).collect();
        let a = lpc(&frame, 16);
        assert_all_close(&poly_from_roots(&poly_roots(&a)), &a, 1e-6);
    }

    #[test]
    fn l16_mcadams() {
        let a = [1.0, -2.0 * 0.9 * 0.5f64.cos(), 0.81];
        assert_all_close(&mcadams(&a, 1.0), &a, 1e-9);
        let moved = mcadams(&a, 0.8);
        for z in poly_roots(&moved) {
            assert_close(z.abs(), 0.9, 1e-9);
            assert_close(z.arg().abs(), 0.5f64.powf(0.8), 1e-9);
        }
    }

    #[test]
    fn l16_anonymize() {
        let v = voice(&VoiceParams { f0: 120.0, seconds: 0.3, sample_rate: 16_000, snr_db: Some(30.0), ..Default::default() });
        let y = mcadams_anonymize(&v.samples, 16_000, 0.8);
        assert_eq!(y.len(), v.samples.len());
        assert!(y.iter().all(|s| s.is_finite()));
        assert_close(peak(&y), 1.0, 1e-12);
    }
}

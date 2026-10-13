//! Lesson 03: Loudness, decibels and bit depth. See `lessons/03-loudness/README.md`.
//!
//! This lesson introduces iterators and closures. Try writing each function
//! as an iterator chain rather than a `for` loop.
//! Check with `cargo test l03`, then run `cargo run --example lesson03`.

/// The largest absolute value in the signal. Empty signal: 0.0.
///
/// Worked example, given: an iterator chain. Read it left to right:
/// "walk the values, take each one's absolute value, then fold them into a
/// single number by keeping the bigger of (best so far, this one), starting
/// from 0.0". `|v| v.abs()` is a closure, a small unnamed function.
pub fn peak(x: &[f64]) -> f64 {
    x.iter().map(|v| v.abs()).fold(0.0, f64::max)
}

/// Root-mean-square: the square root of the mean of the squared samples.
/// It's the "average size" of a signal, and tracks perceived loudness far
/// better than the peak. Empty signal: 0.0.
pub fn rms(x: &[f64]) -> f64 {
    // `.sum::<f64>()` adds up an iterator. The `::<f64>` tells Rust what type
    // the total should be.
    todo!("l03: rms")
}

/// An amplitude ratio in decibels: 20 · log10(a).
/// 1.0 is 0 dB, 0.5 is about −6 dB, 0.1 is −20 dB.
/// log10(0) is −infinity, so floor the input at 1e-10 (−200 dB) first.
pub fn amplitude_to_db(a: f64) -> f64 {
    todo!("l03: amplitude_to_db")
}

/// The inverse of `amplitude_to_db`: 10^(db / 20).
pub fn db_to_amplitude(db: f64) -> f64 {
    todo!("l03: db_to_amplitude")
}

/// Scale the signal so its peak sits at `target_db` dBFS (0 dB = full scale).
/// A silent signal is returned unchanged; don't divide by zero.
pub fn normalize_peak(x: &[f64], target_db: f64) -> Vec<f64> {
    // `.collect()` turns an iterator back into a Vec.
    todo!("l03: normalize_peak")
}

/// Simulate storing the signal with `bits` bits per sample.
///
/// A `bits`-bit signed integer has 2^(bits−1) levels on each side of zero.
/// Scale by that, round to a whole number, clamp to the representable range
/// [−2^(bits−1), 2^(bits−1) − 1], then scale back down.
pub fn quantize(x: &[f64], bits: u32) -> Vec<f64> {
    // `2f64.powi(n)` raises 2.0 to an integer power n (an i32).
    todo!("l03: quantize")
}

/// Signal-to-noise ratio in dB, treating `clean − degraded` as the noise:
/// 10 · log10( Σ clean² / Σ (clean − degraded)² ).
/// (10, not 20, because these are powers, i.e. squared amplitudes.)
pub fn snr_db(clean: &[f64], degraded: &[f64]) -> f64 {
    // `.zip()` walks two iterators side by side, giving you pairs.
    todo!("l03: snr_db")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::sine;
    use crate::testutil::*;

    #[test]
    fn l03_peak() {
        assert_close(peak(&[0.1, -0.5, 0.3]), 0.5, 1e-12);
        assert_close(peak(&[]), 0.0, 1e-12);
    }

    #[test]
    fn l03_rms() {
        assert_close(rms(&[1.0, -1.0, 1.0, -1.0]), 1.0, 1e-12);
        assert_close(rms(&[3.0, 4.0]), (12.5f64).sqrt(), 1e-12);
        assert_close(rms(&[]), 0.0, 1e-12);
        // A sine's RMS is its amplitude divided by √2.
        assert_close(rms(&sine(100.0, 0.8, 8_000, 1.0)), 0.8 / 2f64.sqrt(), 1e-6);
    }

    #[test]
    fn l03_decibels() {
        assert_close(amplitude_to_db(1.0), 0.0, 1e-12);
        assert_close(amplitude_to_db(10.0), 20.0, 1e-12);
        assert_close(amplitude_to_db(0.5), -6.0206, 1e-4);
        assert_close(amplitude_to_db(0.0), -200.0, 1e-9);
        assert_close(db_to_amplitude(-20.0), 0.1, 1e-12);
        for db in [-60.0, -6.0, 0.0, 3.0] {
            assert_close(amplitude_to_db(db_to_amplitude(db)), db, 1e-9);
        }
    }

    #[test]
    fn l03_normalize_peak() {
        let y = normalize_peak(&[0.1, -0.2, 0.05], -6.0);
        assert_close(peak(&y), db_to_amplitude(-6.0), 1e-12);
        assert_close(y[0] / y[1], -0.5, 1e-12); // shape unchanged
        assert_eq!(normalize_peak(&[0.0, 0.0], -1.0), vec![0.0, 0.0]);
    }

    #[test]
    fn l03_quantize() {
        // 2 bits: levels at -1, -0.5, 0, 0.5
        assert_all_close(&quantize(&[0.3, -0.3, 0.9, -1.0, 0.1], 2), &[0.5, -0.5, 0.5, -1.0, 0.0], 1e-12);
        // 16 bits barely changes anything
        let x = sine(997.0, 0.9, 44_100, 0.1);
        assert_all_close(&quantize(&x, 16), &x, 1.0 / 32768.0);
    }

    #[test]
    fn l03_snr() {
        assert_close(snr_db(&[1.0, 1.0], &[1.1, 0.9]), 20.0, 1e-9);
    }

    #[test]
    fn l03_six_db_per_bit() {
        // The rule of thumb: each extra bit buys ~6.02 dB of SNR. For a
        // full-scale sine, SNR ≈ 6.02·bits + 1.76 dB.
        let x = sine(997.0, 0.999, 44_100, 1.0);
        let snr8 = snr_db(&x, &quantize(&x, 8));
        let snr12 = snr_db(&x, &quantize(&x, 12));
        assert_close(snr8, 6.02 * 8.0 + 1.76, 1.5);
        assert_close(snr12, 6.02 * 12.0 + 1.76, 1.5);
        assert_close(snr12 - snr8, 4.0 * 6.02, 1.5);
    }
}

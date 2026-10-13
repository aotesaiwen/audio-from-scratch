//! Lesson 07: Feedback, IIR filters. See `lessons/07-iir-filters/README.md`.
//!
//! This lesson is about structs that remember things (`&mut self`) and
//! traits: one `Filter` interface, many filters behind it.
//! Check with `cargo test l07`, then run `cargo run --example lesson07`.

use std::f64::consts::PI;

/// Anything that turns input samples into output samples, one at a time,
/// remembering whatever it needs between calls.
pub trait Filter {
    /// Feed in one sample, get one sample out.
    fn process_sample(&mut self, x: f64) -> f64;

    /// Forget all remembered state, as if freshly created.
    fn reset(&mut self);

    /// Filter a whole signal. A *default method*: every Filter gets it for
    /// free, built on top of `process_sample`.
    fn process(&mut self, x: &[f64]) -> Vec<f64> {
        todo!("l07: Filter::process")
    }
}

/// The simplest IIR filter: y[n] = (1 − a)·x[n] + a·y[n−1].
/// Each output is mostly the previous output, nudged toward the input.
pub struct OnePoleLowpass {
    a: f64,
    y1: f64, // the previous output, y[n−1]
}

impl OnePoleLowpass {
    /// `a = exp(−2π · cutoff / sample_rate)`. That puts the −3 dB point at
    /// `cutoff_hz` (approximately; it's exact for cutoffs far below Nyquist).
    pub fn new(cutoff_hz: f64, sample_rate: u32) -> Self {
        // Build the struct with `OnePoleLowpass { a: ..., y1: 0.0 }`.
        todo!("l07: OnePoleLowpass::new")
    }
}

impl Filter for OnePoleLowpass {
    fn process_sample(&mut self, x: f64) -> f64 {
        todo!("l07: OnePoleLowpass::process_sample")
    }

    fn reset(&mut self) {
        todo!("l07: OnePoleLowpass::reset")
    }
}

/// Removes DC offset (a constant shift away from zero) and very low rumble:
/// y[n] = x[n] − x[n−1] + r·y[n−1], with r just below 1 (e.g. 0.995).
pub struct DcBlocker {
    r: f64,
    x1: f64,
    y1: f64,
}

impl DcBlocker {
    pub fn new(r: f64) -> Self {
        todo!("l07: DcBlocker::new")
    }
}

impl Filter for DcBlocker {
    fn process_sample(&mut self, x: f64) -> f64 {
        todo!("l07: DcBlocker::process_sample")
    }

    fn reset(&mut self) {
        todo!("l07: DcBlocker::reset")
    }
}

/// The workhorse second-order filter ("biquad"):
/// y[n] = b0·x[n] + b1·x[n−1] + b2·x[n−2] − a1·y[n−1] − a2·y[n−2]
/// Coefficients come from Robert Bristow-Johnson's "Audio EQ Cookbook".
pub struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl Biquad {
    /// Given (worked example): build a Biquad from the cookbook's six raw
    /// coefficients. Dividing everything by a0 makes a0 equal 1, so it can
    /// drop out of the difference equation.
    pub fn from_coefficients(b0: f64, b1: f64, b2: f64, a0: f64, a1: f64, a2: f64) -> Self {
        Biquad {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Low-pass. `q` = 0.7071 (1/√2) gives the flattest passband (Butterworth).
    pub fn lowpass(cutoff_hz: f64, q: f64, sample_rate: u32) -> Self {
        // Formulas are in the README. Compute w0, cos(w0) and alpha first.
        todo!("l07: Biquad::lowpass")
    }

    /// High-pass.
    pub fn highpass(cutoff_hz: f64, q: f64, sample_rate: u32) -> Self {
        todo!("l07: Biquad::highpass")
    }

    /// Band-pass with 0 dB gain at the centre frequency. Higher `q`, narrower band.
    pub fn bandpass(center_hz: f64, q: f64, sample_rate: u32) -> Self {
        todo!("l07: Biquad::bandpass")
    }
}

impl Filter for Biquad {
    fn process_sample(&mut self, x: f64) -> f64 {
        // Compute y, then shift the memory along: x2 ← x1, x1 ← x, and the same for y.
        todo!("l07: Biquad::process_sample")
    }

    fn reset(&mut self) {
        todo!("l07: Biquad::reset")
    }
}

/// Several filters in series: the output of each feeds the next.
///
/// `Box<dyn Filter>` means "some Filter, I don't care which type, stored on
/// the heap". That's how one Vec can hold a Biquad *and* a DcBlocker.
#[derive(Default)]
pub struct FilterChain {
    filters: Vec<Box<dyn Filter>>,
}

impl FilterChain {
    pub fn new() -> Self {
        FilterChain::default()
    }

    /// Add a filter to the end of the chain (builder style, returns self).
    pub fn then(mut self, filter: impl Filter + 'static) -> Self {
        // `Box::new(filter)` puts it on the heap.
        todo!("l07: FilterChain::then")
    }
}

impl Filter for FilterChain {
    fn process_sample(&mut self, x: f64) -> f64 {
        todo!("l07: FilterChain::process_sample")
    }

    fn reset(&mut self) {
        todo!("l07: FilterChain::reset")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fir::measure_gain_db;
    use crate::signal::sine;
    use crate::testutil::*;

    /// A trivial filter, to check the default `process` method on its own.
    struct Doubler;
    impl Filter for Doubler {
        fn process_sample(&mut self, x: f64) -> f64 {
            2.0 * x
        }
        fn reset(&mut self) {}
    }

    #[test]
    fn l07_default_process_method() {
        assert_eq!(Doubler.process(&[1.0, -0.5]), vec![2.0, -1.0]);
    }

    #[test]
    fn l07_one_pole_step_response() {
        let mut f = OnePoleLowpass::new(100.0, 16_000);
        let y = f.process(&[1.0; 4_000]);
        let a = (-2.0 * PI * 100.0 / 16_000.0).exp();
        assert_close(y[0], 1.0 - a, 1e-12);
        assert!(y.windows(2).all(|w| w[1] >= w[0]), "a step response should only rise");
        assert_close(y[3_999], 1.0, 1e-3);
    }

    #[test]
    fn l07_one_pole_cutoff() {
        let g = measure_gain_db(|x| OnePoleLowpass::new(100.0, 16_000).process(x), 100.0, 16_000);
        assert_close(g, -3.01, 0.3);
    }

    #[test]
    fn l07_dc_blocker_removes_offset() {
        let x: Vec<f64> = sine(200.0, 0.3, 16_000, 1.0).iter().map(|v| v + 0.5).collect();
        let y = DcBlocker::new(0.995).process(&x);
        let tail = &y[8_000..];
        let mean = tail.iter().sum::<f64>() / tail.len() as f64;
        assert!(mean.abs() < 0.01, "mean should be ~0, got {mean}");
    }

    #[test]
    fn l07_biquad_lowpass() {
        let lp = |x: &[f64]| Biquad::lowpass(1_000.0, 0.7071, 16_000).process(x);
        assert_close(measure_gain_db(lp, 100.0, 16_000), 0.0, 0.3);
        assert_close(measure_gain_db(lp, 1_000.0, 16_000), -3.01, 0.3);
        assert!(measure_gain_db(lp, 4_000.0, 16_000) < -20.0);
    }

    #[test]
    fn l07_biquad_highpass() {
        let hp = |x: &[f64]| Biquad::highpass(1_000.0, 0.7071, 16_000).process(x);
        assert!(measure_gain_db(hp, 100.0, 16_000) < -30.0);
        assert_close(measure_gain_db(hp, 1_000.0, 16_000), -3.01, 0.3);
        assert_close(measure_gain_db(hp, 6_000.0, 16_000), 0.0, 0.3);
    }

    #[test]
    fn l07_biquad_bandpass() {
        let bp = |x: &[f64]| Biquad::bandpass(1_000.0, 2.0, 16_000).process(x);
        assert_close(measure_gain_db(bp, 1_000.0, 16_000), 0.0, 0.3);
        assert!(measure_gain_db(bp, 250.0, 16_000) < -10.0);
        assert!(measure_gain_db(bp, 4_000.0, 16_000) < -10.0);
    }

    #[test]
    fn l07_reset_forgets() {
        let mut f = Biquad::lowpass(500.0, 0.7071, 16_000);
        f.process(&[1.0, 0.0, 0.0]);
        f.reset();
        assert_eq!(f.process(&[0.0; 5]), vec![0.0; 5]);
    }

    #[test]
    fn l07_chain() {
        let one = measure_gain_db(|x| Biquad::lowpass(1_000.0, 0.7071, 16_000).process(x), 4_000.0, 16_000);
        let two = measure_gain_db(
            |x| {
                FilterChain::new()
                    .then(Biquad::lowpass(1_000.0, 0.7071, 16_000))
                    .then(Biquad::lowpass(1_000.0, 0.7071, 16_000))
                    .process(x)
            },
            4_000.0,
            16_000,
        );
        assert_close(two, 2.0 * one, 0.5);
    }
}

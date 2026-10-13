//! Lesson 14: Jitter, shimmer and HNR. See `lessons/14-voice-quality/README.md`.
//!
//! The classic voice-quality measures, as computed by Praat calls like
//! `"Get jitter (local)", 0, 0, 0.0001, 0.02, 1.3`. By the end of this
//! lesson you'll know what every one of those numbers means.
//! Check with `cargo test l14`, then run `cargo run --example lesson14`.

use crate::frames::frames;
use crate::pitch::{mean_and_std, parabolic_peak, periodicity, pitch_track, PitchConfig};
use crate::wav::Audio;
use std::fmt;

/// One glottal cycle's peak: when it happened and how high it was.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CyclePeak {
    pub time: f64,
    pub amplitude: f64,
}

/// Index of the largest value. Panics on an empty slice.
pub fn argmax(x: &[f64]) -> usize {
    todo!("l14: argmax")
}

/// Find one peak per glottal cycle in a sustained vowel whose F0 is about
/// `f0`. (Praat calls the result a PointProcess.)
///
/// Let T = sample_rate / f0, the expected period in samples.
/// 1. First peak: argmax of the first T samples, then walk uphill
///    (i += 1 while x[i+1] > x[i]) in case it landed on a rising edge.
/// 2. Next peak: argmax within the window [prev + 0.7·T, prev + 1.3·T].
///    Stop when the window would run past the end of the signal, or when
///    the biggest value sits on the window's first or last sample. That
///    means the signal is just sloping through the window with no peak
///    inside it (the voice has ended).
/// 3. Record each peak with sub-sample precision using `parabolic_peak`:
///    time = position / sample_rate, amplitude = the interpolated value.
///
/// Returns no peaks if the signal is shorter than 2·T.
pub fn find_cycle_peaks(x: &[f64], sample_rate: u32, f0: f64) -> Vec<CyclePeak> {
    todo!("l14: find_cycle_peaks")
}

/// Time between consecutive peaks (seconds): one fewer than there are peaks.
pub fn periods(peaks: &[CyclePeak]) -> Vec<f64> {
    todo!("l14: periods")
}

/// The peak amplitude of each cycle.
pub fn amplitudes(peaks: &[CyclePeak]) -> Vec<f64> {
    todo!("l14: amplitudes")
}

/// Which periods Praat trusts. These are the three mystery numbers in
/// `"Get jitter (local)", 0, 0, 0.0001, 0.02, 1.3`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PeriodLimits {
    /// Periods shorter than this (s) are ignored. (Praat: "period floor")
    pub shortest: f64,
    /// Periods longer than this (s) are ignored: probably a voice break.
    pub longest: f64,
    /// Neighbouring periods differing by more than this factor aren't
    /// compared: probably a tracking error. (Praat: "maximum period factor")
    pub max_factor: f64,
}

impl Default for PeriodLimits {
    fn default() -> Self {
        PeriodLimits { shortest: 0.0001, longest: 0.02, max_factor: 1.3 }
    }
}

impl PeriodLimits {
    /// Given helper: is this single period within [shortest, longest]?
    pub fn is_valid(&self, t: f64) -> bool {
        t >= self.shortest && t <= self.longest
    }

    /// Given helper: can these two neighbouring periods be compared? Both
    /// must be valid, and the longer at most `max_factor` × the shorter.
    pub fn pair_ok(&self, a: f64, b: f64) -> bool {
        self.is_valid(a) && self.is_valid(b) && a.max(b) / a.min(b) <= self.max_factor
    }
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

/// Jitter (local, absolute), in seconds: the mean of |T[i] − T[i+1]| over
/// neighbouring pairs that pass `limits.pair_ok`. None if no pair passes.
pub fn jitter_local_absolute(periods: &[f64], limits: &PeriodLimits) -> Option<f64> {
    todo!("l14: jitter_local_absolute")
}

/// Jitter (local), as a fraction (×100 for Praat's %): jitter (local,
/// absolute) divided by the mean of the *valid* periods.
pub fn jitter_local(periods: &[f64], limits: &PeriodLimits) -> Option<f64> {
    todo!("l14: jitter_local")
}

/// Jitter (RAP, "relative average perturbation"): for each period with a
/// neighbour on both sides (both pairs passing `pair_ok`), the distance
/// between it and the average of the three; the mean of those distances,
/// divided by the mean valid period.
/// (Praat's "ddp" jitter is exactly 3 × RAP: a different number with the
/// same information.)
pub fn jitter_rap(periods: &[f64], limits: &PeriodLimits) -> Option<f64> {
    todo!("l14: jitter_rap")
}

/// Can two neighbouring amplitudes be compared? The bigger must be at most
/// `max_factor` × the smaller (Praat's "maximum amplitude factor", 1.6).
fn amp_pair_ok(a: f64, b: f64, max_factor: f64) -> bool {
    a > 0.0 && b > 0.0 && a.max(b) / a.min(b) <= max_factor
}

/// Shimmer (local), as a fraction: mean |A[i] − A[i+1]| over neighbouring
/// pairs passing the amplitude-factor check, divided by the mean amplitude.
pub fn shimmer_local(amps: &[f64], max_factor: f64) -> Option<f64> {
    // The private helper `amp_pair_ok` above is yours to use.
    todo!("l14: shimmer_local")
}

/// Shimmer (local, dB): mean |20·log10(A[i+1] / A[i])| over valid pairs.
pub fn shimmer_local_db(amps: &[f64], max_factor: f64) -> Option<f64> {
    todo!("l14: shimmer_local_db")
}

/// Shimmer (APQ3): RAP's idea applied to amplitudes. For each amplitude
/// with valid pairs on both sides, |A[i] − mean of A[i−1], A[i], A[i+1]|;
/// the mean of those, divided by the mean amplitude.
/// (Praat's "dda" shimmer is exactly 3 × APQ3.)
pub fn shimmer_apq3(amps: &[f64], max_factor: f64) -> Option<f64> {
    todo!("l14: shimmer_apq3")
}

/// Harmonics-to-noise ratio (dB), averaged over voiced frames.
///
/// For a frame whose best normalised autocorrelation peak is r (from
/// `periodicity`), a fraction r of its energy repeats from cycle to cycle
/// (harmonic) and 1 − r doesn't (noise), so
///     HNR = 10 · log10(r / (1 − r))        (Boersma, 1993)
/// Clamp r into [1e-9, 1 − 1e-9] first. None if no frame is voiced.
pub fn hnr_db(x: &[f64], sample_rate: u32, cfg: &PitchConfig) -> Option<f64> {
    todo!("l14: hnr_db")
}

/// A Praat-style voice report.
#[derive(Debug, Clone, PartialEq)]
pub struct VoiceReport {
    pub mean_f0: f64,
    pub std_f0: f64,
    /// Fractions: multiply by 100 for %.
    pub jitter_local: f64,
    /// Seconds.
    pub jitter_local_absolute: f64,
    pub jitter_rap: f64,
    pub shimmer_local: f64,
    /// dB.
    pub shimmer_local_db: f64,
    pub shimmer_apq3: f64,
    pub hnr_db: f64,
}

/// Lets you `println!("{report}")`. Print one measure per line, with units,
/// percentages for the fractions, and microseconds for absolute jitter.
impl fmt::Display for VoiceReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // `writeln!(f, "Mean F0:  {:.2} Hz", self.mean_f0)?;` writes one line.
        // The `?` passes any formatting error on. End with `Ok(())`.
        todo!("l14: Display for VoiceReport")
    }
}

/// Analyse a sustained vowel end to end:
/// 1. Pitch-track it; mean and std of F0 (None if under two voiced frames).
/// 2. Real voices often have their sharpest peaks pointing *down*. If the
///    most negative sample is bigger in size than the most positive one,
///    flip the signal (multiply by −1) before finding peaks.
/// 3. Find cycle peaks at the mean F0; compute periods and amplitudes.
/// 4. Jitter with `PeriodLimits::default()`, shimmer with max factor 1.6,
///    HNR with the same PitchConfig.
///
/// Any measure that comes out None makes the whole report None (`?`).
pub fn analyze(audio: &Audio, cfg: &PitchConfig) -> Option<VoiceReport> {
    todo!("l14: analyze")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::synth::{voice, VoiceParams};
    use crate::testutil::*;

    #[test]
    fn l14_argmax() {
        assert_eq!(argmax(&[0.1, 0.9, -2.0, 0.5]), 1);
    }

    #[test]
    fn l14_periods_and_amplitudes() {
        let peaks = [
            CyclePeak { time: 0.010, amplitude: 0.5 },
            CyclePeak { time: 0.017, amplitude: 0.4 },
            CyclePeak { time: 0.025, amplitude: 0.6 },
        ];
        assert_all_close(&periods(&peaks), &[0.007, 0.008], 1e-12);
        assert_all_close(&amplitudes(&peaks), &[0.5, 0.4, 0.6], 1e-12);
    }

    #[test]
    fn l14_jitter_by_hand() {
        let t = [0.010, 0.011, 0.010, 0.011];
        let lim = PeriodLimits::default();
        assert_close(jitter_local_absolute(&t, &lim).unwrap(), 0.001, 1e-12);
        assert_close(jitter_local(&t, &lim).unwrap(), 0.001 / 0.0105, 1e-12);
        let t2 = [0.010, 0.012, 0.010, 0.012];
        assert_close(jitter_rap(&t2, &lim).unwrap(), (0.004 / 3.0) / 0.011, 1e-12);
    }

    #[test]
    fn l14_jitter_respects_limits() {
        let lim = PeriodLimits::default();
        // 0.05 s is longer than 0.02 s: both pairs touching it are skipped,
        // and it's left out of the mean period too.
        let t = [0.010, 0.011, 0.05, 0.010, 0.011];
        assert_close(jitter_local(&t, &lim).unwrap(), 0.001 / 0.0105, 1e-12);
        // 0.010 vs 0.014 differ by a factor 1.4 > 1.3: not compared.
        assert_eq!(jitter_local_absolute(&[0.010, 0.014], &lim), None);
    }

    #[test]
    fn l14_shimmer_by_hand() {
        let a = [1.0, 0.8, 1.0, 0.8];
        assert_close(shimmer_local(&a, 1.6).unwrap(), 0.2 / 0.9, 1e-12);
        assert_close(shimmer_local_db(&a, 1.6).unwrap(), -20.0 * 0.8f64.log10(), 1e-12);
        assert_close(shimmer_apq3(&a, 1.6).unwrap(), (0.4 / 3.0) / 0.9, 1e-12);
        assert_eq!(shimmer_local(&[1.0, 0.5], 1.6), None); // factor 2 > 1.6
    }

    #[test]
    fn l14_finds_every_cycle() {
        let v = voice(&VoiceParams { f0: 150.0, ..Default::default() });
        let peaks = find_cycle_peaks(&v.samples, v.sample_rate, 150.0);
        assert_eq!(peaks.len(), v.pulse_times.len());
        for (p, &t) in peaks.iter().zip(&v.pulse_times) {
            assert_close(p.time, t, 2e-6);
        }
    }

    #[test]
    fn l14_recovers_known_jitter_and_shimmer() {
        let v = voice(&VoiceParams { f0: 140.0, jitter: 0.01, shimmer: 0.05, seconds: 2.0, ..Default::default() });
        let peaks = find_cycle_peaks(&v.samples, v.sample_rate, 140.0);
        let lim = PeriodLimits::default();
        let measured_j = jitter_local(&periods(&peaks), &lim).unwrap();
        let true_j = jitter_local(&v.true_periods(), &lim).unwrap();
        assert_close(measured_j, true_j, 0.001);
        assert_close(measured_j, 0.01, 0.002); // and close to what we asked for
        let measured_s = shimmer_local(&amplitudes(&peaks), 1.6).unwrap();
        let true_s = shimmer_local(&v.pulse_amplitudes, 1.6).unwrap();
        assert_close(measured_s, true_s, 0.005);
    }

    #[test]
    fn l14_hnr_tracks_noise() {
        let cfg = PitchConfig::default();
        let hnr = |snr: Option<f64>| {
            let v = voice(&VoiceParams { f0: 147.0, snr_db: snr, ..Default::default() });
            hnr_db(&v.samples, v.sample_rate, &cfg).unwrap()
        };
        assert!(hnr(None) > 30.0);
        assert_close(hnr(Some(20.0)), 20.0, 2.0);
        assert_close(hnr(Some(10.0)), 10.0, 2.0);
    }

    #[test]
    fn l14_report() {
        let healthy = voice(&VoiceParams { jitter: 0.003, shimmer: 0.02, snr_db: Some(25.0), ..Default::default() });
        let rough = voice(&VoiceParams { jitter: 0.02, shimmer: 0.10, snr_db: Some(8.0), ..Default::default() });
        let cfg = PitchConfig::default();
        let a = analyze(&healthy.audio(), &cfg).expect("healthy voice should analyse");
        let b = analyze(&rough.audio(), &cfg).expect("rough voice should analyse");
        assert_close(a.mean_f0, 150.0, 2.0);
        assert!(b.jitter_local > a.jitter_local);
        assert!(b.shimmer_local > a.shimmer_local);
        assert!(b.hnr_db < a.hnr_db);
        let text = a.to_string();
        assert!(text.contains("F0") && text.contains('%'), "report text:\n{text}");
        // Upside-down recordings must give the same jitter.
        let flipped = Audio::new(44_100, healthy.samples.iter().map(|v| -v).collect());
        let c = analyze(&flipped, &cfg).unwrap();
        assert_close(c.jitter_local, a.jitter_local, 1e-9);
    }
}

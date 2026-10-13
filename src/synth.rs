//! Test-signal generators. Given code.
//!
//! The lessons use these to make signals whose right answer is known in
//! advance (a voice with 1% jitter, noise with a fixed seed) so tests can
//! check that your analysis recovers it. That's the standard way to test
//! signal processing: synthesise something with known properties, analyse
//! it, compare.

use crate::wav::Audio;
use std::f64::consts::PI;

/// A tiny, deterministic pseudo-random number generator (xorshift64*).
/// Same seed, same numbers, every run on every machine. That's what makes
/// tests involving noise repeatable.
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng { state: seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1 }
    }

    /// The next raw 64-bit value. Shifts and XORs scramble the bits.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1).
    pub fn uniform(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in [-1, 1).
    pub fn bipolar(&mut self) -> f64 {
        2.0 * self.uniform() - 1.0
    }

    /// Standard normal (mean 0, standard deviation 1), by the Box–Muller method.
    pub fn gaussian(&mut self) -> f64 {
        let u1 = self.uniform().max(1e-300);
        let u2 = self.uniform();
        (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
    }
}

/// `n` samples of Gaussian white noise with standard deviation `std`.
pub fn white_noise(n: usize, std: f64, seed: u64) -> Vec<f64> {
    let mut rng = Rng::new(seed);
    (0..n).map(|_| std * rng.gaussian()).collect()
}

/// A sine whose frequency glides linearly from `f_start` to `f_end` Hz.
///
/// The frequency is the rate of change of the phase, so a linearly rising
/// frequency needs a phase that grows like t²:
/// phase(t) = 2π (f_start·t + (f_end − f_start)·t² / (2·seconds)).
pub fn chirp(f_start: f64, f_end: f64, amplitude: f64, sample_rate: u32, seconds: f64) -> Vec<f64> {
    let sr = sample_rate as f64;
    let n = (sr * seconds).round() as usize;
    (0..n)
        .map(|i| {
            let t = i as f64 / sr;
            let phase = 2.0 * PI * (f_start * t + 0.5 * (f_end - f_start) * t * t / seconds);
            amplitude * phase.sin()
        })
        .collect()
}

/// Ring `x` through a two-pole resonator at `freq` Hz: one formant. (The
/// same idea as lesson 07's band-pass biquad, kept here so this file doesn't
/// depend on your code.)
fn resonate(x: &[f64], freq: f64, bandwidth: f64, sample_rate: u32) -> Vec<f64> {
    let sr = sample_rate as f64;
    let r = (-PI * bandwidth / sr).exp();
    let (a1, a2) = (2.0 * r * (2.0 * PI * freq / sr).cos(), -r * r);
    let (mut y1, mut y2) = (0.0, 0.0);
    x.iter()
        .map(|&v| {
            let y = (1.0 - r) * v + a1 * y1 + a2 * y2;
            y2 = y1;
            y1 = y;
            y
        })
        .collect()
}

/// A fake utterance for examples that need "speech": silence, an "ah", an
/// "sss", a short gap, an "ee", silence. About 1.8 s. The vowels are voice
/// pulses passed through three formant resonators each. A very quiet noise
/// floor keeps the silences from being exactly zero, as in a recording.
pub fn utterance(sample_rate: u32) -> Vec<f64> {
    let sr = sample_rate as f64;
    let silence = |seconds: f64| vec![0.0; (sr * seconds) as usize];
    // (frequency, bandwidth) of the first three formants
    let vowel = |f0: f64, seconds: f64, seed: u64, formants: [(f64, f64); 3]| {
        let p = VoiceParams { f0, jitter: 0.005, shimmer: 0.03, seconds, sample_rate, seed, ..Default::default() };
        let mut x = voice(&p).samples;
        for (freq, bw) in formants {
            x = resonate(&x, freq, bw, sample_rate);
        }
        let peak = x.iter().fold(0.0f64, |m, v| m.max(v.abs())).max(1e-12);
        // 10 ms fades at both ends, so the vowel doesn't start or stop with a click.
        let ramp = (0.01 * sr) as usize;
        let n = x.len();
        (0..n)
            .map(|i| 0.5 * x[i] / peak * (i.min(n - 1 - i) as f64 / ramp as f64).min(1.0))
            .collect::<Vec<f64>>()
    };
    let ah = [(730.0, 90.0), (1_090.0, 110.0), (2_440.0, 170.0)];
    let ee = [(270.0, 60.0), (2_290.0, 100.0), (3_010.0, 170.0)];
    // "sss": white noise, differenced (each sample minus the one before) to
    // tilt its energy toward high frequencies, like a real fricative.
    let hiss: Vec<f64> = white_noise((sr * 0.3) as usize + 1, 0.05, 7).windows(2).map(|w| w[1] - w[0]).collect();

    let mut x = silence(0.25);
    x.extend(vowel(130.0, 0.45, 2, ah));
    x.extend(hiss);
    x.extend(silence(0.15));
    x.extend(vowel(110.0, 0.4, 3, ee));
    x.extend(silence(0.25));
    let noise_floor = white_noise(x.len(), 0.0005, 11);
    for (s, n) in x.iter_mut().zip(noise_floor) {
        *s += n;
    }
    x
}

/// Settings for [`voice`].
#[derive(Debug, Clone)]
pub struct VoiceParams {
    /// Fundamental frequency (Hz).
    pub f0: f64,
    /// Roughly the "jitter (local)" you'll measure, as a fraction (0.01 = 1%).
    pub jitter: f64,
    /// Roughly the "shimmer (local)" you'll measure, as a fraction.
    pub shimmer: f64,
    /// Add white noise this many dB below the voice. `None` = no noise.
    pub snr_db: Option<f64>,
    pub seconds: f64,
    pub sample_rate: u32,
    /// Change this to get a different random (but repeatable) voice.
    pub seed: u64,
}

impl Default for VoiceParams {
    fn default() -> Self {
        VoiceParams {
            f0: 150.0,
            jitter: 0.0,
            shimmer: 0.0,
            snr_db: None,
            seconds: 1.0,
            sample_rate: 44_100,
            seed: 1,
        }
    }
}

/// A synthetic voice plus the ground truth it was built from.
#[derive(Debug, Clone)]
pub struct SyntheticVoice {
    pub samples: Vec<f64>,
    pub sample_rate: u32,
    /// The exact time (s) of each glottal pulse's peak.
    pub pulse_times: Vec<f64>,
    /// The exact height of each pulse's peak.
    pub pulse_amplitudes: Vec<f64>,
}

impl SyntheticVoice {
    /// The true length of every cycle, in seconds.
    pub fn true_periods(&self) -> Vec<f64> {
        self.pulse_times.windows(2).map(|w| w[1] - w[0]).collect()
    }

    pub fn audio(&self) -> Audio {
        Audio::new(self.sample_rate, self.samples.clone())
    }
}

/// A crude sustained vowel: a train of smooth "glottal pulses", one per vocal
/// fold cycle, like the /a/ recordings in voice-disorder databases.
///
/// Each cycle's length is wobbled by up to ±1.5·`jitter` and each pulse's
/// height by up to ±1.5·`shimmer` (uniformly). For uniform wobble, the mean
/// absolute difference between neighbours is 2/3 of the maximum wobble, so
/// the measured jitter/shimmer land close to the values you ask for.
///
/// It sounds buzzy, like a reed, because real voices also pass through the
/// resonances of the throat and mouth (formants). Lesson 07's example adds those.
pub fn voice(p: &VoiceParams) -> SyntheticVoice {
    let sr = p.sample_rate as f64;
    let n = (sr * p.seconds).round() as usize;
    let mut rng = Rng::new(p.seed);
    let tau = 0.0006; // time from pulse onset to its peak (s)
    let pulse_len = 12.0 * tau; // long enough for the pulse to die away

    // First decide when each pulse starts and how tall it should end up.
    let mut onsets = Vec::new();
    let mut heights = Vec::new();
    let mut t = 0.002;
    while t + pulse_len < p.seconds {
        onsets.push(t);
        heights.push(0.4 * (1.0 + 1.5 * p.shimmer * rng.bipolar()));
        t += (1.0 + 1.5 * p.jitter * rng.bipolar()) / p.f0;
    }

    // The pulses are all positive, so removing the DC offset afterwards will
    // lower every peak by the mean. Lift them by that much in advance so the
    // final heights match `heights`. (Each pulse of height h has area h·τ·e.)
    let alpha = tau * std::f64::consts::E * sr / n.max(1) as f64;
    let lift = alpha * heights.iter().sum::<f64>() / (1.0 - alpha * heights.len() as f64);

    let mut samples = vec![0.0; n];
    for (&t, &h) in onsets.iter().zip(&heights) {
        let amp = lift + h;
        let first = (t * sr).ceil() as usize;
        let last = (((t + pulse_len) * sr).floor() as usize).min(n.saturating_sub(1));
        for (i, s) in samples.iter_mut().enumerate().take(last + 1).skip(first) {
            let u = (i as f64 / sr - t) / tau;
            *s += amp * u * (1.0 - u).exp(); // peaks at u = 1 with height `amp`
        }
    }
    let amps: Vec<f64> = heights.iter().map(|h| lift + h).collect();

    // Remove the DC offset (a microphone can't record one).
    let mean = samples.iter().sum::<f64>() / n.max(1) as f64;
    for s in samples.iter_mut() {
        *s -= mean;
    }

    if let Some(snr_db) = p.snr_db {
        let power = samples.iter().map(|v| v * v).sum::<f64>() / n.max(1) as f64;
        let std = (power / 10f64.powf(snr_db / 10.0)).sqrt();
        for s in samples.iter_mut() {
            *s += std * rng.gaussian();
        }
    }

    SyntheticVoice {
        samples,
        sample_rate: p.sample_rate,
        pulse_times: onsets.iter().map(|t| t + tau).collect(),
        pulse_amplitudes: amps.iter().map(|a| a - mean).collect(),
    }
}

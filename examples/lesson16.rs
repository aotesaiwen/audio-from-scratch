//! Lesson 16: the LPC envelope, and McAdams anonymisation.
//!     cargo run --example lesson16
//!     cargo run --example lesson16 -- my_voice.wav

use lab::fft::rfft;
use lab::iir::{Biquad, Filter};
use lab::level::amplitude_to_db;
use lab::lpc::{lpc, mcadams, mcadams_anonymize};
use lab::plot::Plot;
use lab::synth::{utterance, voice, VoiceParams};
use lab::wav::{read_wav, write_wav, Audio};
use lab::window::{apply_window, hann};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;
    let sr = 16_000;

    // A vowel with known formants (as in lesson 07).
    let buzz = voice(&VoiceParams { f0: 120.0, jitter: 0.005, seconds: 1.0, sample_rate: sr, snr_db: Some(40.0), ..Default::default() }).samples;
    let mut aah = vec![0.0; buzz.len()];
    for (formant, q, gain) in [(730.0, 6.0, 1.0), (1_090.0, 8.0, 0.6), (2_440.0, 10.0, 0.3)] {
        for (a, b) in aah.iter_mut().zip(Biquad::bandpass(formant, q, sr).process(&buzz)) {
            *a += gain * b;
        }
    }

    // Spectrum of one 32 ms frame vs the LPC envelope 1/|A|.
    let n = 512;
    let frame = apply_window(&aah[8_000..8_000 + n], &hann(n));
    let spectrum: Vec<f64> = rfft(&frame, n).iter().map(|c| amplitude_to_db(c.abs())).collect();
    let envelope = |a: &[f64]| -> Vec<f64> { rfft(a, n).iter().map(|c| -amplitude_to_db(c.abs())).collect() };
    let a = lpc(&frame, 16);
    // Shift the envelopes so they sit on top of the spectrum's peak.
    let shift = spectrum.iter().copied().fold(f64::NEG_INFINITY, f64::max) - envelope(&a).iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let lift = |v: Vec<f64>| v.iter().map(|d| d + shift).collect::<Vec<f64>>();
    let freqs: Vec<f64> = (0..=n / 2).map(|k| k as f64 * sr as f64 / n as f64).collect();
    Plot::new("A vowel frame: spectrum and LPC envelope")
        .x_label("frequency (Hz)")
        .y_label("magnitude (dB)")
        .line("spectrum", &freqs, &spectrum)
        .line("LPC envelope (order 16)", &freqs, &lift(envelope(&a)))
        .line("after McAdams, alpha 0.8", &freqs, &lift(envelope(&mcadams(&a, 0.8))))
        .save("out/lesson16_envelope.svg")?;

    // Anonymise speech.
    let audio = match std::env::args().nth(1) {
        Some(path) => read_wav(&path)?,
        None => Audio::new(sr, aah.iter().chain(&utterance(sr)).copied().collect()),
    };
    write_wav("out/lesson16_original.wav", &audio)?;
    for alpha in [0.8, 0.9] {
        let y = mcadams_anonymize(&audio.samples, audio.sample_rate, alpha);
        write_wav(format!("out/lesson16_mcadams_{alpha}.wav"), &Audio::new(audio.sample_rate, y))?;
    }
    println!("Wrote out/lesson16_envelope.svg, out/lesson16_original.wav, out/lesson16_mcadams_{{0.8,0.9}}.wav");
    Ok(())
}

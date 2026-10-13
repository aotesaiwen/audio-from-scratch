//! Lesson 07: IIR filters, and turning a buzz into a vowel.
//!     cargo run --example lesson07
//!     cargo run --example lesson07 -- my_voice.wav

use lab::fir::measure_gain_db;
use lab::iir::{Biquad, Filter, FilterChain, OnePoleLowpass};
use lab::level::normalize_peak;
use lab::plot::Plot;
use lab::synth::{utterance, voice, white_noise, VoiceParams};
use lab::wav::{read_wav, write_wav, Audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;
    let sr = 16_000;
    let save = |name: &str, sr: u32, y: &[f64]| write_wav(format!("out/lesson07_{name}.wav"), &Audio::new(sr, normalize_peak(y, -1.0)));

    // 1. Frequency responses
    let freqs: Vec<f64> = (1..160).map(|i| i as f64 * 50.0).collect();
    let curve = |make: &dyn Fn() -> Box<dyn Filter>| -> Vec<f64> {
        freqs.iter().map(|&f| measure_gain_db(|x| make().process(x), f, sr).max(-60.0)).collect()
    };
    Plot::new("IIR filters (sample rate 16 kHz)")
        .x_label("frequency (Hz)")
        .y_label("gain (dB)")
        .line("one-pole low-pass, 500 Hz", &freqs, &curve(&|| Box::new(OnePoleLowpass::new(500.0, sr))))
        .line("biquad low-pass, 500 Hz", &freqs, &curve(&|| Box::new(Biquad::lowpass(500.0, 0.7071, sr))))
        .line("biquad high-pass, 3 kHz", &freqs, &curve(&|| Box::new(Biquad::highpass(3_000.0, 0.7071, sr))))
        .line("biquad band-pass, 1 kHz, Q 4", &freqs, &curve(&|| Box::new(Biquad::bandpass(1_000.0, 4.0, sr))))
        .save("out/lesson07_response.svg")?;

    // 2. Filtered noise
    let noise = white_noise(2 * sr as usize, 0.2, 1);
    save("noise", sr, &noise)?;
    save("noise_lowpass", sr, &Biquad::lowpass(500.0, 0.7071, sr).process(&noise))?;
    save("noise_highpass", sr, &Biquad::highpass(3_000.0, 0.7071, sr).process(&noise))?;
    save("noise_bandpass", sr, &Biquad::bandpass(1_000.0, 4.0, sr).process(&noise))?;

    // 3. Source + filter = vowel. A buzzy pulse train through three
    //    resonances (formants) placed where an adult's /a/ has them.
    let buzz = voice(&VoiceParams { f0: 120.0, jitter: 0.005, shimmer: 0.03, seconds: 2.0, sample_rate: sr, ..Default::default() }).samples;
    let mut aah = vec![0.0; buzz.len()];
    for (formant, q, gain) in [(730.0, 6.0, 1.0), (1_090.0, 8.0, 0.6), (2_440.0, 10.0, 0.3)] {
        let band = Biquad::bandpass(formant, q, sr).process(&buzz);
        for (a, b) in aah.iter_mut().zip(&band) {
            *a += gain * b;
        }
    }
    save("buzz", sr, &buzz)?;
    save("vowel_aah", sr, &aah)?;

    // 4. Telephone voice: keep only 300 Hz .. 3.4 kHz.
    let audio = match std::env::args().nth(1) {
        Some(path) => read_wav(&path)?,
        None => Audio::new(sr, utterance(sr)),
    };
    let mut phone = FilterChain::new()
        .then(Biquad::highpass(300.0, 0.7071, audio.sample_rate))
        .then(Biquad::highpass(300.0, 0.7071, audio.sample_rate))
        .then(Biquad::lowpass(3_400.0, 0.7071, audio.sample_rate))
        .then(Biquad::lowpass(3_400.0, 0.7071, audio.sample_rate));
    save("original", audio.sample_rate, &audio.samples)?;
    save("telephone", audio.sample_rate, &phone.process(&audio.samples))?;

    println!("Wrote out/lesson07_response.svg and out/lesson07_*.wav");
    println!("Compare buzz.wav with vowel_aah.wav: same source, different filter.");
    Ok(())
}

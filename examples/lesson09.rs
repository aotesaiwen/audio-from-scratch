//! Lesson 09: the FFT's speed, and the spectrum of a voice.
//!     cargo run --example lesson09

use lab::complex::Complex;
use lab::dft::dft;
use lab::fft::{fft, rfft};
use lab::level::amplitude_to_db;
use lab::plot::Plot;
use lab::synth::{voice, Rng, VoiceParams};
use std::time::Instant;

fn time_ms(f: impl FnOnce()) -> f64 {
    let start = Instant::now();
    f();
    start.elapsed().as_secs_f64() * 1000.0
}

fn main() -> std::io::Result<()> {
    let mut rng = Rng::new(1);
    println!("      N        DFT (ms)     FFT (ms)");
    for p in [8, 10, 12, 14, 16, 20] {
        let n: usize = 1 << p; // 2^p
        let x: Vec<Complex> = (0..n).map(|_| Complex::new(rng.bipolar(), 0.0)).collect();
        let t_fft = time_ms(|| {
            fft(&x);
        });
        let t_dft = if n <= 4096 {
            format!("{:10.2}", time_ms(|| {
                dft(&x);
            }))
        } else {
            "  (skipped)".to_string()
        };
        println!("{n:8}   {t_dft}   {t_fft:10.3}");
    }

    // The spectrum of a synthetic voice: a comb of harmonics at multiples of F0.
    let sr = 16_000;
    let v = voice(&VoiceParams { f0: 150.0, sample_rate: sr, ..Default::default() });
    let n_fft = 8192;
    let spec = rfft(&v.samples, n_fft);
    let freqs: Vec<f64> = (0..spec.len()).map(|k| k as f64 * sr as f64 / n_fft as f64).collect();
    let db: Vec<f64> = spec.iter().map(|c| amplitude_to_db(c.abs())).collect();
    Plot::new("Spectrum of a synthetic voice, F0 = 150 Hz")
        .x_label("frequency (Hz)")
        .y_label("magnitude (dB)")
        .x_range(0.0, 3_000.0)
        .line("magnitude", &freqs, &db)
        .save("out/lesson09_voice_spectrum.svg")?;
    println!("\nWrote out/lesson09_voice_spectrum.svg. Count the peaks between 0 and 1000 Hz.");
    Ok(())
}

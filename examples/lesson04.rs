//! Lesson 04: hear aliasing.
//!     cargo run --example lesson04

use lab::aliasing::{alias_frequency, estimate_frequency, naive_square, odd_harmonics_below_nyquist, square_from_harmonics};
use lab::plot::Plot;
use lab::synth::chirp;
use lab::wav::{write_wav, Audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;

    // 1. A sine sweeping from 0 to 8000 Hz, sampled at only 8000 Hz.
    let sr = 8_000;
    let seconds = 4.0;
    let sweep = chirp(0.0, 8_000.0, 0.5, sr, seconds);
    write_wav("out/lesson04_sweep.wav", &Audio::new(sr, sweep.clone()))?;

    // Measure the frequency every 50 ms by counting zero crossings.
    let chunk = 400;
    let (mut times, mut measured, mut intended) = (Vec::new(), Vec::new(), Vec::new());
    for (i, piece) in sweep.chunks(chunk).enumerate() {
        let t = (i as f64 + 0.5) * chunk as f64 / sr as f64;
        times.push(t);
        measured.push(estimate_frequency(piece, sr));
        intended.push(8_000.0 * t / seconds);
    }
    Plot::new("A sweep to 8 kHz, sampled at 8 kHz")
        .x_label("time (s)")
        .y_label("frequency (Hz)")
        .line("frequency we asked for", &times, &intended)
        .line("frequency in the samples", &times, &measured)
        .save("out/lesson04_sweep.svg")?;

    println!("Predicted aliases at {sr} Hz:");
    for f in [1_000.0, 3_900.0, 4_100.0, 7_000.0, 8_000.0, 9_000.0] {
        println!("  {f:6.0} Hz -> {:6.0} Hz", alias_frequency(f, sr));
    }

    // 2. A high square wave (G6, 1568 Hz) at 16 kHz, two ways.
    let sr = 16_000;
    let g6 = 1_567.98;
    let n = odd_harmonics_below_nyquist(g6, sr);
    write_wav("out/lesson04_square_naive.wav", &Audio::new(sr, naive_square(g6, sr, 2.0)))?;
    let clean: Vec<f64> = square_from_harmonics(g6, sr, 2.0, n).iter().map(|v| v * 0.8).collect();
    write_wav("out/lesson04_square_bandlimited.wav", &Audio::new(sr, clean))?;
    println!("\nOnly {n} odd harmonics of {g6} Hz fit below Nyquist ({} Hz).", sr / 2);
    println!("Listen to lesson04_square_naive.wav vs lesson04_square_bandlimited.wav.");
    println!("Wrote out/lesson04_sweep.wav, out/lesson04_sweep.svg and the two square waves.");
    Ok(())
}

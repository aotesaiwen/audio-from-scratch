//! Lesson 01: see a sampled sine wave.
//!     cargo run --example lesson01
//! Then open the .svg files in out/ with a web browser.

use lab::plot::Plot;
use lab::signal::{chord, samples_per_cycle, sine, time_axis};

fn main() -> std::io::Result<()> {
    let sr = 8_000;
    let freq = 440.0;

    // The samples a computer would store: 10 ms of a 440 Hz sine at 8 kHz.
    let samples = sine(freq, 0.8, sr, 0.01);
    let t_ms: Vec<f64> = time_axis(sr, samples.len()).iter().map(|t| t * 1000.0).collect();

    // A stand-in for the "real" continuous wave: the same sine sampled so
    // densely that it looks smooth.
    let dense_sr = 400_000;
    let smooth = sine(freq, 0.8, dense_sr, 0.01);
    let smooth_t: Vec<f64> = time_axis(dense_sr, smooth.len()).iter().map(|t| t * 1000.0).collect();

    Plot::new("A 440 Hz sine, sampled at 8 kHz")
        .x_label("time (ms)")
        .y_label("amplitude")
        .line("continuous wave", &smooth_t, &smooth)
        .points("samples", &t_ms, &samples)
        .save("out/lesson01_sine.svg")?;

    // An A major chord: A4, C#5, E5.
    let a_major = chord(&[440.0, 554.37, 659.25], 0.25, sr, 0.03);
    let t_chord: Vec<f64> = time_axis(sr, a_major.len()).iter().map(|t| t * 1000.0).collect();
    Plot::new("An A major chord: three sines added together")
        .x_label("time (ms)")
        .y_label("amplitude")
        .line("A major", &t_chord, &a_major)
        .save("out/lesson01_chord.svg")?;

    println!("Samples per cycle of {freq} Hz at {sr} Hz: {:.3}", samples_per_cycle(freq, sr));
    println!("First 8 samples: {:?}", &samples[..8]);
    println!("Wrote out/lesson01_sine.svg and out/lesson01_chord.svg");
    Ok(())
}

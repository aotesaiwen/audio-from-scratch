//! Lesson 08: your first spectrum, and how slow the plain DFT is.
//!     cargo run --example lesson08

use lab::dft::{bin_frequency, dft, magnitude_spectrum, to_complex};
use lab::plot::Plot;
use lab::signal::chord;
use std::time::Instant;

fn main() -> std::io::Result<()> {
    let sr = 8_000;
    // 0.25 s → 2000 samples → bins 4 Hz apart.
    let x = chord(&[440.0, 554.37, 659.25], 0.3, sr, 0.25);
    let mags = magnitude_spectrum(&x);
    let freqs: Vec<f64> = (0..mags.len()).map(|k| bin_frequency(k, x.len(), sr)).collect();

    Plot::new("Spectrum of an A major chord")
        .x_label("frequency (Hz)")
        .y_label("|X[k]|")
        .x_range(0.0, 1_000.0)
        .line("magnitude", &freqs, &mags)
        .save("out/lesson08_spectrum.svg")?;

    // Local maxima well above the floor = the notes.
    let floor = mags.iter().copied().fold(0.0, f64::max) * 0.1;
    println!("Peaks found:");
    for k in 1..mags.len() - 1 {
        if mags[k] > floor && mags[k] > mags[k - 1] && mags[k] >= mags[k + 1] {
            // A sine of amplitude A gives a peak of about A·N/2.
            println!("  {:7.1} Hz   amplitude ≈ {:.3}", freqs[k], 2.0 * mags[k] / x.len() as f64);
        }
    }

    println!("\nHow long the DFT takes (N² operations):");
    for n in [256, 512, 1024, 2048, 4096] {
        let input = to_complex(&x[..n.min(x.len())]);
        let input: Vec<_> = input.iter().copied().cycle().take(n).collect();
        let start = Instant::now();
        let _ = dft(&input);
        println!("  N = {n:5}: {:8.2} ms", start.elapsed().as_secs_f64() * 1000.0);
    }
    println!("Each doubling of N should take about 4× longer. Lesson 09 fixes that.");
    println!("Wrote out/lesson08_spectrum.svg");
    Ok(())
}

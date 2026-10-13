//! Lesson 10: see spectral leakage, and what a window does about it.
//!     cargo run --example lesson10

use lab::fft::rfft;
use lab::level::amplitude_to_db;
use lab::plot::Plot;
use lab::signal::sine;
use lab::window::{apply_window, coherent_gain, hamming, hann, rectangular};

/// Magnitude spectrum in dB, scaled so a full-scale sine reads 0 dB
/// whatever the window.
fn spectrum_db(x: &[f64], w: &[f64]) -> Vec<f64> {
    let n = x.len();
    let scale = 2.0 / (n as f64 * coherent_gain(w));
    rfft(&apply_window(x, w), n)
        .iter()
        .map(|c| amplitude_to_db(c.abs() * scale).max(-140.0))
        .collect()
}

fn main() -> std::io::Result<()> {
    let (sr, n) = (8_000, 1024);
    let seconds = n as f64 / sr as f64;
    let freqs: Vec<f64> = (0..=n / 2).map(|k| k as f64 * sr as f64 / n as f64).collect();

    // 1. A tone exactly halfway between two bins: the worst case for leakage.
    let x = sine(1_003.906_25, 1.0, sr, seconds);
    Plot::new("One tone between two bins (1003.9 Hz, N = 1024)")
        .x_label("frequency (Hz)")
        .y_label("magnitude (dB)")
        .y_range(-140.0, 5.0)
        .line("rectangular (no window)", &freqs, &spectrum_db(&x, &rectangular(n)))
        .line("Hamming", &freqs, &spectrum_db(&x, &hamming(n)))
        .line("Hann", &freqs, &spectrum_db(&x, &hann(n)))
        .save("out/lesson10_leakage.svg")?;

    // 2. A loud tone and a quiet one 50 dB down, 500 Hz apart.
    let loud = sine(1_003.906_25, 1.0, sr, seconds);
    let quiet = sine(1_501.953_125, 10f64.powf(-50.0 / 20.0), sr, seconds);
    let both: Vec<f64> = loud.iter().zip(&quiet).map(|(a, b)| a + b).collect();
    Plot::new("A loud tone at 1004 Hz and a quiet one at 1502 Hz (−50 dB)")
        .x_label("frequency (Hz)")
        .y_label("magnitude (dB)")
        .x_range(500.0, 2_000.0)
        .y_range(-120.0, 5.0)
        .line("rectangular (no window)", &freqs, &spectrum_db(&both, &rectangular(n)))
        .line("Hann", &freqs, &spectrum_db(&both, &hann(n)))
        .save("out/lesson10_hidden_tone.svg")?;

    println!("Wrote out/lesson10_leakage.svg and out/lesson10_hidden_tone.svg");
    println!("In the second plot: which window lets you see the quiet tone?");
    Ok(())
}

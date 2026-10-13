//! Lesson 03: hear what bit depth does.
//!     cargo run --example lesson03

use lab::level::{amplitude_to_db, normalize_peak, peak, quantize, rms, snr_db};
use lab::plot::Plot;
use lab::signal::{chord, time_axis};
use lab::wav::{write_wav, Audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;
    let sr = 44_100;
    let x = normalize_peak(&chord(&[220.0, 277.18, 329.63], 0.3, sr, 2.0), -1.0);
    println!("Chord: peak {:.1} dBFS, RMS {:.1} dBFS", amplitude_to_db(peak(&x)), amplitude_to_db(rms(&x)));

    println!("\n bits   measured SNR   6.02·bits + 1.76");
    for bits in [16, 12, 8, 6, 4, 3, 2] {
        let q = quantize(&x, bits);
        println!("{bits:5}   {:9.1} dB   {:9.1} dB", snr_db(&x, &q), 6.02 * bits as f64 + 1.76);
        write_wav(format!("out/lesson03_{bits}bit.wav"), &Audio::new(sr, q))?;
    }
    println!("(The formula assumes a full-scale sine. Why is the chord lower?)");

    // Zoom in on 3-bit quantisation: a staircase.
    let n = 400;
    let t_ms: Vec<f64> = time_axis(sr, n).iter().map(|t| t * 1000.0).collect();
    let q3 = quantize(&x[..n], 3);
    let error: Vec<f64> = x[..n].iter().zip(&q3).map(|(a, b)| b - a).collect();
    Plot::new("3-bit quantisation")
        .x_label("time (ms)")
        .y_label("amplitude")
        .line("original", &t_ms, &x[..n])
        .line("3-bit", &t_ms, &q3)
        .line("error (3-bit − original)", &t_ms, &error)
        .save("out/lesson03_quantization.svg")?;

    println!("\nWrote out/lesson03_*bit.wav and out/lesson03_quantization.svg");
    Ok(())
}

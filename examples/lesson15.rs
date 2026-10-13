//! Lesson 15: three ways to go from 48 kHz to 16 kHz.
//!     cargo run --example lesson15

use lab::plot::Heatmap;
use lab::resample::{decimate_naive, resample_linear, resample_sinc};
use lab::stft::{power_spectrogram, power_to_db, StftConfig};
use lab::synth::chirp;
use lab::wav::{write_wav, Audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;
    // A sweep from 100 Hz to 20 kHz: the top 12 kHz of it cannot exist at 16 kHz.
    let x = chirp(100.0, 20_000.0, 0.5, 48_000, 3.0);
    let versions = [
        ("naive", decimate_naive(&x, 3)),
        ("linear", resample_linear(&x, 48_000, 16_000)),
        ("sinc", resample_sinc(&x, 48_000, 16_000, 16)),
    ];
    let cfg = StftConfig { n_fft: 512, hop_length: 128, center: true };
    for (name, y) in &versions {
        write_wav(format!("out/lesson15_{name}.wav"), &Audio::new(16_000, y.clone()))?;
        Heatmap::new(&format!("48 → 16 kHz, {name}"))
            .x_label("time (s)")
            .y_label("frequency (Hz)")
            .value_label("dB")
            .x_extent(0.0, 3.0)
            .y_extent(0.0, 8_000.0)
            .save(format!("out/lesson15_{name}.svg"), &power_to_db(&power_spectrogram(y, &cfg), 80.0))?;
    }
    println!("Wrote out/lesson15_{{naive,linear,sinc}}.{{wav,svg}}");
    println!("A correct resampler shows one line that stops at 8 kHz. Count the extra lines in the others.");
    Ok(())
}

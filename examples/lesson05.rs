//! Lesson 05: frame-level energy and zero-crossing rate, and silence trimming.
//!     cargo run --example lesson05                  (a synthetic utterance)
//!     cargo run --example lesson05 -- my_voice.wav  (your own recording)

use lab::frames::{fade, frame_rms_db, frame_zcr, trim_silence};
use lab::plot::Plot;
use lab::synth::utterance;
use lab::wav::{read_wav, write_wav, Audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;

    // `std::env::args()` is the command line; `.nth(1)` is the first
    // argument after the program name, if there is one.
    let audio = match std::env::args().nth(1) {
        Some(path) => read_wav(&path)?,
        None => Audio::new(16_000, utterance(16_000)),
    };
    let sr = audio.sample_rate;
    println!("{} samples at {sr} Hz ({:.2} s)", audio.samples.len(), audio.duration());

    // 25 ms frames every 10 ms: the standard choice for speech.
    let frame_len = (0.025 * sr as f64) as usize;
    let hop = (0.010 * sr as f64) as usize;
    let energy = frame_rms_db(&audio.samples, frame_len, hop);
    let zcr = frame_zcr(&audio.samples, frame_len, hop);
    let t: Vec<f64> = (0..energy.len()).map(|i| (i * hop + frame_len / 2) as f64 / sr as f64).collect();

    Plot::new("Loudness per frame")
        .x_label("time (s)")
        .y_label("RMS (dB)")
        .line("RMS", &t, &energy)
        .save("out/lesson05_energy.svg")?;
    Plot::new("Zero-crossing rate per frame")
        .x_label("time (s)")
        .y_label("crossings per sample")
        .line("ZCR", &t, &zcr)
        .save("out/lesson05_zcr.svg")?;

    let trimmed = trim_silence(&audio.samples, frame_len, hop, 40.0);
    let mut trimmed = trimmed.to_vec(); // copy the borrowed slice so we can change it
    fade(&mut trimmed, (0.01 * sr as f64) as usize);
    write_wav("out/lesson05_trimmed.wav", &Audio::new(sr, trimmed.clone()))?;
    println!(
        "Trimmed {:.2} s down to {:.2} s",
        audio.duration(),
        trimmed.len() as f64 / sr as f64
    );
    println!("Wrote out/lesson05_energy.svg, out/lesson05_zcr.svg, out/lesson05_trimmed.wav");
    Ok(())
}

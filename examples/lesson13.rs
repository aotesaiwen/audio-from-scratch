//! Lesson 13: track pitch.
//!     cargo run --example lesson13
//!     cargo run --example lesson13 -- my_voice.wav

use lab::pitch::{mean_and_std, normalized_autocorrelation, pitch_track, PitchConfig};
use lab::plot::Plot;
use lab::synth::{voice, VoiceParams};
use lab::wav::{read_wav, write_wav, Audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;
    let cfg = PitchConfig::default();

    // Without a file: sing do–mi–sol–do (C3 E3 G3 C4) with gaps between notes.
    let audio = match std::env::args().nth(1) {
        Some(path) => read_wav(&path)?,
        None => {
            let sr = 16_000;
            let mut x = Vec::new();
            for (i, f0) in [130.81, 164.81, 196.0, 261.63].into_iter().enumerate() {
                x.extend(vec![0.0; 2_400]);
                let p = VoiceParams { f0, jitter: 0.004, shimmer: 0.03, seconds: 0.4, sample_rate: sr, seed: i as u64, ..Default::default() };
                x.extend(voice(&p).samples);
            }
            x.extend(vec![0.0; 2_400]);
            write_wav("out/lesson13_do_mi_sol_do.wav", &Audio::new(sr, x.clone()))?;
            Audio::new(sr, x)
        }
    };
    let sr = audio.sample_rate;

    let track = pitch_track(&audio.samples, sr, &cfg);
    let frame_len = (cfg.frame_seconds * sr as f64).round();
    let times: Vec<f64> = (0..track.len()).map(|i| (i as f64 * cfg.hop_seconds) + frame_len / 2.0 / sr as f64).collect();
    let f0: Vec<f64> = track.iter().map(|f| f.unwrap_or(f64::NAN)).collect(); // NaN = gap in the plot
    Plot::new("Pitch track")
        .x_label("time (s)")
        .y_label("F0 (Hz)")
        .line("F0", &times, &f0)
        .save("out/lesson13_pitch.svg")?;

    // What the algorithm "sees": the normalised autocorrelation of one voiced frame.
    if let Some(i) = track.iter().position(|f| f.is_some()) {
        let start = (i as f64 * cfg.hop_seconds * sr as f64) as usize + sr as usize / 20; // a bit later, to be safely inside
        let len = frame_len as usize;
        if start + len <= audio.samples.len() {
            let max_lag = (sr as f64 / cfg.fmin).ceil() as usize;
            let r = normalized_autocorrelation(&audio.samples[start..start + len], max_lag);
            let lags_ms: Vec<f64> = (0..r.len()).map(|l| 1000.0 * l as f64 / sr as f64).collect();
            Plot::new("Normalised autocorrelation of one voiced frame")
                .x_label("lag (ms)")
                .y_label("r")
                .line("r", &lags_ms, &r)
                .save("out/lesson13_autocorrelation.svg")?;
        }
    }

    match mean_and_std(&track) {
        Some((mean, std)) => println!("Mean F0 {mean:.1} Hz, std {std:.1} Hz"),
        None => println!("Fewer than two voiced frames found."),
    }
    let voiced = track.iter().filter(|f| f.is_some()).count();
    println!("{voiced} of {} frames voiced", track.len());
    println!("Wrote out/lesson13_pitch.svg and out/lesson13_autocorrelation.svg");
    Ok(())
}

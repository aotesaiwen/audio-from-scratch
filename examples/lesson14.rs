//! Lesson 14: Praat-style voice reports.
//!     cargo run --example lesson14
//!     cargo run --example lesson14 -- sustained_a.wav

use lab::pitch::PitchConfig;
use lab::plot::Plot;
use lab::synth::{voice, VoiceParams};
use lab::voice::{analyze, find_cycle_peaks, periods};
use lab::wav::{read_wav, write_wav};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;
    let cfg = PitchConfig::default();

    if let Some(path) = std::env::args().nth(1) {
        let audio = read_wav(&path)?;
        match analyze(&audio, &cfg) {
            Some(report) => println!("{path}\n{report}"),
            None => println!("{path}: not enough voiced signal to analyse"),
        }
        return Ok(());
    }

    let smooth = voice(&VoiceParams { jitter: 0.003, shimmer: 0.02, snr_db: Some(30.0), seconds: 2.0, ..Default::default() });
    let rough = voice(&VoiceParams { jitter: 0.02, shimmer: 0.10, snr_db: Some(10.0), seconds: 2.0, seed: 2, ..Default::default() });
    write_wav("out/lesson14_smooth.wav", &smooth.audio())?;
    write_wav("out/lesson14_rough.wav", &rough.audio())?;

    for (name, v) in [("Smooth", &smooth), ("Rough", &rough)] {
        println!("== {name} ==");
        match analyze(&v.audio(), &cfg) {
            Some(report) => println!("{report}\n"),
            None => println!("(could not analyse)\n"),
        }
    }

    // Plot cycle lengths: jitter made visible.
    let as_ms = |v: &lab::synth::SyntheticVoice| -> Vec<f64> {
        periods(&find_cycle_peaks(&v.samples, v.sample_rate, 150.0)).iter().map(|t| t * 1000.0).collect()
    };
    let (h, r) = (as_ms(&smooth), as_ms(&rough));
    let idx = |n: usize| (0..n).map(|i| i as f64).collect::<Vec<f64>>();
    Plot::new("Length of each glottal cycle")
        .x_label("cycle number")
        .y_label("period (ms)")
        .line("smooth", &idx(h.len()), &h)
        .line("rough", &idx(r.len()), &r)
        .save("out/lesson14_periods.svg")?;

    println!("Wrote out/lesson14_smooth.wav, out/lesson14_rough.wav, out/lesson14_periods.svg");
    Ok(())
}

//! Lesson 11: spectrograms.
//!     cargo run --example lesson11
//!     cargo run --example lesson11 -- my_voice.wav

use lab::plot::Heatmap;
use lab::signal::sine;
use lab::stft::{power_spectrogram, power_to_db, StftConfig};
use lab::synth::{chirp, utterance};
use lab::wav::{read_wav, Audio};

/// Compute a dB spectrogram and save it as a heatmap.
fn save_spectrogram(path: &str, title: &str, x: &[f64], sr: u32, cfg: &StftConfig) -> std::io::Result<()> {
    let db = power_to_db(&power_spectrogram(x, cfg), 80.0);
    Heatmap::new(title)
        .x_label("time (s)")
        .y_label("frequency (Hz)")
        .value_label("dB")
        .x_extent(0.0, x.len() as f64 / sr as f64)
        .y_extent(0.0, sr as f64 / 2.0)
        .save(path, &db)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sr = 16_000;
    let cfg = StftConfig { n_fft: 512, hop_length: 128, center: true };

    save_spectrogram("out/lesson11_chirp.svg", "A chirp from 200 Hz to 6 kHz", &chirp(200.0, 6_000.0, 0.5, sr, 2.0), sr, &cfg)?;

    let audio = match std::env::args().nth(1) {
        Some(path) => read_wav(&path)?,
        None => Audio::new(sr, utterance(sr)),
    };
    // 32 ms frames every 8 ms, whatever the file's sample rate.
    let n_fft = ((0.032 * audio.sample_rate as f64) as usize).next_power_of_two();
    let speech_cfg = StftConfig { n_fft, hop_length: audio.sample_rate as usize / 125, center: true };
    save_spectrogram("out/lesson11_speech.svg", "Speech", &audio.samples, audio.sample_rate, &speech_cfg)?;

    // The time–frequency trade-off: two tones 60 Hz apart, plus clicks.
    let mut x: Vec<f64> = sine(1_000.0, 0.3, sr, 2.0).iter().zip(sine(1_060.0, 0.3, sr, 2.0)).map(|(a, b)| a + b).collect();
    for click in (2_000..x.len()).step_by(4_000) {
        x[click] += 1.0;
    }
    for n_fft in [128, 4096] {
        let c = StftConfig { n_fft, hop_length: 64, center: true };
        let title = format!("Two close tones + clicks, n_fft = {n_fft}");
        save_spectrogram(&format!("out/lesson11_tradeoff_{n_fft}.svg"), &title, &x, sr, &c)?;
    }

    println!("Wrote out/lesson11_chirp.svg, out/lesson11_speech.svg, out/lesson11_tradeoff_{{128,4096}}.svg");
    println!("In the trade-off plots: which one shows the clicks sharply? Which separates the two tones?");
    Ok(())
}

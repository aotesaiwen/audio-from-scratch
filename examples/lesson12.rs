//! Lesson 12: mel filterbank, mel spectrogram, MFCCs.
//!     cargo run --example lesson12
//!     cargo run --example lesson12 -- my_voice.wav

use lab::mel::{mel_filterbank, mel_spectrogram, mfcc};
use lab::plot::{Heatmap, Plot};
use lab::stft::{bin_frequencies, StftConfig};
use lab::synth::utterance;
use lab::wav::{read_wav, Audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The filterbank itself: 20 triangles on a 16 kHz, 512-point FFT.
    let fb = mel_filterbank(20, 512, 16_000, 0.0, 8_000.0);
    let freqs = bin_frequencies(512, 16_000);
    let mut plot = Plot::new("20 mel filters (sample rate 16 kHz, n_fft 512)")
        .x_label("frequency (Hz)")
        .y_label("weight")
        .line("mel filter", &freqs, &fb[0]);
    for row in &fb[1..] {
        plot = plot.line_same(&freqs, row);
    }
    plot.save("out/lesson12_filterbank.svg")?;

    let audio = match std::env::args().nth(1) {
        Some(path) => read_wav(&path)?,
        None => Audio::new(16_000, utterance(16_000)),
    };
    let sr = audio.sample_rate;
    // 25 ms windows (rounded up to a power of two) every 10 ms, 40 mel bands, 13 MFCCs:
    // a very common speech front end.
    let cfg = StftConfig {
        n_fft: ((0.025 * sr as f64) as usize).next_power_of_two(),
        hop_length: sr as usize / 100,
        center: true,
    };
    let duration = audio.duration();

    let mel = mel_spectrogram(&audio.samples, sr, &cfg, 40);
    let mel_db: Vec<Vec<f64>> = mel.iter().map(|f| f.iter().map(|&p| 10.0 * p.max(1e-10).log10()).collect()).collect();
    let top = mel_db.iter().flatten().copied().fold(f64::NEG_INFINITY, f64::max);
    Heatmap::new("Log-mel spectrogram (40 bands)")
        .x_label("time (s)")
        .y_label("mel band")
        .value_label("dB")
        .x_extent(0.0, duration)
        .y_extent(0.0, 40.0)
        .value_range(top - 80.0, top)
        .save("out/lesson12_mel.svg", &mel_db)?;

    let coeffs = mfcc(&audio.samples, sr, &cfg, 40, 13);
    // Skip MFCC 0 (overall loudness) so the others aren't drowned out.
    let rest: Vec<Vec<f64>> = coeffs.iter().map(|c| c[1..].to_vec()).collect();
    Heatmap::new("MFCCs 1–12")
        .x_label("time (s)")
        .y_label("coefficient")
        .value_label("value")
        .x_extent(0.0, duration)
        .y_extent(1.0, 13.0)
        .save("out/lesson12_mfcc.svg", &rest)?;

    let middle = &coeffs[coeffs.len() / 2];
    let pretty: Vec<String> = middle.iter().map(|v| format!("{v:.1}")).collect();
    println!("MFCCs of the middle frame: [{}]", pretty.join(", "));
    println!("Wrote out/lesson12_filterbank.svg, out/lesson12_mel.svg, out/lesson12_mfcc.svg");
    Ok(())
}

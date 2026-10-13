//! Lesson 06: measure and hear FIR filters.
//!     cargo run --example lesson06
//!     cargo run --example lesson06 -- my_voice.wav

use lab::fir::{echo_kernel, fir_filter, measure_gain_db, moving_average_kernel, pre_emphasis};
use lab::level::normalize_peak;
use lab::plot::Plot;
use lab::synth::utterance;
use lab::wav::{read_wav, write_wav, Audio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all("out")?;

    // Frequency response, measured one sine at a time.
    let sr = 16_000;
    let freqs: Vec<f64> = (1..160).map(|i| i as f64 * 50.0).collect(); // 50 Hz .. 7950 Hz
    let ma5 = moving_average_kernel(5);
    let ma21 = moving_average_kernel(21);
    let curve = |filter: &dyn Fn(&[f64]) -> Vec<f64>| -> Vec<f64> {
        freqs.iter().map(|&f| measure_gain_db(filter, f, sr).max(-60.0)).collect()
    };
    Plot::new("Measured frequency response (sample rate 16 kHz)")
        .x_label("frequency (Hz)")
        .y_label("gain (dB)")
        .line("5-tap moving average", &freqs, &curve(&|x| fir_filter(x, &ma5)))
        .line("21-tap moving average", &freqs, &curve(&|x| fir_filter(x, &ma21)))
        .line("pre-emphasis 0.97", &freqs, &curve(&|x| pre_emphasis(x, 0.97)))
        .save("out/lesson06_response.svg")?;

    // Hear them.
    let audio = match std::env::args().nth(1) {
        Some(path) => read_wav(&path)?,
        None => Audio::new(sr, utterance(sr)),
    };
    let sr = audio.sample_rate;
    let x = &audio.samples;
    let save = |name: &str, y: Vec<f64>| write_wav(format!("out/lesson06_{name}.wav"), &Audio::new(sr, normalize_peak(&y, -1.0)));
    save("original", x.clone())?;
    save("muffled", fir_filter(x, &moving_average_kernel(21)))?;
    save("pre_emphasis", pre_emphasis(x, 0.97))?;
    save("echo", fir_filter(x, &echo_kernel((0.25 * sr as f64) as usize, 0.5)))?;

    println!("Wrote out/lesson06_response.svg and out/lesson06_{{original,muffled,pre_emphasis,echo}}.wav");
    Ok(())
}

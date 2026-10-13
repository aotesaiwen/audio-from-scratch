//! Lesson 02: your first sounds.
//!     cargo run --example lesson02
//! Then play the .wav files in out/ with any audio player.
//!
//! Or inspect one of your own recordings:
//!     cargo run --example lesson02 -- path/to/recording.wav

use lab::signal::{chord, sine};
use lab::wav::{encode_wav, read_u16, read_u32, read_wav, write_wav, Audio};

/// `Box<dyn std::error::Error>` means "any kind of error". It lets `?` pass
/// on both file errors (io::Error) and our own String errors.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(path) = std::env::args().nth(1) {
        let bytes = std::fs::read(&path)?;
        if bytes.len() >= 36 && &bytes[12..16] == b"fmt " {
            println!(
                "Header says: format {}, {} channel(s), {} Hz, {} bits",
                read_u16(&bytes, 20),
                read_u16(&bytes, 22),
                read_u32(&bytes, 24),
                read_u16(&bytes, 34)
            );
        }
        let audio = read_wav(&path)?;
        println!(
            "{path}: {} samples at {} Hz = {:.3} s (mixed to mono)",
            audio.samples.len(),
            audio.sample_rate,
            audio.duration()
        );
        return Ok(());
    }

    std::fs::create_dir_all("out")?;
    let sr = 44_100;

    let a440 = Audio::new(sr, sine(440.0, 0.5, sr, 2.0));
    write_wav("out/lesson02_a440.wav", &a440)?;

    // Amplitude 1.5 doesn't fit in [-1, 1], so f64_to_i16 clips the tops off.
    let clipped = Audio::new(sr, sine(440.0, 1.5, sr, 2.0));
    write_wav("out/lesson02_a440_clipped.wav", &clipped)?;

    let a_major = Audio::new(sr, chord(&[440.0, 554.37, 659.25], 0.25, sr, 2.0));
    write_wav("out/lesson02_a_major.wav", &a_major)?;

    // Look inside a WAV file: the first 44 bytes are the header.
    let bytes = encode_wav(&a440);
    println!("The first 48 bytes of out/lesson02_a440.wav:");
    for (row, chunk) in bytes[..48].chunks(12).enumerate() {
        let hex: Vec<String> = chunk.iter().map(|b| format!("{b:02x}")).collect();
        let text: String = chunk.iter().map(|&b| if b.is_ascii_graphic() { b as char } else { '.' }).collect();
        println!("  {:3}: {}   {}", row * 12, hex.join(" "), text);
    }

    let back = read_wav("out/lesson02_a440.wav")?;
    println!(
        "\nRead it back: {} samples at {} Hz = {:.2} s",
        back.samples.len(),
        back.sample_rate,
        back.duration()
    );
    println!("Wrote out/lesson02_a440.wav, out/lesson02_a440_clipped.wav, out/lesson02_a_major.wav");
    Ok(())
}

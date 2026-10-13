//! Lesson 02: WAV files, bytes on disk. See `lessons/02-wav-files/README.md`.
//!
//! Check with `cargo test l02`, then run `cargo run --example lesson02`.

use std::path::Path;

/// Mono audio: samples in [-1.0, 1.0], plus the rate they were taken at.
#[derive(Debug, Clone, PartialEq)]
pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<f64>,
}

impl Audio {
    pub fn new(sample_rate: u32, samples: Vec<f64>) -> Self {
        Audio { sample_rate, samples }
    }

    /// Length in seconds.
    pub fn duration(&self) -> f64 {
        // Inside a method, `self` is the Audio, so `self.samples.len()` works.
        todo!("l02: Audio::duration")
    }
}

/// Float sample to 16-bit integer: scale [-1, 1] up to [-32767, 32767] and
/// round. Values outside [-1, 1] are clipped to the limit first.
pub fn f64_to_i16(x: f64) -> i16 {
    // f64 has `.clamp(lo, hi)` and `.round()`. Then `as i16`.
    todo!("l02: f64_to_i16")
}

/// 16-bit integer to float: divide by 32768, giving values in [-1, 1).
/// (librosa and soundfile use this same convention.)
pub fn i16_to_f64(s: i16) -> f64 {
    todo!("l02: i16_to_f64")
}

/// Encode audio as a complete 16-bit PCM mono WAV file: a 44-byte header
/// followed by every sample as two little-endian bytes.
pub fn encode_wav(audio: &Audio) -> Vec<u8> {
    // The first 12 bytes are written for you as a worked example. Follow the
    // byte table in lessons/02-wav-files/README.md for the rest.
    let data_size = (audio.samples.len() * 2) as u32;
    let mut out: Vec<u8> = Vec::new();
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_size).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    todo!("l02: encode_wav: the fmt chunk, the data chunk header, then the samples")
}

/// Write audio to a WAV file.
///
/// `impl AsRef<Path>` means "anything that can be viewed as a file path":
/// a `&str`, a `String` or a `PathBuf` all work.
pub fn write_wav(path: impl AsRef<Path>, audio: &Audio) -> std::io::Result<()> {
    // One line: std::fs::write takes a path and the bytes to write.
    todo!("l02: write_wav")
}

/// Read a little-endian u16 starting at `bytes[at]`. Given helper.
pub fn read_u16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

/// Read a little-endian u32 starting at `bytes[at]`. Given helper.
pub fn read_u32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}

/// Parse the bytes of a WAV file.
///
/// Real WAV files contain more chunks than the two you wrote ("LIST",
/// "fact", "bext", ...), in any order, so walk through the chunks one at a
/// time: read a 4-byte id and a 4-byte size, handle "fmt " and "data", skip
/// anything else. Chunks with an odd size are followed by one padding byte.
///
/// Returns an error message (instead of crashing) for files it can't read.
pub fn decode_wav(bytes: &[u8]) -> Result<Audio, String> {
    // See the README for the chunk layout and for how `Result` and `?` work.
    todo!("l02: decode_wav")
}

/// Turn the raw bytes of a "data" chunk into mono samples in [-1, 1].
/// Interleaved channels (L R L R ...) are averaged into one.
///
/// Required: 16-bit integer PCM (format tag 1, 16 bits).
/// Stretch goal: 8/24/32-bit PCM and 32-bit float (format tag 3).
pub fn decode_samples(data: &[u8], format_tag: u16, channels: u16, bits: u16) -> Result<Vec<f64>, String> {
    // Start with 16-bit only and return an Err for anything else, e.g.
    //     return Err(format!("unsupported: format {format_tag}, {bits} bits"));
    todo!("l02: decode_samples")
}

/// Read a WAV file from disk.
pub fn read_wav(path: impl AsRef<Path>) -> Result<Audio, String> {
    // std::fs::read gives a Result<Vec<u8>, std::io::Error>. Turn its error
    // into a String with `.map_err(|e| e.to_string())` so `?` can pass it on.
    todo!("l02: read_wav")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signal::sine;
    use crate::testutil::*;

    /// Build a WAV file by hand, optionally with an extra chunk before "data".
    fn make_wav(channels: u16, bits: u16, sample_rate: u32, extra: Option<(&[u8; 4], &[u8])>, data: &[u8]) -> Vec<u8> {
        let mut body = Vec::new();
        body.extend_from_slice(b"WAVE");
        body.extend_from_slice(b"fmt ");
        body.extend_from_slice(&16u32.to_le_bytes());
        body.extend_from_slice(&1u16.to_le_bytes());
        body.extend_from_slice(&channels.to_le_bytes());
        body.extend_from_slice(&sample_rate.to_le_bytes());
        let block = channels as u32 * bits as u32 / 8;
        body.extend_from_slice(&(sample_rate * block).to_le_bytes());
        body.extend_from_slice(&(block as u16).to_le_bytes());
        body.extend_from_slice(&bits.to_le_bytes());
        if let Some((id, content)) = extra {
            body.extend_from_slice(id);
            body.extend_from_slice(&(content.len() as u32).to_le_bytes());
            body.extend_from_slice(content);
            if content.len() % 2 == 1 {
                body.push(0);
            }
        }
        body.extend_from_slice(b"data");
        body.extend_from_slice(&(data.len() as u32).to_le_bytes());
        body.extend_from_slice(data);
        let mut file = b"RIFF".to_vec();
        file.extend_from_slice(&(body.len() as u32).to_le_bytes());
        file.extend_from_slice(&body);
        file
    }

    #[test]
    fn l02_duration() {
        assert_close(Audio::new(16_000, vec![0.0; 8_000]).duration(), 0.5, 1e-12);
        assert_close(Audio::new(44_100, vec![]).duration(), 0.0, 1e-12);
    }

    #[test]
    fn l02_f64_to_i16() {
        assert_eq!(f64_to_i16(0.0), 0);
        assert_eq!(f64_to_i16(1.0), 32767);
        assert_eq!(f64_to_i16(-1.0), -32767);
        assert_eq!(f64_to_i16(0.5), 16384);
        assert_eq!(f64_to_i16(1.7), 32767, "values above 1.0 should clip");
        assert_eq!(f64_to_i16(-3.0), -32767, "values below -1.0 should clip");
    }

    #[test]
    fn l02_i16_to_f64() {
        assert_close(i16_to_f64(0), 0.0, 1e-12);
        assert_close(i16_to_f64(-32768), -1.0, 1e-12);
        assert_close(i16_to_f64(16384), 0.5, 1e-12);
    }

    #[test]
    fn l02_header_bytes() {
        let bytes = encode_wav(&Audio::new(16_000, vec![0.0, 0.5]));
        assert_eq!(bytes.len(), 48, "44-byte header + 2 samples × 2 bytes");
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(read_u32(&bytes, 4), 40, "RIFF size = file length − 8");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(read_u32(&bytes, 16), 16, "fmt chunk size");
        assert_eq!(read_u16(&bytes, 20), 1, "format tag (1 = PCM)");
        assert_eq!(read_u16(&bytes, 22), 1, "channels");
        assert_eq!(read_u32(&bytes, 24), 16_000, "sample rate");
        assert_eq!(read_u32(&bytes, 28), 32_000, "byte rate");
        assert_eq!(read_u16(&bytes, 32), 2, "block align");
        assert_eq!(read_u16(&bytes, 34), 16, "bits per sample");
        assert_eq!(&bytes[36..40], b"data");
        assert_eq!(read_u32(&bytes, 40), 4, "data size in bytes");
        assert_eq!(&bytes[44..48], &[0, 0, 0x00, 0x40], "samples 0 and 16384, little-endian");
    }

    #[test]
    fn l02_round_trip() {
        let original = Audio::new(22_050, sine(440.0, 0.8, 22_050, 0.1));
        let decoded = decode_wav(&encode_wav(&original)).expect("decoding failed");
        assert_eq!(decoded.sample_rate, 22_050);
        assert_all_close(&decoded.samples, &original.samples, 1e-4);
    }

    #[test]
    fn l02_write_then_read_a_file() {
        let path = std::env::temp_dir().join("audio_from_scratch_l02_test.wav");
        let original = Audio::new(8_000, sine(1000.0, 0.5, 8_000, 0.05));
        write_wav(&path, &original).expect("writing failed");
        let back = read_wav(&path).expect("reading failed");
        assert_eq!(back.sample_rate, 8_000);
        assert_all_close(&back.samples, &original.samples, 1e-4);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn l02_stereo_is_mixed_to_mono() {
        // Two frames: (L=16384, R=0) and (L=-16384, R=-16384)
        let mut data = Vec::new();
        for s in [16384i16, 0, -16384, -16384] {
            data.extend_from_slice(&s.to_le_bytes());
        }
        let audio = decode_wav(&make_wav(2, 16, 44_100, None, &data)).expect("decoding failed");
        assert_eq!(audio.sample_rate, 44_100);
        assert_all_close(&audio.samples, &[0.25, -0.5], 1e-9);
    }

    #[test]
    fn l02_skips_unknown_chunks() {
        // A 3-byte LIST chunk: odd size, so one padding byte follows it.
        let data = 8192i16.to_le_bytes();
        let wav = make_wav(1, 16, 16_000, Some((b"LIST", &[1, 2, 3])), &data);
        let audio = decode_wav(&wav).expect("decoding failed");
        assert_all_close(&audio.samples, &[0.25], 1e-9);
    }

    #[test]
    fn l02_rejects_garbage() {
        assert!(decode_wav(b"definitely not a wav file").is_err());
        assert!(decode_wav(&[]).is_err());
        assert!(read_wav("this/file/does/not/exist.wav").is_err());
    }

    #[test]
    #[ignore = "stretch goal: run with `cargo test l02 -- --ignored`"]
    fn l02_stretch_24_bit_and_float() {
        // 24-bit: 0x400000 = half scale, then -half scale
        let data24 = [0x00, 0x00, 0x40, 0x00, 0x00, 0xC0];
        let audio = decode_wav(&make_wav(1, 24, 48_000, None, &data24)).expect("24-bit failed");
        assert_all_close(&audio.samples, &[0.5, -0.5], 1e-9);
        let floats: Vec<u8> = [0.25f32, -0.75].iter().flat_map(|v| v.to_le_bytes()).collect();
        let samples = decode_samples(&floats, 3, 1, 32).expect("float failed");
        assert_all_close(&samples, &[0.25, -0.75], 1e-9);
    }
}

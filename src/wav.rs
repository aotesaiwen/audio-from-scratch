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
        self.samples.len() as f64 / self.sample_rate as f64
    }
}

/// Float sample to 16-bit integer: scale [-1, 1] up to [-32767, 32767] and
/// round. Values outside [-1, 1] are clipped to the limit first.
pub fn f64_to_i16(x: f64) -> i16 {
    (x.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// 16-bit integer to float: divide by 32768, giving values in [-1, 1).
/// (librosa and soundfile use this same convention.)
pub fn i16_to_f64(s: i16) -> f64 {
    s as f64 / 32768.0
}

/// Encode audio as a complete 16-bit PCM mono WAV file: a 44-byte header
/// followed by every sample as two little-endian bytes.
pub fn encode_wav(audio: &Audio) -> Vec<u8> {
    let bytes_per_sample = 2u32;
    let data_size = audio.samples.len() as u32 * bytes_per_sample;
    let mut out = Vec::with_capacity(44 + data_size as usize);
    // RIFF header: "this is a RIFF file of N more bytes, and it's a WAVE"
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data_size).to_le_bytes());
    out.extend_from_slice(b"WAVE");
    // fmt chunk: how to interpret the samples
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes()); // size of the rest of this chunk
    out.extend_from_slice(&1u16.to_le_bytes()); // format 1 = integer PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // channels
    out.extend_from_slice(&audio.sample_rate.to_le_bytes());
    out.extend_from_slice(&(audio.sample_rate * bytes_per_sample).to_le_bytes()); // bytes per second
    out.extend_from_slice(&(bytes_per_sample as u16).to_le_bytes()); // bytes per frame
    out.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
    // data chunk: the samples themselves
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_size.to_le_bytes());
    for &x in &audio.samples {
        out.extend_from_slice(&f64_to_i16(x).to_le_bytes());
    }
    out
}

/// Write audio to a WAV file.
///
/// `impl AsRef<Path>` means "anything that can be viewed as a file path":
/// a `&str`, a `String` or a `PathBuf` all work.
pub fn write_wav(path: impl AsRef<Path>, audio: &Audio) -> std::io::Result<()> {
    std::fs::write(path, encode_wav(audio))
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
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err("not a RIFF/WAVE file".to_string());
    }
    // (format tag, channels, sample rate, bits per sample), once we've seen "fmt "
    let mut format: Option<(u16, u16, u32, u16)> = None;
    let mut pos = 12;
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let size = read_u32(bytes, pos + 4) as usize;
        let body_start = pos + 8;
        let body = &bytes[body_start..(body_start + size).min(bytes.len())];

        if id == b"fmt " {
            if body.len() < 16 {
                return Err("fmt chunk is too short".to_string());
            }
            let mut tag = read_u16(body, 0);
            let channels = read_u16(body, 2);
            let sample_rate = read_u32(body, 4);
            let bits = read_u16(body, 14);
            if tag == 0xFFFE && body.len() >= 26 {
                // WAVE_FORMAT_EXTENSIBLE keeps the real format tag further in.
                tag = read_u16(body, 24);
            }
            format = Some((tag, channels, sample_rate, bits));
        } else if id == b"data" {
            let (tag, channels, sample_rate, bits) = format.ok_or("data chunk before fmt chunk")?;
            let samples = decode_samples(body, tag, channels, bits)?;
            return Ok(Audio::new(sample_rate, samples));
        }
        pos = body_start + size + size % 2;
    }
    Err("no data chunk found".to_string())
}

/// Turn the raw bytes of a "data" chunk into mono samples in [-1, 1].
/// Interleaved channels (L R L R ...) are averaged into one.
///
/// Required: 16-bit integer PCM (format tag 1, 16 bits).
/// Stretch goal: 8/24/32-bit PCM and 32-bit float (format tag 3).
pub fn decode_samples(data: &[u8], format_tag: u16, channels: u16, bits: u16) -> Result<Vec<f64>, String> {
    if channels == 0 || bits % 8 != 0 || bits == 0 {
        return Err(format!("unsupported WAV layout: {channels} channels, {bits} bits"));
    }
    let channels = channels as usize;
    let width = (bits / 8) as usize;
    let frame_size = width * channels;
    let mut out = Vec::with_capacity(data.len() / frame_size);
    for frame in data.chunks_exact(frame_size) {
        let mut sum = 0.0;
        for b in frame.chunks_exact(width) {
            sum += match (format_tag, bits) {
                (1, 8) => (b[0] as f64 - 128.0) / 128.0, // 8-bit WAV is unsigned
                (1, 16) => i16_to_f64(i16::from_le_bytes([b[0], b[1]])),
                // Put the 3 bytes in the top of an i32, then shift back down:
                // the arithmetic shift keeps the sign.
                (1, 24) => (i32::from_le_bytes([0, b[0], b[1], b[2]]) >> 8) as f64 / 8_388_608.0,
                (1, 32) => i32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64 / 2_147_483_648.0,
                (3, 32) => f32::from_le_bytes([b[0], b[1], b[2], b[3]]) as f64,
                _ => return Err(format!("unsupported WAV format: tag {format_tag}, {bits} bits")),
            };
        }
        out.push(sum / channels as f64);
    }
    Ok(out)
}

/// Read a WAV file from disk.
pub fn read_wav(path: impl AsRef<Path>) -> Result<Audio, String> {
    let path = path.as_ref();
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
    decode_wav(&bytes)
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

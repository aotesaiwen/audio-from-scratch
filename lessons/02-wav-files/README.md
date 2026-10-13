# Lesson 02: WAV files, bytes on disk

> **Audio:** PCM, bit depth, clipping, file headers · **Rust:** structs, bytes, `Result` and `?`
> **You edit:** `src/wav.rs` · **Check:** `cargo test l02` · **Demo:** `cargo run --example lesson02` · **Time:** ~2–3 h

Last lesson's samples lived in memory. Now you'll write them to a file any
audio player can open, and read real recordings back in. After this lesson
you can load your own dataset without any library.

## 1. The idea

A WAV file is a short **header** that says how to read the samples, followed
by the samples themselves, stored as plain integers. That storage scheme is
called **PCM** (pulse-code modulation).

**16-bit samples.** Most WAV files store each sample as a 16-bit signed
integer, from −32768 to 32767. Your floats in [−1, 1] get scaled up:

```
float  -1.0      -0.5      0.0      0.5      1.0
int16  -32767   -16384      0     16384    32767
```

Anything beyond ±1.0 can't be stored. It gets **clipped** to the limit,
which flattens the tops of the wave and sounds harsh. You'll hear it.

**Little-endian.** A 16-bit number takes two bytes, and WAV stores the
*least* significant byte first. 16384 is `0x4000` in hex, stored as `00 40`.
A 32-bit 44100 (`0x0000AC44`) is stored as `44 AC 00 00`.

**Chunks.** A WAV file is a RIFF file: a sequence of chunks, each starting
with a 4-letter id and a 4-byte size. The simplest WAV has two chunks inside
the outer RIFF one:

| Offset | Bytes | Field | Value (16-bit mono) |
|---|---|---|---|
| 0 | 4 | id | `"RIFF"` |
| 4 | 4 | size of everything after this field | 36 + data size |
| 8 | 4 | RIFF type | `"WAVE"` |
| 12 | 4 | chunk id | `"fmt "` (with a space) |
| 16 | 4 | chunk size | 16 |
| 20 | 2 | format tag | 1 (integer PCM) |
| 22 | 2 | channels | 1 |
| 24 | 4 | sample rate | e.g. 16000 |
| 28 | 4 | byte rate | sample rate × channels × 2 |
| 32 | 2 | block align (bytes per frame) | channels × 2 |
| 34 | 2 | bits per sample | 16 |
| 36 | 4 | chunk id | `"data"` |
| 40 | 4 | chunk size | number of samples × 2 |
| 44 | … | samples | each an i16, little-endian |

Real recorders add other chunks (`LIST` for metadata, `fact`, `bext`, ...).
A robust reader therefore *walks* the chunks: read an id and a size, deal
with the chunk if you know it, otherwise skip `size` bytes. One quirk: if
`size` is odd, one padding byte follows.

**Stereo** files interleave channels, `L R L R ...`. To get mono, average
each pair. That's what `librosa.load(..., mono=True)` does.

## 2. Rust for this lesson

**Structs and methods:**

```rust
pub struct Audio {
    pub sample_rate: u32,
    pub samples: Vec<f64>,
}

impl Audio {
    pub fn duration(&self) -> f64 { ... }   // `&self`: read access to this Audio
}

let a = Audio::new(16_000, vec![0.0; 100]);
a.duration();
```

**Bytes:**

```rust
let b: &[u8; 4] = b"RIFF";                 // a byte-string literal
let bytes: [u8; 4] = 44100u32.to_le_bytes(); // [0x44, 0xAC, 0x00, 0x00]
let n = u32::from_le_bytes([0x44, 0xAC, 0, 0]); // back to 44100
let mut out: Vec<u8> = Vec::new();
out.extend_from_slice(b"RIFF");             // append several bytes
out.extend_from_slice(&n.to_le_bytes());
```

**Errors are values.** Reading a file can fail, and Rust makes you deal with
that. A function that can fail returns `Result<T, E>`: either `Ok(value)` or
`Err(error)`. The `?` operator means *if this is an error, return it from my
function right now; otherwise give me the value*. Here's a parallel example
that parses a made-up format, "the letters HI, then a 2-byte count":

```rust
fn parse_count(bytes: &[u8]) -> Result<u16, String> {
    if bytes.len() < 4 || &bytes[0..2] != b"HI" {
        return Err("not a HI file".to_string());
    }
    Ok(u16::from_le_bytes([bytes[2], bytes[3]]))
}

fn read_count(path: &str) -> Result<u16, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?; // io::Error → String
    parse_count(&bytes)
}
```

- `map_err` converts one error type into another, so `?` can pass it on.
- `option.ok_or("message")?` turns a `None` into an error.
- Slicing `&bytes[a..b]` *panics* if `b` is past the end, so check lengths first.
- `match (format_tag, bits) { (1, 16) => ..., _ => ... }` matches several
  values at once. `_` means "anything else".
- `data.chunks_exact(4)` walks a slice in pieces of exactly 4.

## 3. Predict first

1. How many bytes is a 3-second, 16-bit mono WAV at 16 kHz?
2. Write out the two bytes for the i16 values 1, 256 and −1.
3. A sine with amplitude 1.5 is written to 16-bit. Sketch what's stored.
4. Why is `f64_to_i16` asked to multiply by 32767, not 32768?

## 4. Exercises

In `src/wav.rs`, in this order (each one is tested on its own):

1. `Audio::duration`
2. `f64_to_i16` and `i16_to_f64`
3. `encode_wav` (the first 12 bytes are done for you)
4. `write_wav`
5. `decode_samples` (16-bit is required; the others are a stretch goal)
6. `decode_wav`
7. `read_wav`

Stretch goal: support 24-bit, 32-bit and float WAVs, then run
`cargo test l02 -- --ignored`. Worth doing if your recordings aren't 16-bit.

<details><summary>Hint: the shape of decode_wav</summary>

```
check bytes 0..4 are "RIFF" and 8..12 are "WAVE"
pos = 12; format = None
while at least 8 bytes remain at pos:
    id   = bytes[pos .. pos+4]
    size = read_u32(bytes, pos + 4)
    body = bytes[pos+8 .. pos+8+size]       (don't run past the end of bytes)
    if id == "fmt ": format = Some((tag, channels, rate, bits)) read from body
    if id == "data": decode body with the remembered format; return Ok(...)
    pos = pos + 8 + size + (size % 2)
return Err("no data chunk")
```

The `fmt ` body has the format tag at offset 0, channels at 2, sample rate at
4, bits per sample at 14. Use the given `read_u16` / `read_u32`.

</details>

<details><summary>Hint: decode_samples</summary>

Bytes per frame = (bits / 8) × channels. Loop over `data.chunks_exact(frame_size)`.
Inside, loop over the channels, convert each 2-byte piece with
`i16::from_le_bytes([b[0], b[1]])` and `i16_to_f64`, add them up, then
divide by the number of channels.

</details>

## 5. Run it

```bash
cargo run --example lesson02
```

The demo writes three WAVs and prints the first 48 bytes of one as hex.
Find `RIFF`, `fmt `, `data` and the sample rate (`44 ac 00 00`).

Listen to `lesson02_a440.wav` and then `lesson02_a440_clipped.wav`.

Then inspect one of your own recordings:

```bash
cargo run --example lesson02 -- C:\path\to\a\recording.wav
```

If it's not 16-bit and you skipped the stretch goal, you'll get your own
error message back.

## 6. Check your understanding

1. Why divide by 32768 when reading, but multiply by 32767 when writing?
2. What's the *first* thing you'd check in the header of a dataset you
   just downloaded?
3. A stereo recording has the right channel wired with inverted polarity.
   What does mixing it to mono do?

<details><summary>Answers</summary>

1. i16 runs from −32768 to +32767, so it's lopsided. Reading: −32768/32768 is
   exactly −1.0 and nothing exceeds 1. Writing: 1.0 × 32768 would overflow, so
   use 32767. (Libraries differ on this detail, and it's why round trips can
   be off by one count.)
2. Sample rate, channels and bit depth. Mixing up sample rates silently
   changes pitch and timing, and every downstream feature shifts.
3. L + (−L) = 0. Silence, or a thin, phasey remainder. Averaging channels
   assumes they're in phase.

</details>

Next: [Lesson 03, loudness and bits](../03-loudness/README.md)

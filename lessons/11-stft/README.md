# Lesson 11: The STFT and spectrograms

> **Audio:** short-time Fourier transform, spectrograms, the time–frequency trade-off · **Rust:** config structs, `Default`, struct update syntax, `Vec<Vec<T>>`
> **You edit:** `src/stft.rs` · **Check:** `cargo test l11` · **Demo:** `cargo run --example lesson11` · **Time:** ~1.5 h

This is where your lesson 05 frames, lesson 09 FFT and lesson 10 window
come together. By the end, `librosa.stft(wav, n_fft=1024, hop_length=256,
win_length=1024, window='hann')` will be a function you've written.

## 1. The idea

One FFT of a whole recording tells you which frequencies occur, but not
*when*. The **short-time Fourier transform** cuts the signal into
overlapping frames, windows each one, and FFTs each one:

```
signal  ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
frames  [hann·fft] [hann·fft] [hann·fft] ...      one spectrum per hop
               →  stack them side by side  →  spectrogram (time × frequency)
```

Plot |X|² in dB, with time across, frequency up and colour for level, and
you have a **spectrogram**.

**Parameters, as librosa names them:**

| | meaning | librosa default |
|---|---|---|
| `n_fft` | frame length = FFT size → frequency resolution sr / n_fft | 2048 |
| `hop_length` | samples between frame starts → time step | 512 (n_fft/4) |
| `center` | pad n_fft/2 zeros at both ends so frame i is *centred* on sample i·hop | true |

With `center = true` there are `1 + len / hop` frames (integer division).

**The time–frequency trade-off.** A long frame pins frequency down
precisely but blurs *when* things happen. A short frame is the reverse. You
can't have both (it's the same maths as the uncertainty principle). Speech
scientists use both on purpose:

- **Wideband** (short, ~5 ms windows) shows formants as broad dark bands,
  and individual vocal-fold pulses as vertical stripes.
- **Narrowband** (long, ~30–50 ms) shows individual harmonics as horizontal lines.

**Decibels for display.** `power_to_db(S, ref=max, top_db=80)` makes the
loudest cell 0 dB and floors everything below −80 dB, so quiet noise
doesn't swamp the colour scale.

Common settings, decoded:

- `hop_length = n_fft // 4` is librosa's standard 75% overlap.
- `torchaudio.transforms.MelSpectrogram(sample_rate=16000, n_mels=64)`
  defaults to `n_fft=400` (25 ms) and `hop_length=200` (12.5 ms). 400 isn't a
  power of two; torch's FFT handles any length, while yours would need
  zero-padding to 512.

## 2. Rust for this lesson

**Config structs and `Default`.** Long argument lists are error-prone, so
group settings in a struct and give it defaults:

```rust
impl Default for StftConfig {
    fn default() -> Self { StftConfig { n_fft: 2048, /* ... */ } }
}

let cfg = StftConfig::default();
let cfg = StftConfig { n_fft: 512, ..Default::default() }; // override one field
```

`..Default::default()` is the **struct update syntax**: "take the rest of the
fields from this other value".

**2-D data** here is `Vec<Vec<f64>>`, indexed `[frame][bin]`. (Big numeric
libraries use one flat Vec plus index arithmetic for speed. Nested Vecs are
clearer.)

**Joining slices:** `[a, b, c].concat()` glues three slices into one new Vec.
That's handy for padding.

`spec.iter().flatten()` walks every value of every row.

## 3. Predict first

1. 16,000 samples with the default config: how many frames, and how many bins each?
2. `n_fft=1024, hop=256` at 16 kHz: frames per second? Frequency
   resolution? Window duration?
3. Sketch the spectrogram of a chirp from 200 Hz to 6 kHz.
4. The demo makes two close tones (1000 and 1060 Hz) plus a click every
   0.25 s, analysed with n_fft = 128 and 4096. Which plot shows sharp clicks?
   Which separates the tones?

## 4. Exercises

In `src/stft.rs`:

1. `Default for StftConfig`
2. `stft`
3. `power_spectrogram`
4. `power_to_db`
5. `frame_times`, `bin_frequencies`

<details><summary>Hint: stft</summary>

Build an owned `signal: Vec<f64>`: either the padded version or `x.to_vec()`.
Then `frames(&signal, n_fft, hop)` from lesson 05, and for each frame
`rfft(&apply_window(frame, &window), n_fft)`. Make the window once, outside the loop.

</details>

## 5. Run it

```bash
cargo run --example lesson11
```

- `out/lesson11_chirp.svg`: does it match your sketch?
- `out/lesson11_speech.svg`: the synthetic utterance. Find the two vowels
  (dark horizontal bands are formants; thin lines are harmonics), the "sss"
  (noise up high) and the silences.
- `out/lesson11_tradeoff_128.svg` vs `_4096.svg`: check your prediction.

Then look at your own voice:

```bash
cargo run --example lesson11 -- C:\path\to\recording.wav
```

## 6. Check your understanding

1. Why does `center = true` exist?
2. In a narrowband spectrogram of a sustained /a/, what does jitter look like
   on the higher harmonics?
3. For a CNN classifier, what might you gain and lose by switching from
   n_fft = 2048 to n_fft = 512 at 16 kHz?

<details><summary>Answers</summary>

1. So frame i describes the sound *at* time i·hop/sr rather than starting
   there, and the first frame isn't missing the start of the signal.
2. The k-th harmonic sits at k·F0, so F0 wobble is multiplied by k: high
   harmonics look smeared and fuzzy while low ones stay sharp.
3. Gain: 32 ms frames instead of 128 ms, so onsets and fast changes stay
   sharp. Lose: 31 Hz bins instead of 8 Hz, so individual harmonics (pitch
   detail) blur into the envelope.

</details>

Next: [Lesson 12, mel spectrograms and MFCCs](../12-mel-mfcc/README.md)

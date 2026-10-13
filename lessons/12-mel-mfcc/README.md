# Lesson 12: Mel spectrograms and MFCCs

> **Audio:** the mel scale, triangular filterbanks, log compression, the DCT, cepstra · **Rust:** matrix maths with iterators
> **You edit:** `src/mel.rs` · **Check:** `cargo test l12` · **Demo:** `cargo run --example lesson12` · **Time:** ~2 h

These are the standard inputs to speech classifiers: log-mel for CNNs,
MFCCs for GMMs. After this lesson you'll know exactly what they contain, and
what they throw away.

## 1. The idea

**The mel scale.** We hear pitch roughly in ratios above about 1 kHz (500
→ 1000 Hz sounds like the same jump as 2000 → 4000 Hz) and closer to
linearly below it. The mel scale bends frequency to match:

    mel = 2595 · log10(1 + hz / 700)        (the "HTK" formula)

1000 Hz ≈ 1000 mel by design. Equal steps in mel are narrow in Hz at the
bottom and wide at the top.

**The filterbank.** Space `n_mels + 2` points evenly *in mel* between fmin and
fmax, convert them back to Hz, and build a triangle on each consecutive
three:

```
weight
 1 |    /\      /\         /\              /\
   |   /  \    /  \       /  \            /  \
 0 +--/----\--/----\-----/----\----------/----\------> Hz
     narrow at low frequencies ... wide at high frequencies
```

Each triangle adds up the power in its band. That gives `n_mels` numbers per
frame instead of `n_fft/2 + 1`. Neighbouring triangles overlap so that,
between the first and last centres, the weights at any frequency sum to 1.

**Mel spectrogram** = filterbank × power spectrum, frame by frame. Take its
log and you have a **log-mel spectrogram**, the image a CNN sees.

**What mel smoothing removes.** Low filters are narrow enough to see
individual harmonics. High filters are hundreds of Hz wide and average
across several harmonics, so mostly the **envelope** survives (the formants,
the vocal tract) while the fine harmonic structure (the source, pitch) gets
blurred.

**MFCCs** go one step further: take the **DCT** of each log-mel frame and
keep the first ~13 coefficients.

    X[k] = s_k · Σ_n x[n] · cos(π·k·(2n + 1) / (2N)),  s_0 = √(1/N), s_k = √(2/N)

The DCT re-describes a frame's shape as a sum of cosines across the mel
bands: c0 is the overall level, c1 the overall tilt, higher ones finer
ripples. Keeping only the first few keeps the *smooth* shape (envelope) and
drops the fine ripple (harmonics), so MFCCs are partly pitch-independent by
design. The DCT also **decorrelates**: neighbouring mel bands move together,
but DCT coefficients are nearly independent.

That's why the two model families wanted different inputs:

| Model | Input | Why |
|---|---|---|
| GMM with diagonal covariances | MFCCs | it assumes the dimensions are independent, which is close to true for MFCCs and far from true for mel bands |
| CNN | log-mel | it *exploits* local structure: neighbouring bands and frames form patterns |

**Conventions differ, and they matter.** librosa's default is the *Slaney*
mel scale (linear below 1 kHz) with *area-normalised* triangles. torchaudio's
`MelSpectrogram` defaults to HTK with peak-1 triangles, which is what you'll
build (it matches `librosa.filters.mel(..., htk=True, norm=None)`).
Features from the two libraries aren't interchangeable, even with the same
`n_mels`.

## 2. Rust for this lesson

The filterbank application is a matrix product. With iterators:

```rust
// out[frame][m] = Σ_bin fb[m][bin] · power[frame][bin]
power.iter().map(|frame|                 // for each frame
    fb.iter().map(|filter|               //   for each filter
        filter.iter().zip(frame)         //     pair up weights and powers
              .map(|(w, p)| w * p)
              .sum()                     //     and add them up
    ).collect()                          //   → one Vec per frame
).collect()                              // → Vec of Vecs
```

That's nothing new, just nesting. Read it from the inside out.

## 3. Predict first

1. With 40 mels from 0 to 8 kHz, roughly how wide (in Hz) are the first and
   last triangles?
2. At a frequency halfway between two triangle centres, what are the two weights?
3. What does MFCC 0 measure? What would make MFCC 1 change sign?
4. The utterance has an "ah" (formants at 730, 1090, 2440 Hz) and an "ee"
   (270, 2290, 3010 Hz). Where will their energy sit in the mel spectrogram?

## 4. Exercises

In `src/mel.rs`:

1. `hz_to_mel`, `mel_to_hz`
2. `mel_filterbank`
3. `apply_filterbank`
4. `mel_spectrogram`
5. `dct2`
6. `mfcc`

<details><summary>Hint: mel_filterbank</summary>

1. Convert fmin and fmax to mel.
2. Make `n_mels + 2` evenly spaced mel values between them and convert each
   back to Hz. These are the triangle edges.
3. Get each bin's frequency with your `bin_frequencies` from lesson 11.
4. Filter m uses edges m, m+1, m+2 as (lo, centre, hi). Its weight at
   frequency f is `min(rising, falling)` clamped at 0.

</details>

## 5. Run it

```bash
cargo run --example lesson12
```

- `out/lesson12_filterbank.svg`: the triangles.
- `out/lesson12_mel.svg`: compare "ah" and "ee". Formants are the dark bands.
- `out/lesson12_mfcc.svg`: harder to read by eye. That's the point: it was made for models.

On your own data:

```bash
cargo run --example lesson12 -- C:\path\to\recording.wav
```

## 6. Check your understanding

1. A model trained on 16 kHz log-mels (fmax 8 kHz) is given telephone audio
   (band-limited to 3.4 kHz) upsampled to 16 kHz. What does it see?
2. Jitter is cycle-to-cycle variation in pitch. How much of it survives into
   13 MFCCs? What does that suggest about MFCC-only dysphonia detectors?
3. Why can't you directly compare MFCCs computed by librosa with ones from
   another toolkit, even with the same n_mfcc?

<details><summary>Answers</summary>

1. The mel bands above 3.4 kHz are empty: a near-silent stripe across the
   top it never saw in training. That's a classic domain shift.
2. Very little. Frame-level smoothing (25 ms frames span several cycles),
   wide upper filters and DCT truncation all blur fine pitch structure. That's
   why studies add jitter, shimmer and HNR as separate features, or use
   higher-resolution spectrograms.
3. Different mel formula (HTK or Slaney), filter normalisation, log
   (10·log10 with top_db, or ln), liftering, and DCT scaling all change the
   numbers.

</details>

Next: [Lesson 13, pitch tracking](../13-pitch/README.md)

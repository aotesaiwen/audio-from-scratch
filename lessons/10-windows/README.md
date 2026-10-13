# Lesson 10: Windows and spectral leakage

> **Audio:** leakage, Hann and Hamming windows, the resolution–leakage trade-off · **Rust:** `max_by` and `total_cmp`, `saturating_sub`
> **You edit:** `src/window.rs` · **Check:** `cargo test l10` · **Demo:** `cargo run --example lesson10` · **Time:** ~1 h

`window='hann'` is the default in almost every STFT call. This lesson shows
what it does and why leaving it out is a mistake.

## 1. The idea

The DFT quietly assumes the frame repeats forever. If the frame holds a
whole number of cycles, the repeats join up smoothly. If not, there's a jump
at every seam:

```
whole number of cycles:   /\/\/\/\|/\/\/\/\|/\/\/\/\     seamless
not a whole number:       /\/\/\/ |/\/\/\/ |/\/\/\/      a jump at each |
```

A jump contains *every* frequency, so the energy of one pure tone smears
across the whole spectrum. That's **spectral leakage**. It's why lesson 08's
554.37 Hz note came out at the wrong height: it fell between bins.

**Windows** fix the seam by fading the frame to zero at both ends before the
FFT, so the repeats always meet at zero. The cost:

- The peak gets **wider** (worse at separating two very close tones).
- The peak gets **shorter**. The amplitude is scaled by the window's mean,
  called its *coherent gain*: 0.5 for Hann.

| Window | Formula (periodic, i = 0..n−1) | Nearest side lobe | Far side lobes |
|---|---|---|---|
| rectangular | 1 | −13 dB | fall slowly |
| Hann | 0.5 − 0.5·cos(2πi/n) | −31 dB | fall fast (−18 dB/octave) |
| Hamming | 0.54 − 0.46·cos(2πi/n) | −43 dB | fall slowly |

Hann reaches exactly zero at the edges, so its leakage dies away quickly.
Hamming cancels the *nearest* side lobe better but leaves a floor. Speech
tools default to Hann (librosa) or Hamming (HTK, Kaldi, older MFCC recipes).

"Periodic" divides by n instead of n − 1. That makes the window the right
shape for spectral analysis, and it's what `scipy.signal.get_window('hann', n)`
returns.

## 2. Rust for this lesson

**Finding the biggest float.** `f64` isn't fully ordered, because NaN isn't
bigger or smaller than anything, so `.max()` doesn't work on an iterator of
floats. Use `max_by` with `total_cmp`, which puts every float (even NaN) in a
fixed order:

```rust
let (best_i, best_v) = xs.iter().copied().enumerate()
    .max_by(|a, b| a.1.total_cmp(&b.1))
    .unwrap();        // None only if xs is empty
```

`a.1` is the second element of the tuple `(index, value)`.

**`saturating_sub`**: `3usize.saturating_sub(5)` is 0 instead of a panic.
Handy for "the bin `guard` places to the left, but not below 0".

## 3. Predict first

1. 1000 Hz at 8 kHz with N = 1024: how many cycles fit in the frame? Will it
   leak?
2. Same for 1003.9 Hz.
3. The demo's second plot has a loud tone and a quiet one, 50 dB softer and
   500 Hz away. Will you see the quiet one without a window? With Hann?

## 4. Exercises

In `src/window.rs`:

1. `hann`, `hamming`
2. `apply_window`
3. `coherent_gain`
4. `max_sidelobe_db`

## 5. Run it

```bash
cargo run --example lesson10
```

- `out/lesson10_leakage.svg`: one off-bin tone through three windows.
- `out/lesson10_hidden_tone.svg`: find the quiet tone.

## 6. Check your understanding

1. Why do harmonics in a speech spectrogram look crisper with a Hann window?
2. When would you choose a rectangular window anyway?
3. The demo scales each spectrum by `2 / (N · coherent_gain)`. What does that
   achieve?

<details><summary>Answers</summary>

1. Each harmonic's leakage would otherwise spill across the gaps between
   harmonics and fill in the valleys.
2. When the signal *does* fit a whole number of times (synthetic tests), or
   for transients that start and end inside the frame. Then no seam exists,
   and you get the narrowest possible peak.
3. A sine of amplitude A on a bin gives a peak of A·N/2 × coherent gain.
   Dividing by that makes the peak read 0 dB for a full-scale sine whatever
   the window, so the curves are comparable.

</details>

Next: [Lesson 11, the STFT](../11-stft/README.md)

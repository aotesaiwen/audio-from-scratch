# Lesson 03: Loudness, decibels and bit depth

> **Audio:** peak, RMS, decibels, quantisation noise · **Rust:** iterators and closures
> **You edit:** `src/level.rs` · **Check:** `cargo test l03` · **Demo:** `cargo run --example lesson03` · **Time:** ~1.5 h

## 1. The idea

**Peak and RMS.** The *peak* is the biggest absolute sample: it tells you
how close you are to clipping. The *RMS* (root-mean-square) is the square
root of the average squared sample: it tells you how much energy there is,
and tracks how loud things sound much better than the peak.

    RMS = √( (x[0]² + x[1]² + … + x[N−1]²) / N )

A sine of amplitude A has RMS A/√2 ≈ 0.707·A. A square wave of amplitude A
has RMS A.

**Decibels** describe ratios on a log scale, because hearing works in
ratios: going from 0.01 to 0.02 sounds like the same step as 0.1 to 0.2.

    amplitude ratio → dB:   20 · log10(a / reference)
    power ratio     → dB:   10 · log10(p / reference)

Power is amplitude squared, so both formulas give the *same* dB for the same
signal. That's why librosa has both `amplitude_to_db` and `power_to_db`.
Numbers worth memorising:

| Amplitude × | dB |
|---|---|
| 2 | +6.02 |
| 1/2 | −6.02 |
| 10 | +20 |
| 1/10 | −20 |
| 1/1000 | −60 |

**dBFS** (dB relative to full scale) uses 1.0 as the reference, so every
sample is at or below 0 dBFS.

**Quantisation.** Storing a sample in `bits` bits leaves 2^bits levels.
Rounding to the nearest level makes a small error, at most half a step,
which behaves like added noise. Each extra bit halves the step size, and
halving the noise amplitude is −6.02 dB. For a full-scale sine:

    SNR ≈ 6.02 · bits + 1.76 dB

(The 1.76 comes from comparing a sine's power, 1/2, to the power of noise
spread uniformly over one step, step²/12.) 16-bit audio gives about 98 dB of
dynamic range, more than enough for speech.

## 2. Rust for this lesson: iterators

Iterators let you describe *what* to do with every element without managing
indices. `peak` in `src/level.rs` is written for you as a worked example:

```rust
x.iter()                 // walk the values (each is a &f64)
 .map(|v| v.abs())       // transform each one (|v| ... is a closure)
 .fold(0.0, f64::max)    // combine them into one value, starting from 0.0
```

Another parallel example, the mean:

```rust
x.iter().sum::<f64>() / x.len() as f64
```

The tools you'll need:

| Method | Does |
|---|---|
| `.iter()` | start walking a slice |
| `.map(\|v\| ...)` | transform each item |
| `.sum::<f64>()` | add everything up (`::<f64>` says what type the total is) |
| `.fold(start, \|acc, v\| ...)` | general "combine everything into one value" |
| `.zip(other)` | walk two things side by side as pairs `(a, b)` |
| `.collect()` | gather the results into a `Vec` |

Iterators are **lazy**: nothing runs until something like `sum`, `fold` or
`collect` asks for the values. They compile to the same speed as hand-written
loops, so use whichever reads better.

A closure that receives a reference sometimes needs a pattern to get the
value out: `.map(|&v| v * gain)` or `.map(|v| *v * gain)` do the same thing.

## 3. Predict first

1. RMS of `[1, −1, 1, −1]`? Of a sine with amplitude 1?
2. Halving an amplitude changes it by how many dB? Halving a power?
3. Predict the SNR of 8-bit audio.
4. The demo quantises a *chord* instead of a sine. Will its SNR be higher or
   lower than the formula?

## 4. Exercises

In `src/level.rs` (`peak` is already done):

1. `rms`
2. `amplitude_to_db`, `db_to_amplitude`
3. `normalize_peak`
4. `quantize`
5. `snr_db`

Try to write each as an iterator chain.

<details><summary>Hint: quantize</summary>

With `levels = 2^(bits−1)`: multiply the sample by `levels`, round it,
clamp to `[−levels, levels − 1]`, divide by `levels`. For 2 bits that gives
exactly four possible values: −1, −0.5, 0, 0.5.

</details>

## 5. Run it

```bash
cargo run --example lesson03
```

It prints measured SNR against the formula and writes the chord at 16, 12,
8, 6, 4, 3 and 2 bits. Listen from 16 down to 2. Where do you first hear a
difference? Open `out/lesson03_quantization.svg` to see the staircase.

## 6. Check your understanding

1. Why is the chord's SNR about 6 dB below the formula at every bit depth?
2. A recording's loudest moment is −20 dBFS. Roughly how many of its 16 bits
   is it really using?
3. At 2–3 bits the noise doesn't sound like a steady hiss. Why?

<details><summary>Answers</summary>

1. The formula assumes a full-scale sine, whose RMS is −3 dBFS. The chord
   was normalised to a −1 dBFS *peak*, but three sines that rarely peak
   together have a much lower RMS (the demo prints −8.8 dBFS). That's ~5.8 dB
   less signal against the same quantisation noise.
2. 20 dB ÷ 6.02 dB per bit ≈ 3.3 bits wasted, so about 12.7 effective bits.
   Recording too quietly costs resolution.
3. With so few levels, the rounding error follows the signal's shape. It
   repeats with the waveform and sounds like harsh distortion, not random
   noise. (Adding a little noise *before* quantising, called dither, turns it
   back into a benign hiss.)

</details>

Next: [Lesson 04, aliasing](../04-aliasing/README.md)

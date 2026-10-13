# Lesson 04: Nyquist and aliasing

> **Audio:** the sampling theorem, folding, band-limited synthesis · **Rust:** `if` as an expression, `while`, `windows`, `iter_mut`
> **You edit:** `src/aliasing.rs` · **Check:** `cargo test l04` · **Demo:** `cargo run --example lesson04` · **Time:** ~1.5 h

This lesson explains why every resampler has a low-pass filter inside it.

## 1. The idea

Back in lesson 01's "predict first", `sine(8000.0, 1.0, 16_000, ...)` gave all
zeros. Here's why. Sample n of a sine at frequency `sr − f` is

    sin(2π·(sr − f)·n/sr) = sin(2π·n − 2π·f·n/sr) = −sin(2π·f·n/sr)

Because n is a whole number, the `2π·n` part is a whole number of turns, and it
disappears. So **the samples of a sine at sr − f are exactly the samples of a
sine at f, flipped.** Once sampled, the two are indistinguishable. The same
goes for f + sr, f + 2·sr, and so on. Those impostor frequencies are
**aliases**.

The only frequencies that are unambiguous lie between 0 and **half the
sample rate**, the **Nyquist frequency**. Anything above it *folds back*:

```
frequency you get
sr/2 |       /\              /\
     |      /  \            /  \
     |     /    \          /    \
     |    /      \        /      \
     |   /        \      /        \
     |  /          \    /          \
     | /            \  /            \
     |/              \/              \
   0 +--------+-------+-------+-------+--> frequency you recorded
     0      sr/2     sr     3sr/2    2sr
```

That gives three practical rules:

1. **Before sampling**, an analogue filter removes everything above Nyquist.
   (Your microphone interface does this.)
2. **Before downsampling** digitally (48 kHz → 16 kHz), remove everything above
   the *new* Nyquist (8 kHz) first, or it folds into the speech band. That's
   lesson 15.
3. **When synthesising** sharp waveforms, sharp corners contain energy at
   very high frequencies, which folds back as out-of-tune junk.

**Square waves** show rule 3 clearly. A square wave is a sum of odd
harmonics, at f, 3f, 5f, ..., with amplitudes 4/(π·k). Draw it naively (+1 or
−1 on each sample) and you've implicitly included *all* of those harmonics,
most of them above Nyquist, all folding back. Add up only the harmonics below
Nyquist and nothing folds.

**Zero crossings** give a crude frequency meter. A sine crosses zero twice
per cycle, so frequency ≈ crossings / (2 · duration). It only works for
single sines, but it's enough to *measure* aliasing here, and lesson 05
reuses it.

## 2. Rust for this lesson

```rust
// `if` is an expression, so it produces a value:
let label = if x > 0.0 { "positive" } else { "not positive" };

// `%` keeps the sign of the left side; `rem_euclid` always gives 0..b:
(-1.0f64) % 8.0;            // -1.0
(-1.0f64).rem_euclid(8.0);  //  7.0

// Overlapping pairs:
let xs = [1.0, 5.0, 2.0];
for w in xs.windows(2) { /* w = [1.0, 5.0], then [5.0, 2.0] */ }

// Count with a filter (compare with your lesson 00 loop):
let n = xs.iter().filter(|&&x| x > 1.5).count(); // 2
```

The `|&&x|` pattern unpacks two layers of reference: the iterator gives you a
`&f64`, and `filter` hands its closure a reference *to that*.

To change values in place: `for (o, v) in out.iter_mut().zip(&h) { *o += v; }`.
`*o` means "the thing `o` points at".

## 3. Predict first

1. At sr = 8 kHz, what frequency do you actually get for a 5 kHz sine? 7.5 kHz? 12 kHz?
2. The test `l04_a_sine_and_its_mirror_are_indistinguishable` passes before
   you write anything. Explain why, using the equation above.
3. The demo sweeps a sine from 0 to 8 kHz with sr = 8 kHz. **Sketch the
   frequency you'll hear over time** before running it.
4. How many odd harmonics of 1568 Hz fit below 8 kHz?

## 4. Exercises

In `src/aliasing.rs`:

1. `nyquist`
2. `alias_frequency`
3. `zero_crossings`, then `estimate_frequency`
4. `naive_square`
5. `odd_harmonics_below_nyquist`, then `square_from_harmonics`

<details><summary>Hint: alias_frequency</summary>

Two steps. First wrap: `f = freq.abs().rem_euclid(sr)` puts it in
[0, sr). Then fold: if it's above sr/2, it shows up at `sr − f`.

</details>

## 5. Run it

```bash
cargo run --example lesson04
```

- Listen to `out/lesson04_sweep.wav` and compare with your sketch. Then open
  `out/lesson04_sweep.svg`.
- Listen to `lesson04_square_naive.wav` then `lesson04_square_bandlimited.wav`.
  The naive one has extra, unrelated pitches whining underneath. Those are
  the aliases.

## 6. Check your understanding

1. A recording made at 48 kHz is resampled to 16 kHz. What must happen to
   the signal *before* samples are dropped, and what would go wrong otherwise?
2. A 16 kHz recording has nothing above 8 kHz. Which sounds lose the most?
3. Why does the zero-crossing frequency estimate fail on a real voice?

<details><summary>Answers</summary>

1. A low-pass filter has to remove everything above 8 kHz. Otherwise energy
   between 8 and 24 kHz (breath noise, fricatives) folds down into 0–8 kHz,
   where it masquerades as speech-band content.
2. Sounds with energy up high: "s" and "sh", breath noise, cymbals and other
   bright percussion. Pitch and vowel formants sit well below 8 kHz, so
   speech stays clear, but it sounds duller and music loses its sparkle.
3. A voice has many harmonics and formants, so the waveform wiggles across
   zero several times per cycle. Crossings measure the dominant *formant*
   region more than F0. Lesson 13 uses a better method.

</details>

Next: [Lesson 05, frames](../05-frames/README.md)

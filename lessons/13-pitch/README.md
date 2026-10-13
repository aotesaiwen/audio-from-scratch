# Lesson 13: Pitch tracking

> **Audio:** F0, autocorrelation, voicing, octave errors, sub-sample precision, the missing fundamental · **Rust:** `Option`, `?` on Options, `Option::map`, `flatten`
> **You edit:** `src/pitch.rs` · **Check:** `cargo test l13` · **Demo:** `cargo run --example lesson13` · **Time:** ~2 h

In Praat (or parselmouth), pitch analysis is usually
`call(sound, "To Pitch", 0.0, f0min, f0max)` followed by `"Get mean"` and
`"Get standard deviation"`. This lesson builds the same kind of pitch tracker.

## 1. The idea

**F0** is how many times per second the vocal folds open and close. Each
cycle produces roughly the same waveform shape, so a voiced signal (nearly)
**repeats itself** every T = 1/F0 seconds.

**Autocorrelation** finds that repetition. Slide a copy of the frame past
itself and measure how well they line up at each lag τ:

    r[τ] = Σ_n x[n] · x[n + τ]

```
lag 0:      /\/\  /\/\  /\/\        perfect match (r is largest)
            /\/\  /\/\  /\/\
lag T/2:    /\/\  /\/\  /\/\        out of step (r is low or negative)
               /\/\  /\/\  /\/\
lag T:      /\/\  /\/\  /\/\        lined up again: a peak at the period
                  /\/\  /\/\  /\/\
```

**Normalise it.** Raw r shrinks at long lags (fewer samples overlap) and
scales with loudness. Divide by √(energy of each overlapping part) and you
get a value in [−1, 1], where 1 means a perfect repeat. That number is also
a **voicing** measure: near 1 for vowels, low for noise and silence.

**Searching.** Only look at lags between sr/fmax and sr/fmin, Praat's pitch
"floor" and "ceiling". The frame must hold a few periods of the *lowest*
pitch, so Praat uses 3/fmin = 40 ms for a 75 Hz floor.

**Octave errors.** A signal that repeats every T also repeats every 2T, 3T,
... Pick the global maximum and noise may hand you 2T, which reports half
the pitch. So take the **first** lag that's a local peak *and* nearly as
high as the best. (Praat goes further: it keeps several candidates per frame
and picks the smoothest path through time.)

**Sub-sample precision.** Lags are whole samples. At 16 kHz, a 150 Hz voice
has T ≈ 106.7 samples, and lag 106 vs 107 means 150.9 vs 149.5 Hz: 1.4 Hz
steps, far too coarse for jitter, which is about 0.5%. Fit a parabola
through the peak and its two neighbours and use the parabola's top.

**The missing fundamental.** Play 400 + 600 + 800 Hz. There's no energy at
200 Hz, yet you hear a 200 Hz pitch. The waveform repeats every 5 ms, and
autocorrelation agrees with your ear. Pitch is about *periodicity*, not the
loudest frequency. (A telephone removes everything below 300 Hz, but a
110 Hz voice still sounds like 110 Hz.)

## 2. Rust for this lesson

`Option<T>` is `Some(value)` or `None`, Rust's "maybe". Three ways to work
with it:

```rust
fn first_negative(xs: &[f64]) -> Option<usize> {
    xs.iter().position(|&x| x < 0.0)            // Some(index) or None
}

fn half_of_first_negative(xs: &[f64]) -> Option<f64> {
    let i = first_negative(xs)?;                 // `?`: if None, return None now
    Some(xs[i] / 2.0)
}

first_negative(xs).map(|i| i * 2);               // transform what's inside a Some

let voiced: Vec<f64> = track.iter().flatten().copied().collect(); // drop Nones, unwrap Somes
```

`(a..=b).find(|&lag| ...)` returns the first lag that passes a test, as an Option.

## 3. Predict first

1. At 16 kHz, which lag (in samples) is 200 Hz? 75 Hz? 600 Hz?
2. For a 150 Hz voice, where will r have peaks? Where will it be most negative?
3. What F0 will your tracker report for 400 + 600 + 800 Hz?
4. You set fmin = 200 Hz on a man's voice at 110 Hz. What might happen?

## 4. Exercises

In `src/pitch.rs`:

1. `autocorrelation`
2. `normalized_autocorrelation`
3. `parabolic_peak`
4. `periodicity` (the core; follow the numbered steps in its doc comment)
5. `estimate_f0`
6. `pitch_track`
7. `mean_and_std`

<details><summary>Hint: periodicity, the first-peak rule</summary>

```rust
let lag = (min_lag..=max_lag)
    .find(|&l| /* r[l] is high enough AND r[l] >= both neighbours */)?;
```

Your `r` must go one lag past `max_lag` so `r[l + 1]` exists. That's why the
doc says to compute it "up to the largest lag + 1". And `min_lag` is at least
1, so `r[l − 1]` exists too.

</details>

## 5. Run it

```bash
cargo run --example lesson13
```

- Listen to `out/lesson13_do_mi_sol_do.wav`, then open `out/lesson13_pitch.svg`.
- `out/lesson13_autocorrelation.svg` shows what the algorithm "sees" for one
  frame. Find the peak it picked.

Then a recording of your own:

```bash
cargo run --example lesson13 -- C:\path\to\sustained_a.wav
```

If the track jumps to half or double now and then, those are octave errors.

## 6. Check your understanding

1. Why must the frame span ~3 periods of fmin rather than one?
2. Feature scripts often set `f0min`/`f0max` per speaker sex. What goes wrong
   with one wide range (say 50–800 Hz) for everyone?
3. Why does Praat's "Get standard deviation" of F0 on a sustained vowel
   partly measure the same thing as jitter, and how is it different?

<details><summary>Answers</summary>

1. To see a repeat at lag T you need at least two whole cycles in the frame,
   and more cycles make the peak more reliable. With 1–2 cycles, a single
   irregular cycle ruins the estimate.
2. More octave errors. A wide range lets the search find 2T (halving) or
   T/2 (doubling) peaks it would otherwise never look at. It also needs
   longer frames (3/50 Hz = 60 ms), which blur fast changes.
3. Both measure F0 instability. The F0 standard deviation is over
   10 ms-spaced frames, so it captures slow drift and tremor. Jitter compares
   *consecutive cycles*, so it captures fast, cycle-to-cycle irregularity. A
   voice can drift a lot with low jitter, or the reverse.

</details>

Next: [Lesson 14, jitter, shimmer and HNR](../14-voice-quality/README.md)

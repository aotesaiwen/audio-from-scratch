# Lesson 14: Jitter, shimmer and HNR

> **Audio:** glottal cycles, perturbation measures, harmonics-to-noise ratio · **Rust:** `Display`, `writeln!`, `filter_map`, combining everything
> **You edit:** `src/voice.rs` · **Check:** `cargo test l14` · **Demo:** `cargo run --example lesson14` · **Time:** ~2–3 h

Voice-quality features are usually extracted with Praat calls like these:

```python
call(pointProcess, "Get jitter (local)", 0, 0, 0.0001, 0.02, 1.3)
call([sound, pointProcess], "Get shimmer (local)", 0, 0, 0.0001, 0.02, 1.3, 1.6)
call(sound, "To Harmonicity (cc)", 0.01, 75, 0.1, 1.0)
```

By the end you'll know what every one of those numbers does, because you'll
have implemented them.

## 1. The idea

A healthy sustained vowel is *nearly* periodic: each vocal-fold cycle is
almost the same length and the same strength, and almost all the sound
energy is in the harmonics. Disordered voices break this in three
measurable ways:

| Measure | What varies | Heard as |
|---|---|---|
| **jitter** | cycle *length*, from one cycle to the next | roughness, hoarseness |
| **shimmer** | cycle *amplitude*, from one cycle to the next | roughness |
| **HNR** (low) | energy that isn't periodic at all | breathiness, hoarseness |

**Step 1: find every cycle.** Praat's `To PointProcess (periodic, cc)`
places one mark per glottal cycle. You'll do a simpler version: knowing F0
roughly (lesson 13), find the biggest peak in each cycle by searching a
window from 0.7·T to 1.3·T after the previous peak, refined with lesson 13's
`parabolic_peak`.

**Step 2: the measures** (definitions from the Praat manual):

- **Periods** T₁, T₂, ...: time between consecutive peaks. **Amplitudes**
  A₁, A₂, ...: peak height per cycle.
- **Jitter (local)** = mean |Tᵢ − Tᵢ₊₁| / mean T. Report it ×100 as %.
- **Jitter (local, absolute)** = mean |Tᵢ − Tᵢ₊₁| in seconds (Praat shows µs).
- **Jitter (RAP)** = mean |Tᵢ − (Tᵢ₋₁ + Tᵢ + Tᵢ₊₁)/3| / mean T: each period
  against the average of its neighbourhood, so slow drift cancels out.
- **Jitter (ddp)** = 3 × RAP, *exactly*. Same information, different number.
- **Shimmer (local)** = mean |Aᵢ − Aᵢ₊₁| / mean A.
- **Shimmer (local, dB)** = mean |20·log10(Aᵢ₊₁ / Aᵢ)|.
- **Shimmer (APQ3)**: RAP's idea applied to amplitudes. **dda** = 3 × APQ3.

**Those mystery arguments**, in `"Get jitter (local)", 0, 0, 0.0001, 0.02, 1.3`:

| Value | Meaning |
|---|---|
| 0, 0 | time range: 0 to 0 means "the whole sound" |
| 0.0001 | shortest period to trust (0.1 ms) |
| 0.02 | longest period to trust (20 ms, so F0 below 50 Hz is treated as a voice break) |
| 1.3 | maximum period factor: neighbouring periods differing by more than ×1.3 aren't compared (probably a tracking error) |
| 1.6 | (shimmer only) maximum amplitude factor, the same idea for amplitudes |

**HNR.** In lesson 13, the normalised autocorrelation peak r said how
periodic a frame is. If a fraction r of the energy repeats from cycle to
cycle and 1 − r doesn't (Boersma, 1993):

    HNR = 10 · log10( r / (1 − r) )   dB

r = 0.99 gives 20 dB, and r = 0.9 gives 9.5 dB. Average over voiced frames,
as Praat's "Get mean" does.

**A warning you'll see for yourself.** The demo's "healthy" voice was
*built* with 0.3% jitter, but you'll measure more than that. Additive noise
nudges each peak's position, and that shows up as extra jitter. Praat's
cross-correlation method is more robust than peak picking, but no method is
immune. So **recording conditions leak into perturbation measures.** If one
hospital's microphones are noisier, its patients' jitter is inflated, and a
classifier can learn "which hospital" instead of "which pathology". That's
one concrete mechanism behind dataset bias in voice-disorder detection.

## 2. Rust for this lesson

**`Display`** controls how `{}` prints your type:

```rust
use std::fmt;

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "x = {:.2}", self.x)?;   // like println!, but into f
        write!(f, "y = {:.2}", self.y)        // last line: no `?`, return its Result
    }
}
println!("{point}");                          // uses your fmt
```

`{:8.3}` means "8 characters wide, 3 decimals". That's how the report lines up.

**`filter_map`** = filter + map in one: keep the `Some`s, unwrap them.

```rust
frames.iter().filter_map(|f| periodicity(f, sr, cfg))   // only voiced frames
```

**`?` inside a struct literal** works too. Any field that comes out `None`
makes the whole function return `None`:

```rust
Some(VoiceReport { jitter_local: jitter_local(&t, &limits)?, /* ... */ })
```

## 3. Predict first

1. Periods 10, 11, 10, 11 ms: what's jitter (local)?
2. Amplitudes 1.0, 0.8, 1.0, 0.8: shimmer (local)? Shimmer (dB)?
3. Periods 10 ms and 14 ms side by side: does Praat compare them?
4. A common feature vector includes localJitter, rapJitter *and* ddpJitter.
   How many independent numbers is that, really?

## 4. Exercises

In `src/voice.rs`:

1. `argmax`
2. `find_cycle_peaks`
3. `periods`, `amplitudes`
4. `jitter_local_absolute`, `jitter_local`, `jitter_rap` (the helpers
   `PeriodLimits::is_valid` and `pair_ok` are given)
5. `shimmer_local`, `shimmer_local_db`, `shimmer_apq3`
6. `hnr_db`
7. `Display for VoiceReport`
8. `analyze`, which puts it all together

<details><summary>Hint: find_cycle_peaks' loop</summary>

```
i = index of first peak (argmax of the first period, then walk uphill)
loop:
    record parabolic_peak(x, i)
    lo = i + 0.7·T, hi = i + 1.3·T   (as usize; hi rounded up)
    if hi is past the end: stop
    j = argmax of x[lo..=hi]
    if j is 0 or hi − lo (the window's edge): stop
    i = lo + j
```

</details>

## 5. Run it

```bash
cargo run --example lesson14
```

It prints reports for a "healthy-ish" and a "rough" synthetic voice, writes
both as WAVs (listen!), and plots their cycle lengths.

**Now real data.** Take a recording of a sustained /a/:

```bash
cargo run --example lesson14 -- C:\path\to\sustained_a.wav
```

Then get Praat's numbers for the same file:

```bash
python tools/praat_reference.py C:\path\to\sustained_a.wav
```

They *won't* match exactly. Try to explain each difference. Some causes:
Praat finds cycles by waveform cross-correlation rather than peak picking;
its HNR uses one-period windows (that `1.0` argument) with a different
normalisation; and its pitch tracker uses path-finding. If yours is far off,
look for octave errors in your pitch track first.

## 6. Check your understanding

1. Explain every argument of `"Get shimmer (local)", 0, 0, 0.0001, 0.02, 1.3, 1.6`.
2. Why does RAP react less than local jitter to a slow, steady pitch glide?
3. What would you do to stop a classifier learning recording conditions
   through jitter and HNR?

<details><summary>Answers</summary>

1. Whole time range (0, 0); shortest and longest trusted period (0.1 ms,
   20 ms); maximum ratio between neighbouring periods (1.3); maximum ratio
   between neighbouring amplitudes (1.6).
2. In a steady glide, each period is very close to the average of itself and
   its two neighbours, so RAP's deviations stay tiny. Local jitter still sees
   every consecutive difference.
3. Some options: check feature distributions per site and per microphone
   before training; match recording conditions or add matched noise; report
   results per site; use measures known to be robust to noise; and test
   whether a model can predict the *site* from the features. If it can,
   worry.

</details>

**That's the core course.** Lessons 15 and 16 are bonuses: resampling, and
the McAdams voice anonymiser.

Next: [Lesson 15, resampling](../15-resampling/README.md)

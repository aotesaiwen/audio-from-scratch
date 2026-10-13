# Lesson 01: Sound is a list of numbers

> **Audio:** sampling, sample rate, sine waves · **Rust:** `Vec`, `for`, `as` casts
> **You edit:** `src/signal.rs` · **Check:** `cargo test l01` · **Demo:** `cargo run --example lesson01` · **Time:** ~1 h

`librosa.load(path, sr=16000)` returns a float array and a number. This
lesson is about what that array *is*.

## 1. The idea

Sound is air pressure going up and down. A microphone turns pressure into a
voltage, and an analogue-to-digital converter measures that voltage at
regular instants: 16,000 times a second at a **sample rate** of 16 kHz.
Each measurement is a **sample**. A recording is just the list of samples
plus the rate they were taken at.

```
pressure
   ^      .-'''-.                 .-'''-.
   |    o'       'o             o'       '.
   |  o'           'o         o'           o
---+-o---------------o-------o---------------o----> time
   |                  'o   o'
   |                    '-'
     ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^ ^
     samples, every 1/sample_rate seconds
```

Sample `n` is taken at time **t = n / sample_rate**.

Common sample rates: 8 kHz (telephone), 16 kHz (most speech ML), 44.1 kHz
(CDs), 48 kHz (video and most audio interfaces).

**The sine wave** is the simplest sound and the building block of all the
others (lesson 08 proves that). It has a frequency `f` (cycles per second, Hz)
and an amplitude `A` (how far it swings):

    x(t) = A · sin(2π · f · t)

Sampled, that becomes

    x[n] = A · sin(2π · f · n / sample_rate)

One cycle lasts 1/f seconds, which is `sample_rate / f` samples. That's
usually not a whole number. At 8 kHz, a 440 Hz cycle is 18.18 samples long,
so the samples land on slightly different parts of each cycle.

**Adding sounds is adding samples.** Two instruments playing together are, to
the microphone, the sum of their pressure waves. A chord is a sum of sines.

## 2. Rust for this lesson

A parallel example, *not* one of your exercises:

```rust
/// A straight line from 0 up towards `top`, `n` samples long.
pub fn ramp(top: f64, n: usize) -> Vec<f64> {
    let mut out = Vec::with_capacity(n); // empty, with room for n values
    for i in 0..n {
        out.push(top * i as f64 / n as f64);
    }
    out // the last expression is the return value
}
```

- `Vec<f64>` is a growable list you own. `Vec::with_capacity(n)` reserves
  space up front, and `vec![0.0; n]` makes n zeros.
- `i as f64` converts an integer to a float. Rust never converts number types
  for you, so every `as` is deliberate.
- In `top * i as f64 / n as f64`, `as` binds tighter than `*` and `/`, so this
  is `top * (i as f64) / (n as f64)`.
- `&[f64]` (in `chord`) is a borrowed, read-only list. You can loop over it
  with `for &f in freqs`.
- `std::f64::consts::PI` is π. It's already imported at the top of `signal.rs`.

## 3. Predict first

1. How many samples are in 1 second at 16 kHz? In 25 ms?
2. How many samples per cycle does 440 Hz get at 8 kHz? At 44.1 kHz?
3. What four values repeat in `sine(4000.0, 1.0, 16_000, ...)`?
4. What would `sine(8000.0, 1.0, 16_000, ...)` give? Work out the first few
   samples by hand. (Lesson 04 explains it.)

## 4. Exercises

In `src/signal.rs`:

1. `num_samples`
2. `time_axis`
3. `samples_per_cycle`
4. `sine`
5. `chord`

<details><summary>Hint: num_samples</summary>

`sample_rate` is a `u32` and `seconds` an `f64`, and Rust won't multiply them
together. Convert the u32 to f64, multiply, `.round()`, then `as usize`.

</details>

<details><summary>Hint: sine</summary>

Inside the loop, first work out the time of sample `i`, then the value at
that time. Writing `let t = ...;` on its own line keeps it readable.

</details>

<details><summary>Hint: chord</summary>

Start with `let mut out = vec![0.0; num_samples(sample_rate, seconds)];`.
For each frequency, make a sine with your own `sine` function and add it in
sample by sample.

</details>

## 5. Run it

```bash
cargo run --example lesson01
```

Open `out/lesson01_sine.svg` in a browser. The blue curve stands in for the
continuous wave, and the orange dots are what a computer at 8 kHz stores.

- Do the dots land at the same place on every cycle? Why not?
- Now open `out/lesson01_chord.svg`. Can you see three sines in it? (Hard,
  right? Lesson 08 builds the tool that can.)

## 6. Check your understanding

1. Speech ML often uses 16 kHz but music uses 44.1 kHz. What do you think
   limits the highest frequency a sample rate can represent?
2. Doubling the amplitude changes the samples how? Doubling the frequency?
3. A 2-second recording at 44.1 kHz is resampled to 16 kHz. How many samples
   before and after?

<details><summary>Answers</summary>

1. You need at least two samples per cycle to capture an up-and-down at all,
   so the highest representable frequency is half the sample rate. That's
   lesson 04. Speech intelligibility mostly lives below ~4–8 kHz, music
   reaches the limit of hearing (~20 kHz).
2. Doubling the amplitude doubles every sample. Doubling the frequency halves
   the number of samples per cycle: the wave is squeezed in time.
3. 88,200 before, 32,000 after.

</details>

Next: [Lesson 02, WAV files](../02-wav-files/README.md)

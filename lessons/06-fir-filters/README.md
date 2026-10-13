# Lesson 06: Convolution and FIR filters

> **Audio:** impulse responses, convolution, smoothing, pre-emphasis, echo, measuring frequency response · **Rust:** indexing, `usize` underflow, closures as arguments
> **You edit:** `src/fir.rs` · **Check:** `cargo test l06` · **Demo:** `cargo run --example lesson06` · **Time:** ~1.5 h

## 1. The idea

Many audio processes (a room, a microphone, an equaliser, a telephone line)
are **linear and time-invariant**. Twice the input gives twice the output,
and the same input later gives the same output later. Any such system is
completely described by one recording: its **impulse response** `h`, the
output when you feed in a single click `[1, 0, 0, ...]`.

Why is that enough? Because any signal is a row of scaled, delayed clicks.
Each input sample launches its own scaled, delayed copy of `h`, and the
output is the sum of them all. That's **convolution**:

```
x = [1, 2, 3]       h = [0, 1, 0.5]

x[0]=1 launches   0   1   0.5
x[1]=2 launches       0   2   1
x[2]=3 launches           0   3   1.5
                 ---------------------
y              = 0   1   2.5  4   1.5
```

Written as a formula, for each output sample:

    y[n] = Σ_k h[k] · x[n − k]

The output is `len(x) + len(h) − 1` long. A filter whose `h` is finite is a
**FIR** (finite impulse response) filter.

Some useful `h`:

| h | effect |
|---|---|
| `[1]` | nothing (identity) |
| `[0, 0, 1]` | delay by 2 samples |
| `[1/5; 5]` (moving average) | smoothing, which is a low-pass |
| `[1, −1]` (difference) | keeps only change, which is a high-pass |
| `[1, −0.97]` | **pre-emphasis**: gentler high-pass |
| `[1, 0, ..., 0, 0.5]` | one echo |
| a recording of a hall's response | convolution reverb |

**Pre-emphasis** is the first step of classic MFCC and LPC front ends. Voiced
speech loses energy steadily towards high frequencies (about −6 dB per
octave), so without it the low frequencies dominate every later analysis.

**Frequency response.** The gain of a filter depends on frequency. You'll
measure it by experiment: feed in a sine, compare output and input RMS,
repeat for many frequencies. (Lesson 08 shows how to get it directly.) The
moving average gives a surprise: it completely *cancels* any wave whose
period fits a whole number of times into the average.

## 2. Rust for this lesson

**Two ways to loop.** The formula `y[n] = Σ h[k]·x[n−k]` (gather) needs
`n − k`. With `usize`, `0 − 1` doesn't go negative. It **panics**: "attempt
to subtract with overflow". Either guard it (`if k <= n`), or turn the loop
around and scatter each input into the output, as in the diagram:

```
for each input index n:
    for each kernel index k:
        y[n + k] += x[n] * h[k]      // never negative
```

`.enumerate()` gives you `(index, value)` pairs:
`for (n, &xn) in x.iter().enumerate()`.

**Functions as arguments.** `measure_gain_db` takes *any* filter. The type
`impl Fn(&[f64]) -> Vec<f64>` means "something callable that takes a slice
and returns a Vec". A closure can **capture** variables from where it's
written. Here's a parallel example:

```rust
fn apply_twice(f: impl Fn(f64) -> f64, x: f64) -> f64 {
    f(f(x))
}

let gain = 3.0;
apply_twice(|v| v * gain, 1.0); // 9.0; the closure captured `gain`
```

So `measure_gain_db(|x| fir_filter(x, &h), 1000.0, 16_000)` passes a closure
that remembers `h`.

## 3. Predict first

1. Convolve `[1, 2, 3]` with `[0, 1, 0.5]` by hand. (Then check the table above.)
2. A 1 s signal at 16 kHz convolved with a 0.5 s room response: how long is
   the result?
3. A 5-tap moving average at 8 kHz: which frequencies does it cancel
   completely? (Hint: the period must be 5 samples, or 5/2...)
4. Pre-emphasis with α = 0.97: what does it do to a constant (DC) signal?

## 4. Exercises

In `src/fir.rs`:

1. `convolve`
2. `fir_filter`
3. `moving_average_kernel`, `echo_kernel`
4. `pre_emphasis` (one line, once you see what its `h` is)
5. `measure_gain_db`

<details><summary>Hint: measure_gain_db</summary>

Make a 0.5 s sine with your lesson 01 `sine`, run it through `filter(&x)`,
then compare `rms` of the second halves (`&y[half..end]` against
`&x[half..end]`) and convert the ratio with `amplitude_to_db`. Use the shorter
of the two lengths as `end`, in case a filter returns more samples.

</details>

## 5. Run it

```bash
cargo run --example lesson06
```

- `out/lesson06_response.svg` shows measured gain against frequency. Find
  the moving averages' nulls. Do they match your prediction?
- Listen to `lesson06_original.wav`, `_muffled`, `_pre_emphasis` and `_echo`.
  Try it on your own voice: `cargo run --example lesson06 -- my.wav`.

## 6. Check your understanding

1. Convolution is commutative (x∗h = h∗x). What does that mean physically?
2. Why does pre-emphasis use 0.97 rather than 1.0?
3. Direct convolution of N samples with an M-tap filter costs N·M
   multiplications. A 1-minute recording at 48 kHz through a 2-second reverb:
   how many? (Lesson 09 gives a faster way.)

<details><summary>Answers</summary>

1. Playing a recording through a room sounds the same as playing the room's
   impulse response "through" the recording. The roles of signal and system
   are interchangeable.
2. With α = 1, DC and very low frequencies are wiped out entirely. 0.97
   keeps a little low end (gain 0.03 at DC) while still tilting the spectrum.
3. 2,880,000 × 96,000 ≈ 2.8 × 10¹¹, minutes of CPU time. FFT-based
   convolution does the same job in well under a second.

</details>

Next: [Lesson 07, IIR filters](../07-iir-filters/README.md)

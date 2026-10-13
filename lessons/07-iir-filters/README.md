# Lesson 07: Feedback, IIR filters

> **Audio:** recursive filters, biquads, resonance, the source–filter model of speech · **Rust:** structs with state, `&mut self`, traits, `Box<dyn Trait>`
> **You edit:** `src/iir.rs` · **Check:** `cargo test l07` · **Demo:** `cargo run --example lesson07` · **Time:** ~2–3 h

## 1. The idea

An FIR filter only looks at past *inputs*. Let the filter also use its own
past *outputs* and you get **feedback**:

    y[n] = (1 − a)·x[n] + a·y[n−1]          (a one-pole low-pass)

Each output is mostly the previous output, nudged towards the new input. With
a = 0.99 it follows the input slowly, which smooths it. Feed in one click and
the output decays as aⁿ forever, never quite reaching zero. Its impulse
response is infinite, so this is an **IIR** filter.

- **Cheap**: a few multiplies per sample, versus hundreds of taps for an FIR
  doing a similar job.
- **Can be unstable**: if |a| ≥ 1 the feedback grows without bound. Try it.
- **Can resonate**: with two feedback terms the output can ring at a chosen
  frequency, like a bell.

**The biquad** has two past inputs and two past outputs:

    y[n] = b0·x[n] + b1·x[n−1] + b2·x[n−2] − a1·y[n−1] − a2·y[n−2]

Choosing the five coefficients gives low-pass, high-pass, band-pass, notch,
shelving and peaking filters. Almost every equaliser is built from biquads.
The standard recipes come from Robert Bristow-Johnson's *Audio EQ
Cookbook*:

    w0 = 2π · f0 / sample_rate
    α  = sin(w0) / (2·Q)

| | b0 | b1 | b2 | a0 | a1 | a2 |
|---|---|---|---|---|---|---|
| low-pass | (1 − cos w0)/2 | 1 − cos w0 | (1 − cos w0)/2 | 1 + α | −2 cos w0 | 1 − α |
| high-pass | (1 + cos w0)/2 | −(1 + cos w0) | (1 + cos w0)/2 | 1 + α | −2 cos w0 | 1 − α |
| band-pass (0 dB peak) | α | 0 | −α | 1 + α | −2 cos w0 | 1 − α |

Then divide everything by a0 (`from_coefficients` does that for you). **Q**
sets how sharp the corner or peak is. Q = 1/√2 ≈ 0.7071 gives the flattest
possible pass band (a "Butterworth" response), exactly −3 dB at f0.

**The source–filter model of speech.** Your voice is a buzzy **source**
(the vocal folds snapping shut ~100–250 times a second) passed through a
**filter** (the throat and mouth, whose resonances are the **formants**).
Changing the vowel changes the filter. Changing the pitch changes the
source. The demo builds an "ah" from a buzz and three band-pass filters.
This split runs through the rest of the course:

| measures the **source** | measures the **filter** |
|---|---|
| F0, jitter, shimmer, HNR (lessons 13–14) | formants, MFCCs, spectral envelope (lessons 12, 16) |

## 2. Rust for this lesson

**Structs that remember.** A filter needs memory between samples, so it's a
struct, and processing takes `&mut self` because it updates that memory:

```rust
pub struct OnePoleLowpass { a: f64, y1: f64 }

impl OnePoleLowpass {
    pub fn new(cutoff_hz: f64, sample_rate: u32) -> Self { ... } // a constructor
}
```

**Traits** describe what a type can do, so code can work with any type that
can do it. A parallel example:

```rust
trait Shape {
    fn area(&self) -> f64;
    fn describe(&self) -> String {               // a default method
        format!("a shape of area {:.1}", self.area())
    }
}

struct Square(f64);
impl Shape for Square {
    fn area(&self) -> f64 { self.0 * self.0 }   // describe() comes for free
}

let shapes: Vec<Box<dyn Shape>> = vec![Box::new(Square(2.0)), /* a Circle... */];
for s in &shapes { println!("{}", s.describe()); }
```

- `impl Filter for Biquad` promises Biquad has `process_sample` and `reset`.
- `fn process(&mut self, x: &[f64]) -> Vec<f64> { ... }` inside the trait is a
  **default method** that every filter inherits.
- `Box<dyn Filter>` means "some filter, decided at run time, stored on the
  heap". That's how `FilterChain` holds a Biquad *and* a DcBlocker in one Vec.
- `impl Filter + 'static` in `then` means "any filter type that doesn't
  borrow anything temporary". You can ignore the `'static` for now.

## 3. Predict first

1. Sketch the one-pole's output for a step input (0, 0, 1, 1, 1, ...).
2. What happens with a = 1.01?
3. In the "ah" demo, which parameter sets the pitch, and which set the vowel?
4. A chain of two identical low-passes: how does its attenuation in dB at
   4 kHz compare with one?

## 4. Exercises

In `src/iir.rs`:

1. `Filter::process`, the default method (one line)
2. `OnePoleLowpass`: `new`, `process_sample`, `reset`
3. `DcBlocker`: `new`, `process_sample`, `reset`
4. `Biquad`: `lowpass`, `highpass`, `bandpass`, `process_sample`, `reset`
5. `FilterChain`: `then`, `process_sample`, `reset`

<details><summary>Hint: Biquad::process_sample</summary>

Compute `y` from the formula using `self.x1, self.x2, self.y1, self.y2`. Then
shift the memory, oldest first: `self.x2 = self.x1; self.x1 = x;` and the same
for y. Getting the order wrong overwrites x1 before you've copied it.

</details>

<details><summary>Hint: three cookbook filters, one helper</summary>

All three compute `w0`, `cos(w0)` and `α` the same way. You can write a
private helper `fn cookbook_terms(freq, q, sample_rate) -> (f64, f64)` that
returns `(cos_w0, alpha)`. A tuple return is the easy way to hand back two values.

</details>

## 5. Run it

```bash
cargo run --example lesson07
```

- `out/lesson07_response.svg`: compare the one-pole and biquad low-passes.
  Which falls faster?
- Listen to the filtered noises.
- **Listen to `lesson07_buzz.wav` and then `lesson07_vowel_aah.wav`.** Same
  source, different filter.
- `lesson07_telephone.wav` keeps 300–3400 Hz, the old telephone band. Try it
  on your own voice: `cargo run --example lesson07 -- my.wav`.

## 6. Check your understanding

1. Why does the telephone chain use two high-passes and two low-passes
   rather than one of each?
2. Of the usual voice-disorder features (F0, jitter, shimmer, HNR, MFCCs,
   mel spectra), which describe the source, and which the filter?
3. Why are IIR filters a poor choice when the exact *timing* of features
   matters, for example aligning onsets across frequency bands?

<details><summary>Answers</summary>

1. Each biquad falls at 12 dB per octave outside its band. Two in series
   fall at 24 dB per octave, a much sharper edge.
2. Source: F0, jitter, shimmer, HNR. Filter: MFCCs and mel spectra mostly
   describe the envelope, which means formants and vocal-tract shape. (Fine
   harmonic detail in a spectrogram carries source information too.)
3. IIR filters delay different frequencies by different amounts (non-linear
   phase). Symmetric FIR filters delay all frequencies equally.

</details>

Next: [Lesson 08, the DFT](../08-dft/README.md)

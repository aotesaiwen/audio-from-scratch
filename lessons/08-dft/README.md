# Lesson 08: Complex numbers and the DFT

> **Audio:** "how much of each frequency?", spectra, bins, resolution · **Rust:** operator overloading, `Copy`, trait impls from `std::ops`
> **You edit:** `src/complex.rs`, `src/dft.rs` · **Check:** `cargo test l08` · **Demo:** `cargo run --example lesson08` · **Time:** ~2–3 h

Lesson 06 measured a filter's response one sine at a time. The discrete
Fourier transform measures *every* frequency of a signal in one go. This
lesson is the hinge of the course: everything after it lives in the
frequency domain.

## 1. The idea

**Probing.** To ask "how much 440 Hz is in x?", multiply x sample by sample
with a 440 Hz probe and add up the products. If x contains that frequency,
the products are mostly positive and the sum is large. Otherwise they cancel
and the sum is near zero.

**The phase problem.** A cosine-shaped signal probed with a *sine* gives
zero, even though the frequency is right there. They're a quarter cycle out
of step. So probe twice, with a cosine and with a sine, and keep both
answers. A pair of numbers that belong together is exactly what a **complex
number** is.

**Complex numbers in 60 seconds.** `z = re + im·j`, where j² = −1.

- Picture z as an arrow from 0 to the point (re, im). Its **length** |z| is
  the *magnitude*, and its **angle** arg(z) is the *phase*.
- Adding works like adding arrows. **Multiplying multiplies the lengths and
  adds the angles.** That makes complex numbers the natural language of
  rotation, and of oscillation.
- (a + bj)(c + dj) = (ac − bd) + (ad + bc)j
- **Euler's formula**: e^(jθ) = cos θ + j·sin θ, the point at angle θ on the
  unit circle. As θ grows it goes round and round, which makes it a
  perfect probe.

**The DFT.** For N samples, probe at N frequencies, k = 0 … N−1, each making
k whole turns over the N samples:

    X[k] = Σ_{n=0}^{N−1} x[n] · e^(−j·2π·k·n/N)

- Bin k is the frequency **k · sample_rate / N**. Bins are spaced
  **sample_rate / N = 1 / duration** apart, so a longer frame gives finer
  resolution.
- |X[k]| says how much of that frequency there is, and arg(X[k]) its phase.
- A sine of amplitude A sitting exactly on bin k gives |X[k]| = A·N/2.
- For real signals, X[N−k] is the mirror image (complex conjugate) of X[k].
  Only bins 0 to N/2 are new information, and N/2 is Nyquist again.
- The **inverse DFT** rebuilds x by adding up all those rotating arrows.
  Nothing is lost.

**Cost.** N bins × N samples = N² multiplications. One second at 16 kHz is
256 million. The demo times it, and lesson 09 fixes it.

## 2. Rust for this lesson

**Teaching `+` and `*` to your own type.** Operators are traits in
`std::ops`. `Add` for `Complex` is written for you. Here's a parallel example
for a 2-D vector:

```rust
use std::ops::Add;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Vec2 { x: f64, y: f64 }

impl Add for Vec2 {
    type Output = Vec2;                       // what a + b produces
    fn add(self, rhs: Vec2) -> Vec2 {
        Vec2 { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
```

- `impl Mul<f64> for Complex` makes `z * 0.5` work, a different right-hand type.
- `AddAssign` gives you `+=`. Its method takes `&mut self`.

**Why `Copy` matters.** Without `#[derive(Clone, Copy)]`, `a + b` would
*move* a and b into the `add` method, and using `a` again would be an error
("value used after move"). `Copy` says "this is small, duplicate it freely",
the way plain numbers behave.

`Complex::ZERO` is an **associated constant**: a value that belongs to the type.

## 3. Predict first

1. The DFT of `[1, 0, 0, 0]`? Of `[1, 1, 1, 1]`?
2. The demo analyses 0.25 s at 8 kHz. How many samples? Bin spacing?
3. Which bin holds 440 Hz? 554.37 Hz?
4. A sine of amplitude 0.3 lands exactly on a bin, N = 2000. What's |X[k]|?

## 4. Exercises

In `src/complex.rs`: `from_polar`, `expj`, `abs`, `norm_sqr`, `arg`, `conj`,
and the `Sub`, `Mul`, `Mul<f64>` and `AddAssign` impls.

In `src/dft.rs`: `to_complex`, `dft`, `idft`, `bin_frequency`,
`magnitude_spectrum`, `peak_frequency`.

<details><summary>Hint: dft</summary>

For each k, start `let mut sum = Complex::ZERO;`, loop over the samples with
`enumerate`, and add `xn * Complex::expj(angle)`. Collect one sum per k.
Computing `(k * n) % big_n` before converting to f64 keeps the angle small,
because sin and cos of huge angles lose precision.

</details>

## 5. Run it

```bash
cargo run --example lesson08
```

Open `out/lesson08_spectrum.svg`: three peaks, one per note. Read the printed
peak list carefully. The middle note is really 554.37 Hz. Why is it reported
at 556 Hz, and why does its amplitude come out smaller than the others?
(Hold that thought for lesson 10.) Then look at the timing table.

## 6. Check your understanding

1. A 25 ms frame at 16 kHz: how many samples, and how far apart are its bins?
   Can it separate the harmonics of a 100 Hz voice?
2. Why do the bins above N/2 add nothing for a real signal?
3. Spectrograms usually show only magnitude. What does throwing away the phase lose?

<details><summary>Answers</summary>

1. 400 samples, 40 Hz apart. Harmonics of a 100 Hz voice are 100 Hz apart,
   so yes, but barely: each harmonic spreads over a couple of bins.
2. For real x, X[N−k] is the conjugate of X[k]: same magnitude, opposite
   phase. It's a mirror copy.
3. Timing: how the components line up, where transients fall. You can't
   turn a magnitude spectrogram back into audio without guessing the phase.
   That's the job neural vocoders like HiFi-GAN do.

</details>

Next: [Lesson 09, the FFT](../09-fft/README.md)

# Lesson 09: The FFT

> **Audio:** the fast Fourier transform, zero-padding · **Rust:** recursion, `step_by`, `assert!`, `#[should_panic]`, timing
> **You edit:** `src/fft.rs` · **Check:** `cargo test l09` · **Demo:** `cargo run --example lesson09` · **Time:** ~1.5–2 h

## 1. The idea

The FFT computes exactly the same numbers as the DFT, but in N·log₂N steps
instead of N². For a million samples that's ~20 million operations instead
of a trillion.

**The trick (Cooley–Tukey, radix 2).** Split the sum over n into even and
odd samples:

    X[k] = Σ_m x[2m]·e^(−j2πk·2m/N)   +   Σ_m x[2m+1]·e^(−j2πk(2m+1)/N)
         = Σ_m x[2m]·e^(−j2πkm/(N/2)) + e^(−j2πk/N) · Σ_m x[2m+1]·e^(−j2πkm/(N/2))
         =        E[k]                +     W^k     ·        O[k]

E and O are just the DFTs of the even-indexed and odd-indexed samples, each
of length N/2. W^k = e^(−j2πk/N) is called a "twiddle factor". Two more facts
finish it:

- E and O repeat every N/2 bins, so E[k + N/2] = E[k] (same for O).
- W^(k+N/2) = −W^k, because half a turn flips the sign.

So each pair of outputs costs one multiplication:

```
X[k]       = E[k] + W^k · O[k]
X[k + N/2] = E[k] − W^k · O[k]       for k = 0 .. N/2 − 1
```

Then do the same to E and O, and so on down to length 1, where the DFT of
one sample is the sample itself. That's log₂N levels with N work each.

This needs N to be a power of two. Real code **zero-pads** each frame up to
the next power of two. Padding makes the bins closer together, but it only
*interpolates* the spectrum. It can't separate two tones the original frame
couldn't, because resolution comes from the frame's duration.

**Inverse for free:** ifft(X) = conj(fft(conj(X))) / N. Conjugating flips
the direction of rotation, which turns the forward transform into the inverse.

**`rfft`** (as in `np.fft.rfft`): pad or cut a real frame to `n_fft`, FFT it,
keep bins 0 to n_fft/2. The STFT calls it for every frame.

## 2. Rust for this lesson

**Recursion** is a function calling itself on a smaller problem. Here's a
parallel divide-and-conquer example:

```rust
fn sum_halves(x: &[f64]) -> f64 {
    if x.len() <= 1 {
        return x.first().copied().unwrap_or(0.0); // stopping case
    }
    let mid = x.len() / 2;
    sum_halves(&x[..mid]) + sum_halves(&x[mid..])
}
```

- `x.iter().step_by(2)` gives every 2nd item. `.skip(1)` first starts at index 1.
- `.copied()` turns `&Complex` into `Complex` (fine, because Complex is `Copy`).
- `assert!(condition, "message {n}")` panics with your message if the
  condition is false. Use it to reject bad input loudly.
- `#[should_panic(expected = "...")]` on a test means "this test passes only
  if the code panics with that text".
- `usize::is_power_of_two()` and `1 << p` (2 to the power p) are useful here.

## 3. Predict first

1. How much faster should the FFT be than the DFT for N = 4096? For N = 2²⁰?
2. `librosa.stft(..., n_fft=1024)` at 16 kHz: how long is each frame,
   and how far apart are its bins?
3. The demo plots the spectrum of a 150 Hz synthetic voice. How many peaks
   do you expect between 0 and 1000 Hz?

## 4. Exercises

In `src/fft.rs`:

1. `fft` (recursive)
2. `ifft` (the conjugation trick)
3. `rfft`

The tests check your FFT against your DFT from lesson 08 on random signals.

<details><summary>Hint 1: the pieces</summary>

You need three things: a stopping case (length 1), two recursive calls (on
the even samples and on the odd samples), and one loop over k from 0 to N/2
that fills in *two* outputs per pass using the boxed formulas above.

</details>

<details><summary>Hint 2: the full outline (try without it first)</summary>

```
n = x.len(); assert it's a power of two
if n == 1: return vec![x[0]]
even = every 2nd sample from 0;  odd = every 2nd sample from 1
e = fft(even);  o = fft(odd)
out = n zeros
for k in 0 .. n/2:
    t = expj(−2π·k/n) · o[k]
    out[k] = e[k] + t;  out[k + n/2] = e[k] − t
return out
```

</details>

## 5. Run it

```bash
cargo run --example lesson09
```

Compare the two timing columns. Open `out/lesson09_voice_spectrum.svg` and
count the harmonics.

## 6. Check your understanding

1. Zero-padding a 25 ms frame (400 samples at 16 kHz) to 4096 gives bins
   3.9 Hz apart. Can it now separate two tones 10 Hz apart?
2. Why is it "log₂N levels"?
3. Stretch: write an *iterative*, in-place FFT (look up "bit-reversal
   permutation"). Make it pass the same tests. Is it faster?

<details><summary>Answers</summary>

1. No. A 25 ms frame can only resolve frequencies about 1/0.025 = 40 Hz
   apart. Padding draws a smoother curve through the same blurry peaks.
2. Each level halves the length: N, N/2, ..., 1 takes log₂N halvings.
3. Usually yes, by avoiding allocations. Real libraries such as FFTW and
   rustfft also use radix-4 and mixed radices for lengths that aren't powers of two.

</details>

Next: [Lesson 10, windows](../10-windows/README.md)

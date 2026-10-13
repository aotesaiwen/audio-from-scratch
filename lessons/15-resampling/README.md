# Lesson 15 (bonus): Resampling

> **Audio:** reconstruction, interpolation, anti-aliasing, windowed sinc · **Rust:** practice
> **You edit:** `src/resample.rs` · **Check:** `cargo test l15` · **Demo:** `cargo run --example lesson15` · **Time:** ~1.5 h

Speech pipelines nearly always start by resampling, for example with
`torchaudio.transforms.Resample(44100, 16000)`. Here's what that does.

## 1. The idea

Resampling asks: *what would the samples have been at a different rate?*
Answering it means reconstructing the continuous signal between the samples
you have, then sampling that again.

**Naive decimation** (keep every 3rd sample for 48 → 16 kHz) skips the
reconstruction. Anything between the new Nyquist (8 kHz) and the old one
(24 kHz) folds back, exactly as in lesson 04.

**Linear interpolation** draws straight lines between samples. It's cheap
and better than nothing, but a straight line is a poor low-pass filter: some
aliasing still gets through and the highs get dulled.

**The ideal answer** (Whittaker–Shannon): a band-limited signal is
*exactly* recovered by placing a **sinc** at every sample:

    x(t) = Σ_k x[k] · sinc(t − k),      sinc(u) = sin(πu) / (πu)

sinc is 1 at 0 and 0 at every other whole number, so it passes through every
sample and fills the gaps in the only band-limited way. When **downsampling**,
make the sinc wider by the ratio (cutoff < 1). That turns it into a low-pass
at the *new* Nyquist, so filtering and interpolation happen in one step.

The real sinc never ends, so it's cut off after a number of zero crossings
and tapered with a window (Hann here). That's a trade-off: more zero
crossings give a sharper cutoff but cost more time. torchaudio's default is
a Hann-windowed sinc with `lowpass_filter_width=6` and `rolloff=0.99`, the
same design you're building.

```
output position t (in input samples) = i · sr_in / sr_out

       x[k−2]  x[k−1]  x[k]  x[k+1]  x[k+2] ...
         ×       ×      ×      ×       ×
       w·sinc  w·sinc  ...   (centred on t, cut off at ±half_width)
         └───────────── sum ─────────────┘ → y[i]
```

## 2. Exercises

In `src/resample.rs`:

1. `decimate_naive`
2. `output_len`
3. `resample_linear`
4. `sinc`
5. `resample_sinc`

The doc comments give the exact formulas. The tests check a ramp stays a
ramp, a sine survives upsampling, and an out-of-band tone is removed
(naive decimation lets it alias; your sinc version must not).

## 3. Run it

```bash
cargo run --example lesson15
```

A sweep from 100 Hz to 20 kHz at 48 kHz is converted to 16 kHz three ways.
In `out/lesson15_naive.svg`, the part of the sweep above 8 kHz comes back
down as a reflected line. Compare it with `_linear` and `_sinc`, and listen
to all three WAVs.

## 4. Check your understanding

1. Converting 50 kHz to 16 kHz, the ratio is 16/50 = 8/25. Output sample
   positions only ever fall at 8 different fractional offsets between input
   samples. How could a resampler use that to run much faster than yours?
2. What does `rolloff = 0.99` buy you, and what does it cost?
3. Why do resamplers matter for ML reproducibility?

<details><summary>Answers</summary>

1. Precompute the 8 kernels once and reuse them: a **polyphase** resampler.
   Yours recomputes sinc and cos for every tap of every output sample.
2. The filter starts cutting slightly *below* Nyquist, which gives the
   finite-length kernel room to reach full attenuation by Nyquist itself.
   The cost: the top 1% of the band is slightly attenuated.
3. Different libraries (soxr, scipy, torchaudio, librosa's various
   `res_type`s) give slightly different filters, so features differ in the
   high bands. Fix and record the resampler like any other hyperparameter.

</details>

Next: [Lesson 16, LPC and McAdams](../16-lpc-mcadams/README.md)

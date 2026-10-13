# Lesson 16 (bonus): LPC and McAdams anonymisation

> **Audio:** linear prediction, the source–filter model, poles and formants, voice anonymisation · **Rust:** practice (complex arithmetic, numerical iteration)
> **You edit:** `src/lpc.rs` · **Check:** `cargo test l16` · **Demo:** `cargo run --example lesson16` · **Time:** ~3 h

The McAdams-coefficient anonymiser (VoicePrivacy Challenge baseline B2) is
built entirely from things you now have: autocorrelation (13), FIR
filtering (06), complex numbers (08), framing and windows (05, 10).

## 1. The idea

**Linear prediction.** Speech is very predictable from its own recent past.
Try to predict each sample from the previous p:

    x̂[n] = −(a₁·x[n−1] + a₂·x[n−2] + … + a_p·x[n−p])

Choose the a's to make the prediction error as small as possible. The
error, or **residual**, is

    e[n] = x[n] + a₁·x[n−1] + … + a_p·x[n−p]  =  Σ_{j=0}^{p} a_j·x[n−j]   (a₀ = 1)

That's an FIR filter with taps `a`, called A(z). Running e back through the
*all-pole* filter 1/A(z) rebuilds x exactly. So LPC splits speech into:

- **e[n]**: what couldn't be predicted, i.e. the pulses and noise. The **source**.
- **1/A(z)**: a smooth resonant filter. The **vocal tract**, whose peaks are
  the formants.

That's lesson 07's source–filter model again, now *measured* from real speech.

**Solving for a.** Minimising the error leads to a set of equations built
from the autocorrelation r[0..p]. The **Levinson–Durbin recursion** solves
them in p² steps by growing the solution one order at a time (see the doc
comment for the recursion).

**Poles are formants.** The roots of A(z) (the poles of 1/A) come in complex
conjugate pairs. A pole at angle θ and radius ρ makes a resonance at
frequency f = θ · sr / (2π). The closer ρ is to 1, the sharper the peak.

**McAdams.** Move every pole's angle from φ to φ^α, with α ≈ 0.8, and keep
its radius. Angles are in radians from 0 to π:

- φ < 1 rad (below sr/2π, about 2.5 kHz at 16 kHz): φ^0.8 > φ, so those
  formants move **up**.
- φ > 1 rad: they move **down**.

The envelope gets warped, which changes the apparent speaker, while the
residual (and so the pitch and timing) is untouched. That's why it's cheap,
needs no training, and keeps intelligibility reasonably well.

**Root finding.** Durand–Kerner: start from p distinct guesses and
repeatedly nudge each one by P(z) / (product of distances to the other
guesses) until nothing moves. It's simple, and reuses your Complex type
(Div is provided). numpy's `np.roots` uses eigenvalues of a companion matrix instead.

**The recipe** (the VoicePrivacy B2 baseline): 20 ms frames every 10 ms, √(Hann) windows normalised so overlaps add
to 1, LPC order 20, α = 0.8, overlap-add, then normalise the peak.

## 2. Exercises

In `src/lpc.rs`:

1. `levinson_durbin`, then `lpc`
2. `residual`, `synthesize`
3. `poly_roots`, `poly_from_roots`
4. `mcadams`
5. `mcadams_anonymize`

## 3. Run it

```bash
cargo run --example lesson16
```

- `out/lesson16_envelope.svg`: a vowel's spectrum, its LPC envelope, and the
  envelope after McAdams. Check which formants moved which way.
- Listen to `lesson16_original.wav` and the two McAdams versions.
- Try it on your own recording with `-- path.wav`. Is it still recognisably you?

## 4. Check your understanding

1. With α = 0.8 at 16 kHz, does /a/'s F1 (730 Hz) go up or down? Its F3 (2440 Hz)?
2. Why doesn't McAdams change the pitch?
3. Why is McAdams a weak anonymiser against an attacker who knows the method?

<details><summary>Answers</summary>

1. F1: φ = 2π·730/16000 ≈ 0.287 rad, and 0.287^0.8 ≈ 0.368 rad ≈ 937 Hz,
   so it goes up. F3: φ ≈ 0.958 rad, and 0.958^0.8 ≈ 0.966 rad ≈ 2460 Hz,
   which barely moves. It sits close to the 1-radian pivot (~2546 Hz), so
   the warping is gentle there.
2. Pitch lives in the residual (the pulse spacing), which is passed through
   unchanged. Only the envelope filter is modified.
3. The transform is simple and nearly invertible. An attacker can anonymise
   their own enrolment data with the same α and compare like with like (a
   "semi-informed" attack), or estimate and undo the warping.

</details>

**That's the course.** Your `lab` crate now reads audio, filters it,
computes spectrograms, mel features and MFCCs, tracks pitch, measures voice
quality, resamples and anonymises. Remove the `#![allow(...)]` line at the
top of `src/lib.rs` and see what the compiler thinks of your code.

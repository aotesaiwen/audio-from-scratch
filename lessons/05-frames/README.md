# Lesson 05: Frames, chopping audio into analysable pieces

> **Audio:** framing, hop size, short-time energy, zero-crossing rate, silence trimming · **Rust:** ownership and borrowing, slices, `&mut`, `Option`
> **You edit:** `src/frames.rs` · **Check:** `cargo test l05` · **Demo:** `cargo run --example lesson05` · **Time:** ~2 h

Almost every speech feature (spectrograms, MFCCs, pitch tracks) is computed
frame by frame. This lesson builds the frames, and teaches the
part of Rust people find hardest: borrowing.

## 1. The idea

Speech changes constantly, but slowly compared with its vibrations: the
tongue and jaw move over tens of milliseconds while the vocal folds vibrate
every few. Over a short **frame** of 20–40 ms, speech is roughly steady, so
you can analyse it as if it were. Frames start every **hop** samples and
usually overlap:

```
signal  |=========================================|
frame 0 [---------]
frame 1      [---------]
frame 2           [---------]
             <hop>
        <frame_len>
```

Without padding, the number of whole frames is

    1 + (n_samples − frame_len) / hop      (integer division; 0 if too short)

The classic speech settings are 25 ms frames every 10 ms.

Two cheap per-frame measures already separate a lot:

| | Short-time energy (RMS dB) | Zero-crossing rate |
|---|---|---|
| silence | very low | random |
| voiced (vowels) | high | low (dominated by low frequencies) |
| unvoiced ("s", "f") | medium | high (noisy, high frequencies) |

**Trimming silence** (like `librosa.effects.trim(y, top_db=60)`): frames
within `top_db` of the *loudest* frame count as sound. The threshold is
relative because recording levels vary, and an absolute −40 dBFS would mean
different things on different microphones.

**Fades.** Cutting audio abruptly creates a step, and a step sounds like a
click. A 10 ms fade in and out removes it.

## 2. Rust for this lesson: ownership and borrowing

Every value in Rust has one **owner**, and it's freed when its owner goes
away. Other code can **borrow** it:

- `&x`, a shared borrow: read only. Any number at once.
- `&mut x`, an exclusive borrow: read and write. Only one at a time, and no
  shared borrows meanwhile.

A **slice** `&x[a..b]` is a borrowed *view* of part of a Vec. It's just a
pointer and a length, with no copying. So `frames` can return a
`Vec<&[f64]>`, a list of views into the original signal. Making 100 frames
copies no samples at all.

The rules exist to stop this bug:

```rust
let mut v = vec![1.0, 2.0];
let first = &v[0];       // borrow v
v.push(3.0);             // error[E0502]: cannot borrow `v` as mutable
                         //               because it is also borrowed as immutable
println!("{first}");
```

`push` may move the Vec to a bigger block of memory, which would leave
`first` pointing at freed memory. In C that's a crash or silent corruption.
Rust refuses to compile it.

**In-place changes** take `&mut [f64]`:

```rust
pub fn apply_gain(x: &mut [f64], gain: f64) { ... }

let mut samples = vec![0.5, -0.5];
apply_gain(&mut samples, 2.0);  // samples is now [1.0, -1.0]
```

**Returning a borrow.** `trim_silence(x: &[f64], ...) -> &[f64]` returns
part of its input. Rust works out (this is called *lifetime elision*) that
the result borrows from `x`, so it won't let you drop `x` while you still hold
the trimmed slice.

**`Option`** is how Rust says "maybe nothing": `Some(value)` or `None`.
`xs.get(i)` returns `None` past the end instead of panicking;
`.unwrap_or(&0.0)` gives a default. `xs.iter().position(|v| ...)` returns the
`Option<usize>` of the first match.

## 3. Predict first

1. 1 s at 16 kHz, 25 ms frames, 10 ms hop: how many frames?
2. The demo's synthetic utterance is silence, "ah", "sss", silence, "ee",
   silence. Sketch its energy and ZCR curves.
3. Why might `top_db = 60` fail to trim a recording made next to a laptop fan?

## 4. Exercises

In `src/frames.rs`:

1. `num_frames`
2. `frames`
3. `frame_rms_db`, `frame_zcr` (reuse lesson 03 and 04)
4. `apply_gain`, `fade`
5. `mix`
6. `trim_silence`

<details><summary>Hint: frames</summary>

Frame i starts at `i * hop` and ends at `i * hop + frame_len`. Build the Vec
with a loop and `push(&x[start..end])`, or with
`(0..n).map(|i| ...).collect()`.

</details>

<details><summary>Hint: trim_silence</summary>

1. `let db = frame_rms_db(x, frame_len, hop);`
2. Find the loudest value (`fold` with `f64::max`, starting at `f64::NEG_INFINITY`).
3. `db.iter().position(|v| *v > loudest - top_db)` gives the first loud frame,
   and `rposition` the last.
4. `match` on the pair of Options. If both are `Some`, slice from the first
   frame's start to the last frame's end (careful not to pass `x.len()`).
   Otherwise return `&x[0..0]`, an empty slice.

</details>

## 5. Run it

```bash
cargo run --example lesson05
```

Compare `out/lesson05_energy.svg` and `out/lesson05_zcr.svg` with your
sketch, then listen to `out/lesson05_trimmed.wav`.

Now try one of your own recordings:

```bash
cargo run --example lesson05 -- C:\path\to\recording.wav
```

## 6. Check your understanding

1. Why does `frames` return `Vec<&[f64]>` and not `Vec<Vec<f64>>`? When
   would you *need* the second?
2. With 25 ms frames and a 10 ms hop, each sample is in up to 3 frames. Is
   that wasteful?
3. Why must `trim_silence`'s result be a borrow and not a copy? (Or must it?)

<details><summary>Answers</summary>

1. Views are free; copies cost memory and time. You'd need owned frames when
   you want to change each one independently (like multiplying by a window in
   lesson 10), or keep them after the signal is gone.
2. It's the price of time resolution. Hops shorter than the frame mean
   features change smoothly from frame to frame instead of jumping.
3. It doesn't have to be. Returning a borrow is a choice that avoids a copy.
   The demo calls `.to_vec()` when it needs to own and modify the result.

</details>

Next: [Lesson 06, FIR filters](../06-fir-filters/README.md)

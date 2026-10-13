# Audio from scratch

Learn audio signal processing and Rust at the same time by building a small
librosa + Praat yourself, starting from an empty file. The repo uses no
audio crates and hides nothing. Every step from WAV bytes to spectrograms,
MFCCs and pitch is code you write.

By the end you'll have built the front end that most audio and speech tools
share: filters, spectrograms, mel features and MFCCs, pitch tracking,
voice-quality measures and resampling. Lines like these should then read as
plain descriptions of what happens to the signal:

```python
librosa.feature.melspectrogram(y=y, sr=sr, n_mels=64)
call(sound, "To Pitch", 0.0, 75, 600)
torchaudio.transforms.Resample(44100, 16000)
```

## How it works

Each lesson has three parts:

1. **A guide**, `lessons/NN-topic/README.md`. It explains one audio idea and
   the Rust you need for it.
2. **Exercises** in `src/`. These are functions whose body is `todo!()`, and
   your job is to replace each one.
3. **Feedback.** Tests at the bottom of each `src/` file tell you when you're
   right. An example program in `examples/` then makes something to look at
   (an SVG plot) or listen to (a WAV file).

The loop for every lesson:

```
read the guide → answer "Predict first" on paper → write code
→ cargo test l03 → read the failure → fix → ... → all green
→ cargo run --example lesson03 → open out/ → look and listen
→ answer "Check your understanding"
```

## Setup

Rust is already installed on this machine. From this folder:

```bash
cargo test l00
```

You should see 6 failing tests, each saying `not yet implemented: l00: ...`.
That's correct, because you haven't written anything yet. Open
[lessons/00-setup/README.md](lessons/00-setup/README.md) and start there.

| Command | What it does |
|---|---|
| `cargo test l05` | Run only lesson 05's tests (every test name starts with its lesson number) |
| `cargo test` | Run everything (expect lots of failures until you've done every lesson) |
| `cargo run --example lesson05` | Run lesson 05's demo, which writes files into `out/` |
| `cargo run --example lesson05 -- my.wav` | Run a demo on your own recording (from lesson 05 on) |

Open `.svg` files in a web browser and play `.wav` files in any audio player.
An editor with **rust-analyzer** (VS Code: install the "rust-analyzer"
extension) shows types and errors as you type, which helps a lot while learning.

## The lessons

| # | Lesson | Audio idea | Rust idea |
|---|---|---|---|
| 00 | [Setup and warm-up](lessons/00-setup/README.md) | MIDI notes and Hz | functions, types, loops, the test loop |
| **Part I** | **Sound as numbers** | | |
| 01 | [Sound is a list of numbers](lessons/01-sampling/README.md) | sampling, sine waves | `Vec`, `for`, `as` casts |
| 02 | [WAV files](lessons/02-wav-files/README.md) | PCM, bit depth, file headers | structs, bytes, `Result` and `?` |
| 03 | [Loudness and bits](lessons/03-loudness/README.md) | RMS, decibels, quantisation noise | iterators and closures |
| 04 | [Aliasing](lessons/04-aliasing/README.md) | Nyquist, folding | `if` as an expression, `while`, `windows` |
| **Part II** | **Working in time** | | |
| 05 | [Frames](lessons/05-frames/README.md) | framing, energy, zero-crossing rate, trimming silence | borrowing, slices, `&mut` |
| 06 | [FIR filters](lessons/06-fir-filters/README.md) | convolution, impulse response, pre-emphasis | `usize` pitfalls, closures as arguments |
| 07 | [IIR filters](lessons/07-iir-filters/README.md) | feedback, biquads, the source–filter model | structs with state, traits, `Box<dyn Trait>` |
| **Part III** | **Working in frequency** | | |
| 08 | [The DFT](lessons/08-dft/README.md) | complex numbers, spectra | operator overloading, `Copy` |
| 09 | [The FFT](lessons/09-fft/README.md) | divide and conquer | recursion, benchmarking |
| 10 | [Windows](lessons/10-windows/README.md) | spectral leakage | `total_cmp`, `saturating_sub` |
| 11 | [The STFT](lessons/11-stft/README.md) | spectrograms, the time–frequency trade-off | `Default`, config structs, 2-D data |
| **Part IV** | **Speech features** | | |
| 12 | [Mel and MFCC](lessons/12-mel-mfcc/README.md) | perceptual frequency, filterbanks, the DCT | matrix maths with iterators |
| 13 | [Pitch](lessons/13-pitch/README.md) | autocorrelation, F0, octave errors | `Option` and `?` |
| 14 | [Voice quality](lessons/14-voice-quality/README.md) | jitter, shimmer, HNR | `Display`, putting it all together |
| **Bonus** | | | |
| 15 | [Resampling](lessons/15-resampling/README.md) | interpolation, anti-aliasing | practice |
| 16 | [LPC and McAdams](lessons/16-lpc-mcadams/README.md) | linear prediction, formants, voice anonymisation | practice |

Every lesson builds on the earlier ones, because lesson 11 calls your FFT
from lesson 09, and so on. Go in order. Plan on roughly 1–3 hours per lesson.

## When you're stuck

Work down this list:

1. **Read the whole error message.** Rust's compiler errors are unusually
   good. Read them top to bottom, fix the *first* one, and recompile.
2. **Open the hints** in the lesson guide. They're folded away under
   "Hint" so you only see them when you choose to.
3. **Ask Claude Code in this folder.** `CLAUDE.md` tells it to act as a tutor:
   it gives hints and asks questions instead of writing your solutions.
   (You can still ask it to just show you, if you decide you want that.)
4. **Look at the reference solution**, as a last resort or to compare once
   you've passed:
   ```bash
   git show solutions:src/fft.rs
   ```
   Or compare your committed work with it:
   ```bash
   git diff main solutions -- src/fft.rs
   ```

Commit after each lesson (`git add -A`, then `git commit -m "lesson 03"`) so you
can always see your own progress.

## Layout

```
lessons/      one guide per lesson: read these
src/          your library (`lab`): the exercises live here, tests at the bottom of each file
  plot.rs     given: writes SVG line plots and heatmaps
  synth.rs    given: test signals (noise, chirps, synthetic voices with known jitter)
examples/     one demo program per lesson
tools/        optional Python script for comparing with Praat
out/          where the demos write their plots and sounds (not committed)
```

## Progress

- [ ] 00 Setup and warm-up
- [ ] 01 Sound is a list of numbers
- [ ] 02 WAV files
- [ ] 03 Loudness and bits
- [ ] 04 Aliasing
- [ ] 05 Frames
- [ ] 06 FIR filters
- [ ] 07 IIR filters
- [ ] 08 The DFT
- [ ] 09 The FFT
- [ ] 10 Windows
- [ ] 11 The STFT
- [ ] 12 Mel and MFCC
- [ ] 13 Pitch
- [ ] 14 Voice quality
- [ ] 15 Resampling (bonus)
- [ ] 16 LPC and McAdams (bonus)

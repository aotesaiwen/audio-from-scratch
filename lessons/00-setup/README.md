# Lesson 00: Setup and Rust warm-up

> **Audio:** MIDI notes and frequencies · **Rust:** functions, types, loops, the test loop
> **You edit:** `src/warmup.rs` · **Check:** `cargo test l00` · **Time:** ~1 h

This lesson has one goal: getting comfortable with the loop you'll use for
the rest of the course, *edit → test → read the error → fix*. The exercises are
small on purpose.

## 1. The project layout

```
Cargo.toml        the project's settings (name, edition, no dependencies)
src/lib.rs        the list of modules: one per topic
src/warmup.rs     this lesson's module
examples/         small programs that use your library
```

`cargo` is Rust's build tool. You'll mostly use two commands:

```bash
cargo test l00
```

This compiles everything and runs every test whose name contains `l00`.

```bash
cargo run --example lesson01
```

This runs `examples/lesson01.rs`. You'll start using it next lesson.

## 2. Reading a test failure

Run `cargo test l00` now. Each test fails with something like:

```
---- warmup::tests::l00_mean stdout ----
thread 'warmup::tests::l00_mean' panicked at src\warmup.rs:43:5:
not yet implemented: l00: mean
```

That's `todo!()` doing its job. It compiles fine but crashes when it runs.
Once you've written a function, a wrong answer looks like this instead:

```
expected 2.5 (± 0.000000000001), got 2
```

The tests are at the bottom of `src/warmup.rs`. **Read them.** They're the
precise specification of what each function must do.

## 3. Just enough Rust

```rust
let x = 5.0;              // a variable; type f64 is inferred from "5.0"
let mut total = 0.0;      // `mut` means it's allowed to change
total += x;

let n: usize = 10;        // usize: whole numbers for lengths and indices
let f = n as f64;         // `as` converts between number types; Rust never does it silently

fn double(x: f64) -> f64 {
    x * 2.0               // no semicolon: this is the return value
}
```

**Types you'll meet constantly:**

| Type | What for |
|---|---|
| `f64` | sample values, frequencies, seconds: anything with a fractional part |
| `usize` | lengths, indices, counts (can't be negative) |
| `u32` | sample rates |
| `u8`, `i16` | bytes, 16-bit audio samples (lesson 02) |
| `&[f64]` | a borrowed list of f64s that you can read but don't own (lesson 05 explains) |
| `Vec<f64>` | a growable list of f64s that you own |

**Loops:**

```rust
for i in 0..4 { }          // i = 0, 1, 2, 3 (the end is excluded)
for i in (0..4).rev() { }  // i = 3, 2, 1, 0
for &x in xs { }           // each value in the slice xs
```

**The most common beginner error:** putting a semicolon after the last line.

```rust
fn double(x: f64) -> f64 {
    x * 2.0;   // error[E0308]: mismatched types: expected `f64`, found `()`
}
```

With the semicolon, the line becomes a statement that produces nothing
(`()`), so the function has nothing to return. Remove the semicolon.

## 4. Predict first

Answer these on paper before coding:

1. MIDI 69 is 440 Hz and each semitone multiplies by 2^(1/12). What's MIDI 57?
   And MIDI 60 (middle C), roughly?
2. `hz_to_midi` undoes `midi_to_hz`. Which operation undoes "raise 2 to a power"?
3. What should `mean(&[])` return, and why must you handle it specially?

## 5. Exercises

Open `src/warmup.rs` and replace each `todo!(...)` line with real code:

1. `midi_to_hz`
2. `hz_to_midi`
3. `mean`
4. `count_above`
5. `reversed`

Run `cargo test l00` after each one.

<details><summary>Hint: midi_to_hz</summary>

The note is `note − 69` semitones away from A440. Each semitone multiplies
by 2^(1/12), so n semitones multiply by 2^(n/12). Remember `note` is a `u8`:
convert it before subtracting, or `note - 69` underflows for notes below 69.

</details>

<details><summary>Hint: mean of an empty list</summary>

Check `xs.is_empty()` first and `return 0.0;` early. Otherwise you'd divide
0.0 by 0, which gives NaN ("not a number").

</details>

## 6. Version control

This folder is a git repository. When all `l00` tests pass:

```bash
git add -A
```

```bash
git commit -m "lesson 00"
```

## 7. Check your understanding

1. Why does Rust make you write `xs.len() as f64` instead of converting automatically?
2. What's the difference between `let x = 1.0;` and `let mut x = 1.0;`?
3. What would `midi_to_hz(69 + 12 * 20)` do? (Try it in a test. What does the error say?)

<details><summary>Answers</summary>

1. Silent conversions hide bugs, like losing the fractional part, overflowing,
   or mixing up a count with a measurement. Rust makes every conversion visible.
2. The first can never change after it's set. The second can be reassigned.
   Making things immutable by default means you can trust a value stays put.
3. It doesn't compile. 69 + 240 = 309 doesn't fit in a `u8` (0 to 255), and
   the compiler says "this arithmetic operation will overflow". When the
   numbers are only known at run time, a debug build panics with "attempt to
   add with overflow" instead. Either way, Rust checks integer overflow rather
   than quietly wrapping around to a wrong number.

</details>

Next: [Lesson 01, sound is a list of numbers](../01-sampling/README.md)

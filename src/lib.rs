//! # lab: your audio-processing library
//!
//! You build this crate one lesson at a time. By the end it reads WAV files,
//! filters them, computes spectrograms, mel spectrograms and MFCCs, tracks
//! pitch, and measures jitter, shimmer and HNR, all with no black boxes.
//!
//! The course map is in `README.md`. Each module says which lesson fills it in.

// Unfinished exercises leave unused parameters lying around, which would bury
// you in warnings. This line silences them. Delete it once you've finished the
// course and see what the compiler thinks of your code.
#![allow(unused_variables, unused_mut, dead_code, unused_imports)]

// Given code: read it whenever you like, but you don't have to write it.
pub mod plot;
pub mod synth;
#[cfg(test)]
mod testutil;

// Lesson 00: warm-up
pub mod warmup;

// Part I: Sound as numbers
pub mod signal; // 01
pub mod wav; // 02
pub mod level; // 03
pub mod aliasing; // 04

// Part II: Working in time
pub mod frames; // 05
pub mod fir; // 06
pub mod iir; // 07

// Part III: Working in frequency
pub mod complex; // 08
pub mod dft; // 08
pub mod fft; // 09
pub mod window; // 10
pub mod stft; // 11

// Part IV: Speech features
pub mod mel; // 12
pub mod pitch; // 13
pub mod voice; // 14

// Bonus
pub mod resample; // 15
pub mod lpc; // 16

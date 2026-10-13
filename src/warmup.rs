//! Lesson 00: Rust warm-up. Read `lessons/00-setup/README.md` first.
//!
//! Five small functions to get used to Rust's syntax, its types, and the
//! edit → `cargo test` → read-the-error loop you'll use for the whole course.
//! Check them with `cargo test l00`.

/// Convert a MIDI note number to its frequency in Hz.
///
/// MIDI note 69 is concert A, 440 Hz. Each semitone multiplies the frequency
/// by 2^(1/12), so 12 semitones (an octave) doubles it: note 81 is 880 Hz.
pub fn midi_to_hz(note: u8) -> f64 {
    440.0 * 2f64.powf((note as f64 - 69.0) / 12.0)
}

/// The inverse: a frequency in Hz to a (fractional) MIDI note number.
/// 440.0 gives 69.0; 466.16 gives about 70.0.
pub fn hz_to_midi(hz: f64) -> f64 {
    69.0 + 12.0 * (hz / 440.0).log2()
}

/// The arithmetic mean of a list of numbers. An empty list has mean 0.0.
pub fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    let mut total = 0.0;
    for &x in xs {
        total += x;
    }
    total / xs.len() as f64
}

/// How many values are strictly greater than `threshold`?
pub fn count_above(xs: &[f64], threshold: f64) -> usize {
    let mut count = 0;
    for &x in xs {
        if x > threshold {
            count += 1;
        }
    }
    count
}

/// A new Vec with the values in reverse order. Playing audio backwards is
/// exactly this. Write the loop yourself rather than calling `.reverse()`.
pub fn reversed(xs: &[f64]) -> Vec<f64> {
    let mut out = Vec::with_capacity(xs.len());
    for i in (0..xs.len()).rev() {
        out.push(xs[i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;

    #[test]
    fn l00_midi_to_hz() {
        assert_close(midi_to_hz(69), 440.0, 1e-9);
        assert_close(midi_to_hz(81), 880.0, 1e-9);
        assert_close(midi_to_hz(57), 220.0, 1e-9);
        assert_close(midi_to_hz(60), 261.6256, 1e-3); // middle C
    }

    #[test]
    fn l00_hz_to_midi() {
        assert_close(hz_to_midi(440.0), 69.0, 1e-9);
        assert_close(hz_to_midi(880.0), 81.0, 1e-9);
        assert_close(hz_to_midi(466.1638), 70.0, 1e-4);
    }

    #[test]
    fn l00_midi_round_trip() {
        for note in [21u8, 60, 69, 108] {
            assert_close(hz_to_midi(midi_to_hz(note)), note as f64, 1e-9);
        }
    }

    #[test]
    fn l00_mean() {
        assert_close(mean(&[1.0, 2.0, 3.0, 4.0]), 2.5, 1e-12);
        assert_close(mean(&[-1.0, 1.0]), 0.0, 1e-12);
        assert_close(mean(&[]), 0.0, 1e-12);
    }

    #[test]
    fn l00_count_above() {
        assert_eq!(count_above(&[0.1, 0.5, 0.9, 0.5], 0.5), 1);
        assert_eq!(count_above(&[], 0.0), 0);
        assert_eq!(count_above(&[-1.0, -2.0], -3.0), 2);
    }

    #[test]
    fn l00_reversed() {
        assert_eq!(reversed(&[1.0, 2.0, 3.0]), vec![3.0, 2.0, 1.0]);
        assert_eq!(reversed(&[]), Vec::<f64>::new());
    }
}

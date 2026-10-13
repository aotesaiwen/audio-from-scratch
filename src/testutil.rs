//! Helpers for the tests. Given code.
//!
//! Floating-point maths is never exact, so tests compare numbers "to within a
//! tolerance" instead of with `==`. `#[track_caller]` makes a failure point at
//! the line in the test that called these, not at this file.

#[track_caller]
pub fn assert_close(actual: f64, expected: f64, tol: f64) {
    assert!(
        (actual - expected).abs() <= tol,
        "expected {expected} (± {tol}), got {actual}"
    );
}

#[track_caller]
pub fn assert_all_close(actual: &[f64], expected: &[f64], tol: f64) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "length mismatch: got {} values, expected {}",
        actual.len(),
        expected.len()
    );
    for (i, (&a, &e)) in actual.iter().zip(expected).enumerate() {
        assert!(
            (a - e).abs() <= tol,
            "at index {i}: expected {e} (± {tol}), got {a}"
        );
    }
}

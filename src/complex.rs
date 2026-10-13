//! Lesson 08 (part 1): Complex numbers. See `lessons/08-dft/README.md`.
//!
//! You'll teach Rust's `+`, `-` and `*` operators to work on your own type
//! by implementing traits from `std::ops`.
//! Check with `cargo test l08`.

use std::ops::{Add, AddAssign, Div, Mul, Sub};

/// A complex number re + im·j.
///
/// `#[derive(Clone, Copy)]` lets Complex values be copied like plain
/// numbers, so `a + b` doesn't "use up" a and b. That matters a lot for
/// arithmetic types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Complex = Complex { re: 0.0, im: 0.0 };

    pub fn new(re: f64, im: f64) -> Self {
        Complex { re, im }
    }

    /// From polar form: length `r`, angle `theta` radians.
    pub fn from_polar(r: f64, theta: f64) -> Self {
        Complex::new(r * theta.cos(), r * theta.sin())
    }

    /// e^(jθ) = cos θ + j·sin θ: the point at angle θ on the unit circle.
    pub fn expj(theta: f64) -> Self {
        Complex::from_polar(1.0, theta)
    }

    /// Magnitude |z|, the distance from 0. (`f64::hypot` is your friend.)
    pub fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }

    /// |z|², which is cheaper than abs() (no square root). It's the *power*.
    pub fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    /// The angle (phase) in radians, from −π to π. Use `f64::atan2`.
    pub fn arg(self) -> f64 {
        self.im.atan2(self.re)
    }

    /// Complex conjugate: re − im·j.
    pub fn conj(self) -> Self {
        Complex::new(self.re, -self.im)
    }
}

/// Worked example, given: implementing `+`. After this, `a + b` works for
/// Complex values. `type Output` says what `a + b` produces.
impl Add for Complex {
    type Output = Complex;
    fn add(self, rhs: Complex) -> Complex {
        Complex::new(self.re + rhs.re, self.im + rhs.im)
    }
}

impl Sub for Complex {
    type Output = Complex;
    fn sub(self, rhs: Complex) -> Complex {
        Complex::new(self.re - rhs.re, self.im - rhs.im)
    }
}

/// (a + bj)(c + dj) = (ac − bd) + (ad + bc)j
impl Mul for Complex {
    type Output = Complex;
    fn mul(self, rhs: Complex) -> Complex {
        Complex::new(self.re * rhs.re - self.im * rhs.im, self.re * rhs.im + self.im * rhs.re)
    }
}

/// Scaling by a real number: `z * 0.5`.
impl Mul<f64> for Complex {
    type Output = Complex;
    fn mul(self, rhs: f64) -> Complex {
        Complex::new(self.re * rhs, self.im * rhs)
    }
}

/// `a += b`, handy for running sums.
impl AddAssign for Complex {
    fn add_assign(&mut self, rhs: Complex) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

/// Division, given. It isn't needed until lesson 16.
/// a / b = a · conj(b) / |b|²
impl Div for Complex {
    type Output = Complex;
    fn div(self, rhs: Complex) -> Complex {
        let d = rhs.re * rhs.re + rhs.im * rhs.im;
        Complex::new(
            (self.re * rhs.re + self.im * rhs.im) / d,
            (self.im * rhs.re - self.re * rhs.im) / d,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testutil::*;
    use std::f64::consts::PI;

    fn assert_complex(z: Complex, re: f64, im: f64) {
        assert_close(z.re, re, 1e-12);
        assert_close(z.im, im, 1e-12);
    }

    #[test]
    fn l08_add_is_given() {
        assert_complex(Complex::new(1.0, 2.0) + Complex::new(3.0, -5.0), 4.0, -3.0);
    }

    #[test]
    fn l08_sub_mul_scale() {
        let (a, b) = (Complex::new(1.0, 2.0), Complex::new(3.0, 4.0));
        assert_complex(a - b, -2.0, -2.0);
        assert_complex(a * b, -5.0, 10.0);
        assert_complex(Complex::new(0.0, 1.0) * Complex::new(0.0, 1.0), -1.0, 0.0); // j² = −1
        assert_complex(a * 0.5, 0.5, 1.0);
    }

    #[test]
    fn l08_add_assign() {
        let mut z = Complex::ZERO;
        z += Complex::new(1.0, 1.0);
        z += Complex::new(0.5, -2.0);
        assert_complex(z, 1.5, -1.0);
    }

    #[test]
    fn l08_polar() {
        let z = Complex::new(3.0, 4.0);
        assert_close(z.abs(), 5.0, 1e-12);
        assert_close(z.norm_sqr(), 25.0, 1e-12);
        assert_close(Complex::new(0.0, 2.0).arg(), PI / 2.0, 1e-12);
        assert_close(Complex::new(-1.0, 0.0).arg(), PI, 1e-12);
        assert_complex(z.conj(), 3.0, -4.0);
        assert_complex(Complex::from_polar(2.0, PI / 2.0), 0.0, 2.0);
    }

    #[test]
    fn l08_euler() {
        assert_complex(Complex::expj(0.0), 1.0, 0.0);
        assert_complex(Complex::expj(PI / 2.0), 0.0, 1.0);
        assert_complex(Complex::expj(PI), -1.0, 0.0); // e^(jπ) = −1
        // Multiplying unit-circle points adds their angles.
        assert_complex(Complex::expj(0.3) * Complex::expj(0.4), 0.7f64.cos(), 0.7f64.sin());
    }
}

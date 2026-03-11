//! Floating-point approximate equality matchers.

use crate::error::MatchError;
use crate::expectation::Expectation;

macro_rules! impl_float_matchers {
    ($ty:ty, $default_epsilon:expr, $suffix:literal) => {
        impl Expectation<$ty> {
            /// Asserts the value is approximately equal to the expected value.
            ///
            #[doc = concat!("Uses a default epsilon of `", $suffix, "`.")]
            /// Use [`to_approximately_equal_within`](Self::to_approximately_equal_within)
            /// for a custom epsilon.
            ///
            /// `NaN` values never compare as approximately equal (even to
            /// themselves). `INFINITY - INFINITY` produces `NaN`, so
            /// `INFINITY.to_approximately_equal(INFINITY)` also fails.
            ///
            /// # Errors
            ///
            /// Returns [`MatchError`] if the difference exceeds the epsilon.
            ///
            /// ```text
            /// expect!(measurement)
            ///   actual: 1.5
            /// expected: to approximately equal 2.0 (within 0.0000000001)
            /// ```
            ///
            /// # Examples
            ///
            /// ```
            /// use behave::Expectation;
            ///
            #[doc = concat!("let result = Expectation::new(0.1_", stringify!($ty), " + 0.2_", stringify!($ty), ", \"x\")")]
            #[doc = concat!("    .to_approximately_equal(0.3_", stringify!($ty), ");")]
            /// assert!(result.is_ok());
            /// ```
            pub fn to_approximately_equal(
                &self,
                expected: $ty,
            ) -> Result<(), MatchError> {
                self.to_approximately_equal_within(expected, $default_epsilon)
            }

            /// Asserts the value is approximately equal within a custom epsilon.
            ///
            /// # Errors
            ///
            /// Returns [`MatchError`] if the difference exceeds the epsilon.
            ///
            /// # Examples
            ///
            /// ```
            /// use behave::Expectation;
            ///
            #[doc = concat!("let result = Expectation::new(1.005_", stringify!($ty), ", \"x\")")]
            #[doc = concat!("    .to_approximately_equal_within(1.0_", stringify!($ty), ", 0.01_", stringify!($ty), ");")]
            /// assert!(result.is_ok());
            /// ```
            pub fn to_approximately_equal_within(
                &self,
                expected: $ty,
                epsilon: $ty,
            ) -> Result<(), MatchError> {
                let diff = (*self.value() - expected).abs();
                let is_match = diff <= epsilon;
                self.check(
                    is_match,
                    format!("to approximately equal {expected} (within {epsilon})"),
                )
            }
        }
    };
}

impl_float_matchers!(f64, 1e-10_f64, "1e-10");
impl_float_matchers!(f32, 1e-6_f32, "1e-6");

#[cfg(test)]
mod tests {
    use crate::Expectation;

    #[test]
    fn f64_approx_equal_pass() {
        assert!(Expectation::new(0.1_f64 + 0.2_f64, "x")
            .to_approximately_equal(0.3_f64)
            .is_ok());
    }

    #[test]
    fn f64_approx_equal_fail() {
        assert!(Expectation::new(1.0_f64, "x")
            .to_approximately_equal(2.0_f64)
            .is_err());
    }

    #[test]
    fn f64_approx_equal_negated() {
        assert!(Expectation::new(1.0_f64, "x")
            .negate()
            .to_approximately_equal(2.0_f64)
            .is_ok());
    }

    #[test]
    fn f64_nan_not_approx_equal() {
        assert!(Expectation::new(f64::NAN, "x")
            .to_approximately_equal(0.0_f64)
            .is_err());
    }

    #[test]
    fn f64_infinity() {
        assert!(Expectation::new(f64::INFINITY, "x")
            .to_approximately_equal(f64::INFINITY)
            .is_err());
    }

    #[test]
    fn f64_negative_zero() {
        assert!(Expectation::new(-0.0_f64, "x")
            .to_approximately_equal(0.0_f64)
            .is_ok());
    }

    #[test]
    fn f64_within_pass() {
        assert!(Expectation::new(1.005_f64, "x")
            .to_approximately_equal_within(1.0_f64, 0.01_f64)
            .is_ok());
    }

    #[test]
    fn f64_within_fail() {
        assert!(Expectation::new(1.1_f64, "x")
            .to_approximately_equal_within(1.0_f64, 0.01_f64)
            .is_err());
    }

    #[test]
    fn f64_within_boundary_exact() {
        assert!(Expectation::new(1.0_f64, "x")
            .to_approximately_equal_within(1.0_f64, 0.0_f64)
            .is_ok());
    }

    #[test]
    fn f32_approx_equal_pass() {
        assert!(Expectation::new(0.1_f32 + 0.2_f32, "x")
            .to_approximately_equal(0.3_f32)
            .is_ok());
    }

    #[test]
    fn f32_approx_equal_fail() {
        assert!(Expectation::new(1.0_f32, "x")
            .to_approximately_equal(2.0_f32)
            .is_err());
    }

    #[test]
    fn f32_approx_equal_negated() {
        assert!(Expectation::new(1.0_f32, "x")
            .negate()
            .to_approximately_equal(2.0_f32)
            .is_ok());
    }

    #[test]
    fn f32_within_pass() {
        assert!(Expectation::new(1.005_f32, "x")
            .to_approximately_equal_within(1.0_f32, 0.01_f32)
            .is_ok());
    }

    #[test]
    fn f32_within_fail() {
        assert!(Expectation::new(1.1_f32, "x")
            .to_approximately_equal_within(1.0_f32, 0.01_f32)
            .is_err());
    }
}

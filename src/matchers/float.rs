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

macro_rules! impl_float_shape_matchers {
    ($ty:ty) => {
        impl Expectation<$ty> {
            /// Asserts the value is `NaN`.
            ///
            /// # Errors
            ///
            /// Returns [`MatchError`] if the value is not `NaN`.
            ///
            /// # Examples
            ///
            /// ```
            /// use behave::Expectation;
            ///
            #[doc = concat!("let result = Expectation::new(", stringify!($ty), "::NAN, \"x\").to_be_nan();")]
            /// assert!(result.is_ok());
            /// ```
            pub fn to_be_nan(&self) -> Result<(), MatchError> {
                self.check(self.value().is_nan(), "to be NaN")
            }

            /// Asserts the value is finite (not infinite and not `NaN`).
            ///
            /// # Errors
            ///
            /// Returns [`MatchError`] if the value is infinite or `NaN`.
            ///
            /// # Examples
            ///
            /// ```
            /// use behave::Expectation;
            ///
            #[doc = concat!("let result = Expectation::new(1.0_", stringify!($ty), ", \"x\").to_be_finite();")]
            /// assert!(result.is_ok());
            /// ```
            pub fn to_be_finite(&self) -> Result<(), MatchError> {
                self.check(self.value().is_finite(), "to be finite")
            }

            /// Asserts the value is positive or negative infinity.
            ///
            /// # Errors
            ///
            /// Returns [`MatchError`] if the value is finite or `NaN`.
            ///
            /// # Examples
            ///
            /// ```
            /// use behave::Expectation;
            ///
            #[doc = concat!("let result = Expectation::new(", stringify!($ty), "::INFINITY, \"x\").to_be_infinite();")]
            /// assert!(result.is_ok());
            /// ```
            pub fn to_be_infinite(&self) -> Result<(), MatchError> {
                self.check(self.value().is_infinite(), "to be infinite")
            }

            /// Asserts the value is strictly positive (greater than zero and not `NaN`).
            ///
            /// Note: `-0.0` is not considered positive.
            ///
            /// # Errors
            ///
            /// Returns [`MatchError`] if the value is zero, negative, or `NaN`.
            ///
            /// # Examples
            ///
            /// ```
            /// use behave::Expectation;
            ///
            #[doc = concat!("let result = Expectation::new(1.0_", stringify!($ty), ", \"x\").to_be_positive();")]
            /// assert!(result.is_ok());
            /// ```
            pub fn to_be_positive(&self) -> Result<(), MatchError> {
                let v = *self.value();
                self.check(v > 0.0 && !v.is_nan(), "to be positive")
            }

            /// Asserts the value is strictly negative (less than zero and not `NaN`).
            ///
            /// Note: `-0.0` is not considered negative.
            ///
            /// # Errors
            ///
            /// Returns [`MatchError`] if the value is zero, positive, or `NaN`.
            ///
            /// # Examples
            ///
            /// ```
            /// use behave::Expectation;
            ///
            #[doc = concat!("let result = Expectation::new(-1.0_", stringify!($ty), ", \"x\").to_be_negative();")]
            /// assert!(result.is_ok());
            /// ```
            pub fn to_be_negative(&self) -> Result<(), MatchError> {
                let v = *self.value();
                self.check(v < 0.0 && !v.is_nan(), "to be negative")
            }
        }
    };
}

impl_float_shape_matchers!(f64);
impl_float_shape_matchers!(f32);

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

    // --- Float shape matchers (f64) ---

    #[test]
    fn f64_to_be_nan_pass() {
        assert!(Expectation::new(f64::NAN, "x").to_be_nan().is_ok());
    }

    #[test]
    fn f64_to_be_nan_fail() {
        assert!(Expectation::new(1.0_f64, "x").to_be_nan().is_err());
    }

    #[test]
    fn f64_to_be_nan_negated() {
        assert!(Expectation::new(1.0_f64, "x").negate().to_be_nan().is_ok());
    }

    #[test]
    fn f64_to_be_finite_pass() {
        assert!(Expectation::new(1.0_f64, "x").to_be_finite().is_ok());
    }

    #[test]
    fn f64_to_be_finite_fail_infinity() {
        assert!(Expectation::new(f64::INFINITY, "x").to_be_finite().is_err());
    }

    #[test]
    fn f64_to_be_finite_fail_nan() {
        assert!(Expectation::new(f64::NAN, "x").to_be_finite().is_err());
    }

    #[test]
    fn f64_to_be_finite_negated() {
        assert!(Expectation::new(f64::INFINITY, "x")
            .negate()
            .to_be_finite()
            .is_ok());
    }

    #[test]
    fn f64_to_be_infinite_pass() {
        assert!(Expectation::new(f64::INFINITY, "x")
            .to_be_infinite()
            .is_ok());
    }

    #[test]
    fn f64_to_be_infinite_neg_infinity() {
        assert!(Expectation::new(f64::NEG_INFINITY, "x")
            .to_be_infinite()
            .is_ok());
    }

    #[test]
    fn f64_to_be_infinite_fail() {
        assert!(Expectation::new(1.0_f64, "x").to_be_infinite().is_err());
    }

    #[test]
    fn f64_to_be_infinite_negated() {
        assert!(Expectation::new(1.0_f64, "x")
            .negate()
            .to_be_infinite()
            .is_ok());
    }

    #[test]
    fn f64_to_be_positive_pass() {
        assert!(Expectation::new(1.0_f64, "x").to_be_positive().is_ok());
    }

    #[test]
    fn f64_to_be_positive_infinity() {
        assert!(Expectation::new(f64::INFINITY, "x")
            .to_be_positive()
            .is_ok());
    }

    #[test]
    fn f64_to_be_positive_fail_zero() {
        assert!(Expectation::new(0.0_f64, "x").to_be_positive().is_err());
    }

    #[test]
    fn f64_to_be_positive_fail_negative() {
        assert!(Expectation::new(-1.0_f64, "x").to_be_positive().is_err());
    }

    #[test]
    fn f64_to_be_positive_fail_nan() {
        assert!(Expectation::new(f64::NAN, "x").to_be_positive().is_err());
    }

    #[test]
    fn f64_to_be_positive_fail_neg_zero() {
        assert!(Expectation::new(-0.0_f64, "x").to_be_positive().is_err());
    }

    #[test]
    fn f64_to_be_positive_negated() {
        assert!(Expectation::new(-1.0_f64, "x")
            .negate()
            .to_be_positive()
            .is_ok());
    }

    #[test]
    fn f64_to_be_negative_pass() {
        assert!(Expectation::new(-1.0_f64, "x").to_be_negative().is_ok());
    }

    #[test]
    fn f64_to_be_negative_neg_infinity() {
        assert!(Expectation::new(f64::NEG_INFINITY, "x")
            .to_be_negative()
            .is_ok());
    }

    #[test]
    fn f64_to_be_negative_fail_zero() {
        assert!(Expectation::new(0.0_f64, "x").to_be_negative().is_err());
    }

    #[test]
    fn f64_to_be_negative_fail_positive() {
        assert!(Expectation::new(1.0_f64, "x").to_be_negative().is_err());
    }

    #[test]
    fn f64_to_be_negative_fail_nan() {
        assert!(Expectation::new(f64::NAN, "x").to_be_negative().is_err());
    }

    #[test]
    fn f64_to_be_negative_fail_neg_zero() {
        assert!(Expectation::new(-0.0_f64, "x").to_be_negative().is_err());
    }

    #[test]
    fn f64_to_be_negative_negated() {
        assert!(Expectation::new(1.0_f64, "x")
            .negate()
            .to_be_negative()
            .is_ok());
    }

    // --- Float shape matchers (f32) ---

    #[test]
    fn f32_to_be_nan_pass() {
        assert!(Expectation::new(f32::NAN, "x").to_be_nan().is_ok());
    }

    #[test]
    fn f32_to_be_nan_fail() {
        assert!(Expectation::new(1.0_f32, "x").to_be_nan().is_err());
    }

    #[test]
    fn f32_to_be_finite_pass() {
        assert!(Expectation::new(1.0_f32, "x").to_be_finite().is_ok());
    }

    #[test]
    fn f32_to_be_finite_fail() {
        assert!(Expectation::new(f32::INFINITY, "x").to_be_finite().is_err());
    }

    #[test]
    fn f32_to_be_infinite_pass() {
        assert!(Expectation::new(f32::INFINITY, "x")
            .to_be_infinite()
            .is_ok());
    }

    #[test]
    fn f32_to_be_infinite_fail() {
        assert!(Expectation::new(1.0_f32, "x").to_be_infinite().is_err());
    }

    #[test]
    fn f32_to_be_positive_pass() {
        assert!(Expectation::new(1.0_f32, "x").to_be_positive().is_ok());
    }

    #[test]
    fn f32_to_be_positive_fail() {
        assert!(Expectation::new(-1.0_f32, "x").to_be_positive().is_err());
    }

    #[test]
    fn f32_to_be_negative_pass() {
        assert!(Expectation::new(-1.0_f32, "x").to_be_negative().is_ok());
    }

    #[test]
    fn f32_to_be_negative_fail() {
        assert!(Expectation::new(1.0_f32, "x").to_be_negative().is_err());
    }
}

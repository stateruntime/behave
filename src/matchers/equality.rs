//! Equality matchers.

use core::fmt::Debug;

use crate::error::MatchError;
use crate::expectation::Expectation;

impl<T: PartialEq + Debug> Expectation<T> {
    /// Asserts the value equals the expected value.
    ///
    /// Respects negation via [`.negate()`](Expectation::negate).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the values are not equal (or equal when negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(42, "42").to_equal(42);
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_equal(&self, expected: T) -> Result<(), MatchError> {
        let is_match = *self.value() == expected;
        self.check(is_match, format!("{expected:?}"))
    }

    /// Asserts the value does not equal the given value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the values are equal.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(1, "1").to_not_equal(2);
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_not_equal(&self, expected: T) -> Result<(), MatchError> {
        let is_match = *self.value() != expected;
        self.check(is_match, format!("not {expected:?}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    #[test]
    fn to_equal_pass() {
        let result = Expectation::new(42, "42").to_equal(42);
        assert!(result.is_ok());
    }

    #[test]
    fn to_equal_fail() {
        let result = Expectation::new(42, "42").to_equal(99);
        assert!(result.is_err());
    }

    #[test]
    fn to_equal_negated_pass() {
        let result = Expectation::new(42, "42").negate().to_equal(99);
        assert!(result.is_ok());
    }

    #[test]
    fn to_equal_negated_fail() {
        let result = Expectation::new(42, "42").negate().to_equal(42);
        assert!(result.is_err());
    }

    #[test]
    fn to_not_equal_pass() {
        let result = Expectation::new(1, "1").to_not_equal(2);
        assert!(result.is_ok());
    }

    #[test]
    fn to_not_equal_fail() {
        let result = Expectation::new(1, "1").to_not_equal(1);
        assert!(result.is_err());
    }

    #[test]
    fn to_not_equal_negated_pass() {
        let result = Expectation::new(1, "1").negate().to_not_equal(1);
        assert!(result.is_ok());
    }

    #[test]
    fn to_not_equal_negated_fail() {
        let result = Expectation::new(1, "1").negate().to_not_equal(2);
        assert!(result.is_err());
    }
}

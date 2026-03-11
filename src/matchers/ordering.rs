//! Ordering matchers for comparable values.

use core::fmt::Debug;

use crate::error::MatchError;
use crate::expectation::Expectation;

#[allow(clippy::needless_pass_by_value)]
impl<T: PartialOrd + Debug> Expectation<T> {
    /// Asserts the value is strictly greater than the given bound.
    ///
    /// Use [`to_be_at_least`](Self::to_be_at_least) for `>=`.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is less than or equal to the bound.
    ///
    /// ```text
    /// expect!(count)
    ///   actual: 3
    /// expected: to be greater than 5
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(10, "10").to_be_greater_than(5);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_greater_than(&self, bound: T) -> Result<(), MatchError> {
        let is_match = *self.value() > bound;
        self.check(is_match, format!("to be greater than {bound:?}"))
    }

    /// Asserts the value is less than the given bound.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the comparison fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(3, "3").to_be_less_than(5);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_less_than(&self, bound: T) -> Result<(), MatchError> {
        let is_match = *self.value() < bound;
        self.check(is_match, format!("to be less than {bound:?}"))
    }

    /// Asserts the value is greater than or equal to the given bound.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the comparison fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(5, "5").to_be_at_least(5);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_at_least(&self, bound: T) -> Result<(), MatchError> {
        let is_match = *self.value() >= bound;
        self.check(is_match, format!("to be at least {bound:?}"))
    }

    /// Asserts the value is less than or equal to the given bound.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the comparison fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(5, "5").to_be_at_most(5);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_at_most(&self, bound: T) -> Result<(), MatchError> {
        let is_match = *self.value() <= bound;
        self.check(is_match, format!("to be at most {bound:?}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    #[test]
    fn greater_than_pass() {
        assert!(Expectation::new(10, "x").to_be_greater_than(5).is_ok());
    }

    #[test]
    fn greater_than_fail_equal() {
        assert!(Expectation::new(5, "x").to_be_greater_than(5).is_err());
    }

    #[test]
    fn greater_than_fail_less() {
        assert!(Expectation::new(3, "x").to_be_greater_than(5).is_err());
    }

    #[test]
    fn greater_than_negated() {
        assert!(Expectation::new(3, "x")
            .negate()
            .to_be_greater_than(5)
            .is_ok());
    }

    #[test]
    fn less_than_pass() {
        assert!(Expectation::new(3, "x").to_be_less_than(5).is_ok());
    }

    #[test]
    fn less_than_fail_equal() {
        assert!(Expectation::new(5, "x").to_be_less_than(5).is_err());
    }

    #[test]
    fn less_than_fail_greater() {
        assert!(Expectation::new(10, "x").to_be_less_than(5).is_err());
    }

    #[test]
    fn less_than_negated() {
        assert!(Expectation::new(10, "x")
            .negate()
            .to_be_less_than(5)
            .is_ok());
    }

    #[test]
    fn at_least_pass_greater() {
        assert!(Expectation::new(10, "x").to_be_at_least(5).is_ok());
    }

    #[test]
    fn at_least_pass_equal() {
        assert!(Expectation::new(5, "x").to_be_at_least(5).is_ok());
    }

    #[test]
    fn at_least_fail() {
        assert!(Expectation::new(3, "x").to_be_at_least(5).is_err());
    }

    #[test]
    fn at_least_negated() {
        assert!(Expectation::new(3, "x").negate().to_be_at_least(5).is_ok());
    }

    #[test]
    fn at_most_pass_less() {
        assert!(Expectation::new(3, "x").to_be_at_most(5).is_ok());
    }

    #[test]
    fn at_most_pass_equal() {
        assert!(Expectation::new(5, "x").to_be_at_most(5).is_ok());
    }

    #[test]
    fn at_most_fail() {
        assert!(Expectation::new(10, "x").to_be_at_most(5).is_err());
    }

    #[test]
    fn at_most_negated() {
        assert!(Expectation::new(10, "x").negate().to_be_at_most(5).is_ok());
    }
}

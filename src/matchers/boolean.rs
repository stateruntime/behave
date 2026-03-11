//! Boolean matchers.

use crate::error::MatchError;
use crate::expectation::Expectation;

impl Expectation<bool> {
    /// Asserts the value is `true`.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is `false` (or `true` when negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(true, "true").to_be_true();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_true(&self) -> Result<(), MatchError> {
        self.check(*self.value(), "true")
    }

    /// Asserts the value is `false`.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is `true` (or `false` when negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(false, "false").to_be_false();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_false(&self) -> Result<(), MatchError> {
        self.check(!*self.value(), "false")
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    #[test]
    fn to_be_true_pass() {
        assert!(Expectation::new(true, "t").to_be_true().is_ok());
    }

    #[test]
    fn to_be_true_fail() {
        assert!(Expectation::new(false, "f").to_be_true().is_err());
    }

    #[test]
    fn to_be_true_negated_pass() {
        assert!(Expectation::new(false, "f").negate().to_be_true().is_ok());
    }

    #[test]
    fn to_be_true_negated_fail() {
        assert!(Expectation::new(true, "t").negate().to_be_true().is_err());
    }

    #[test]
    fn to_be_false_pass() {
        assert!(Expectation::new(false, "f").to_be_false().is_ok());
    }

    #[test]
    fn to_be_false_fail() {
        assert!(Expectation::new(true, "t").to_be_false().is_err());
    }

    #[test]
    fn to_be_false_negated_pass() {
        assert!(Expectation::new(true, "t").negate().to_be_false().is_ok());
    }

    #[test]
    fn to_be_false_negated_fail() {
        assert!(Expectation::new(false, "f").negate().to_be_false().is_err());
    }
}

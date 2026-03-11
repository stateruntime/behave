//! Option matchers.

use core::fmt::Debug;

use crate::error::MatchError;
use crate::expectation::Expectation;

impl<T: Debug> Expectation<Option<T>> {
    /// Asserts the option is `Some`.
    ///
    /// Use [`to_be_some_with`](Expectation::to_be_some_with) when you also
    /// need to check the inner value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is `None` (or `Some` when negated).
    ///
    /// ```text
    /// expect!(result)
    ///   actual: None
    /// expected: to be Some(_)
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(Some(1), "Some(1)").to_be_some();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_some(&self) -> Result<(), MatchError> {
        self.check(self.value().is_some(), "to be Some(_)")
    }

    /// Asserts the option is `None`.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is `Some` (or `None` when negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(None::<i32>, "None").to_be_none();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_none(&self) -> Result<(), MatchError> {
        self.check(self.value().is_none(), "to be None")
    }
}

impl<T: Debug + PartialEq> Expectation<Option<T>> {
    /// Asserts the option is `Some` containing the expected value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the option is `None` or contains a different value.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(Some(42), "Some(42)").to_be_some_with(42);
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_be_some_with(&self, expected: T) -> Result<(), MatchError> {
        let is_match = self.value().as_ref() == Some(&expected);
        self.check(is_match, format!("to be Some({expected:?})"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    #[test]
    fn to_be_some_pass() {
        assert!(Expectation::new(Some(1), "x").to_be_some().is_ok());
    }

    #[test]
    fn to_be_some_fail() {
        assert!(Expectation::new(None::<i32>, "x").to_be_some().is_err());
    }

    #[test]
    fn to_be_some_negated_pass() {
        assert!(Expectation::new(None::<i32>, "x")
            .negate()
            .to_be_some()
            .is_ok());
    }

    #[test]
    fn to_be_some_negated_fail() {
        assert!(Expectation::new(Some(1), "x")
            .negate()
            .to_be_some()
            .is_err());
    }

    #[test]
    fn to_be_none_pass() {
        assert!(Expectation::new(None::<i32>, "x").to_be_none().is_ok());
    }

    #[test]
    fn to_be_none_fail() {
        assert!(Expectation::new(Some(1), "x").to_be_none().is_err());
    }

    #[test]
    fn to_be_none_negated_pass() {
        assert!(Expectation::new(Some(1), "x").negate().to_be_none().is_ok());
    }

    #[test]
    fn to_be_none_negated_fail() {
        assert!(Expectation::new(None::<i32>, "x")
            .negate()
            .to_be_none()
            .is_err());
    }

    #[test]
    fn to_be_some_with_pass() {
        assert!(Expectation::new(Some(42), "x").to_be_some_with(42).is_ok());
    }

    #[test]
    fn to_be_some_with_wrong_value() {
        assert!(Expectation::new(Some(42), "x").to_be_some_with(99).is_err());
    }

    #[test]
    fn to_be_some_with_none() {
        assert!(Expectation::new(None::<i32>, "x")
            .to_be_some_with(42)
            .is_err());
    }

    #[test]
    fn to_be_some_with_negated_pass() {
        assert!(Expectation::new(Some(42), "x")
            .negate()
            .to_be_some_with(99)
            .is_ok());
    }

    #[test]
    fn to_be_some_with_negated_fail() {
        assert!(Expectation::new(Some(42), "x")
            .negate()
            .to_be_some_with(42)
            .is_err());
    }
}

//! Result matchers.

use core::fmt::Debug;

use crate::error::MatchError;
use crate::expectation::Expectation;

impl<T: Debug, E: Debug> Expectation<Result<T, E>> {
    /// Asserts the result is `Ok`.
    ///
    /// Use [`to_be_ok_with`](Expectation::to_be_ok_with) when you also
    /// need to check the inner value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is `Err` (or `Ok` when negated).
    ///
    /// ```text
    /// expect!(response)
    ///   actual: Err("not found")
    /// expected: to be Ok(_)
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let val: Result<i32, &str> = Ok(1);
    /// let result = Expectation::new(val, "Ok(1)").to_be_ok();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_ok(&self) -> Result<(), MatchError> {
        self.check(self.value().is_ok(), "to be Ok(_)")
    }

    /// Asserts the result is `Err`.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value is `Ok` (or `Err` when negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let val: Result<i32, &str> = Err("oops");
    /// let result = Expectation::new(val, "Err").to_be_err();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_err(&self) -> Result<(), MatchError> {
        self.check(self.value().is_err(), "to be Err(_)")
    }
}

impl<T: Debug, E: Debug> Expectation<Result<T, E>> {
    /// Asserts the result is `Ok` and the inner value satisfies a predicate.
    ///
    /// The `desc` argument appears in failure messages (e.g. `"to be positive"`).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the result is `Err` or the predicate returns `false`.
    ///
    /// ```text
    /// expect!(result)
    ///   actual: Ok(0)
    /// expected: to be Ok(_) and to be positive
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let val: Result<i32, &str> = Ok(42);
    /// let result = Expectation::new(val, "x")
    ///     .to_be_ok_and(|v| *v > 0, "to be positive");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_ok_and(
        &self,
        predicate: impl FnOnce(&T) -> bool,
        desc: &str,
    ) -> Result<(), MatchError> {
        let is_match = self.value().as_ref().is_ok_and(predicate);
        self.check(is_match, format!("to be Ok(_) and {desc}"))
    }

    /// Asserts the result is `Err` and the error value satisfies a predicate.
    ///
    /// The `desc` argument appears in failure messages (e.g. `"to contain 'timeout'"`).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the result is `Ok` or the predicate returns `false`.
    ///
    /// ```text
    /// expect!(result)
    ///   actual: Err("unknown")
    /// expected: to be Err(_) and to contain 'timeout'
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let val: Result<i32, String> = Err("timeout error".to_string());
    /// let result = Expectation::new(val, "x")
    ///     .to_be_err_and(|e| e.contains("timeout"), "to contain 'timeout'");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_err_and(
        &self,
        predicate: impl FnOnce(&E) -> bool,
        desc: &str,
    ) -> Result<(), MatchError> {
        let is_match = self.value().as_ref().is_err_and(predicate);
        self.check(is_match, format!("to be Err(_) and {desc}"))
    }
}

impl<T: Debug + PartialEq, E: Debug> Expectation<Result<T, E>> {
    /// Asserts the result is `Ok` containing the expected value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the result is `Err` or contains a different value.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let val: Result<i32, &str> = Ok(42);
    /// let result = Expectation::new(val, "Ok(42)").to_be_ok_with(42);
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_be_ok_with(&self, expected: T) -> Result<(), MatchError> {
        let is_match = self.value().as_ref().is_ok_and(|val| *val == expected);
        self.check(is_match, format!("to be Ok({expected:?})"))
    }
}

impl<T: Debug, E: Debug + PartialEq> Expectation<Result<T, E>> {
    /// Asserts the result is `Err` containing the expected error value.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the result is `Ok` or contains a different error.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let val: Result<i32, &str> = Err("oops");
    /// let result = Expectation::new(val, "Err(oops)").to_be_err_with("oops");
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_be_err_with(&self, expected: E) -> Result<(), MatchError> {
        let is_match = self.value().as_ref().is_err_and(|err| *err == expected);
        self.check(is_match, format!("to be Err({expected:?})"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    #[test]
    fn to_be_ok_pass() {
        let val: Result<i32, &str> = Ok(1);
        assert!(Expectation::new(val, "x").to_be_ok().is_ok());
    }

    #[test]
    fn to_be_ok_fail() {
        let val: Result<i32, &str> = Err("e");
        assert!(Expectation::new(val, "x").to_be_ok().is_err());
    }

    #[test]
    fn to_be_ok_negated() {
        let val: Result<i32, &str> = Err("e");
        assert!(Expectation::new(val, "x").negate().to_be_ok().is_ok());
    }

    #[test]
    fn to_be_err_pass() {
        let val: Result<i32, &str> = Err("e");
        assert!(Expectation::new(val, "x").to_be_err().is_ok());
    }

    #[test]
    fn to_be_err_fail() {
        let val: Result<i32, &str> = Ok(1);
        assert!(Expectation::new(val, "x").to_be_err().is_err());
    }

    #[test]
    fn to_be_err_negated() {
        let val: Result<i32, &str> = Ok(1);
        assert!(Expectation::new(val, "x").negate().to_be_err().is_ok());
    }

    #[test]
    fn to_be_ok_with_pass() {
        let val: Result<i32, &str> = Ok(42);
        assert!(Expectation::new(val, "x").to_be_ok_with(42).is_ok());
    }

    #[test]
    fn to_be_ok_with_wrong_value() {
        let val: Result<i32, &str> = Ok(42);
        assert!(Expectation::new(val, "x").to_be_ok_with(99).is_err());
    }

    #[test]
    fn to_be_ok_with_err() {
        let val: Result<i32, &str> = Err("e");
        assert!(Expectation::new(val, "x").to_be_ok_with(42).is_err());
    }

    #[test]
    fn to_be_ok_with_negated_pass() {
        let val: Result<i32, &str> = Ok(42);
        assert!(Expectation::new(val, "x")
            .negate()
            .to_be_ok_with(99)
            .is_ok());
    }

    #[test]
    fn to_be_ok_with_negated_fail() {
        let val: Result<i32, &str> = Ok(42);
        assert!(Expectation::new(val, "x")
            .negate()
            .to_be_ok_with(42)
            .is_err());
    }

    #[test]
    fn to_be_err_with_pass() {
        let val: Result<i32, &str> = Err("oops");
        assert!(Expectation::new(val, "x").to_be_err_with("oops").is_ok());
    }

    #[test]
    fn to_be_err_with_wrong_value() {
        let val: Result<i32, &str> = Err("oops");
        assert!(Expectation::new(val, "x").to_be_err_with("other").is_err());
    }

    #[test]
    fn to_be_err_with_ok() {
        let val: Result<i32, &str> = Ok(1);
        assert!(Expectation::new(val, "x").to_be_err_with("e").is_err());
    }

    #[test]
    fn to_be_err_with_negated_pass() {
        let val: Result<i32, &str> = Err("oops");
        assert!(Expectation::new(val, "x")
            .negate()
            .to_be_err_with("other")
            .is_ok());
    }

    #[test]
    fn to_be_err_with_negated_fail() {
        let val: Result<i32, &str> = Err("oops");
        assert!(Expectation::new(val, "x")
            .negate()
            .to_be_err_with("oops")
            .is_err());
    }

    // --- to_be_ok_and ---

    #[test]
    fn to_be_ok_and_pass() {
        let val: Result<i32, &str> = Ok(42);
        assert!(Expectation::new(val, "x")
            .to_be_ok_and(|v| *v > 0, "to be positive")
            .is_ok());
    }

    #[test]
    fn to_be_ok_and_fail_err() {
        let val: Result<i32, &str> = Err("e");
        assert!(Expectation::new(val, "x")
            .to_be_ok_and(|v| *v > 0, "to be positive")
            .is_err());
    }

    #[test]
    fn to_be_ok_and_fail_predicate() {
        let val: Result<i32, &str> = Ok(-1);
        assert!(Expectation::new(val, "x")
            .to_be_ok_and(|v| *v > 0, "to be positive")
            .is_err());
    }

    #[test]
    fn to_be_ok_and_negated() {
        let val: Result<i32, &str> = Ok(-1);
        assert!(Expectation::new(val, "x")
            .negate()
            .to_be_ok_and(|v| *v > 0, "to be positive")
            .is_ok());
    }

    // --- to_be_err_and ---

    #[test]
    fn to_be_err_and_pass() {
        let val: Result<i32, String> = Err("timeout error".to_string());
        assert!(Expectation::new(val, "x")
            .to_be_err_and(|e| e.contains("timeout"), "to contain 'timeout'")
            .is_ok());
    }

    #[test]
    fn to_be_err_and_fail_ok() {
        let val: Result<i32, String> = Ok(1);
        assert!(Expectation::new(val, "x")
            .to_be_err_and(|e| e.contains("timeout"), "to contain 'timeout'")
            .is_err());
    }

    #[test]
    fn to_be_err_and_fail_predicate() {
        let val: Result<i32, String> = Err("other error".to_string());
        assert!(Expectation::new(val, "x")
            .to_be_err_and(|e| e.contains("timeout"), "to contain 'timeout'")
            .is_err());
    }

    #[test]
    fn to_be_err_and_negated() {
        let val: Result<i32, String> = Err("other".to_string());
        assert!(Expectation::new(val, "x")
            .negate()
            .to_be_err_and(|e| e.contains("timeout"), "to contain 'timeout'")
            .is_ok());
    }
}

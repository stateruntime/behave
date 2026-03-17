//! Display and Debug string matchers.

use core::fmt::{Debug, Display};

use crate::error::MatchError;
use crate::expectation::Expectation;

impl<T: Display + Debug> Expectation<T> {
    /// Asserts the value's [`Display`] output equals the expected string.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the display output does not match.
    ///
    /// ```text
    /// expect!(status)
    ///   actual: Active
    /// expected: to display as "Inactive"
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(42, "42").to_display_as("42");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_display_as(&self, expected: &str) -> Result<(), MatchError> {
        let display = format!("{}", self.value());
        let is_match = display == expected;
        self.check(is_match, format!("to display as {expected:?}"))
    }

    /// Asserts the value's [`Display`] output contains the given substring.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the display output does not contain the substring.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(42, "42").to_display_containing("4");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_display_containing(&self, substring: &str) -> Result<(), MatchError> {
        let display = format!("{}", self.value());
        let is_match = display.contains(substring);
        self.check(
            is_match,
            format!("to have Display output containing {substring:?}"),
        )
    }
}

impl<T: Debug> Expectation<T> {
    /// Asserts the value's [`Debug`] output contains the given substring.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the debug output does not contain the substring.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(vec![1, 2, 3], "v")
    ///     .to_debug_containing("[1, 2, 3]");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_debug_containing(&self, substring: &str) -> Result<(), MatchError> {
        let debug = format!("{:?}", self.value());
        let is_match = debug.contains(substring);
        self.check(
            is_match,
            format!("to have Debug output containing {substring:?}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    // --- to_display_as ---

    #[test]
    fn to_display_as_pass() {
        assert!(Expectation::new(42, "x").to_display_as("42").is_ok());
    }

    #[test]
    fn to_display_as_fail() {
        assert!(Expectation::new(42, "x").to_display_as("99").is_err());
    }

    #[test]
    fn to_display_as_negated() {
        assert!(Expectation::new(42, "x")
            .negate()
            .to_display_as("99")
            .is_ok());
    }

    // --- to_display_containing ---

    #[test]
    fn to_display_containing_pass() {
        assert!(Expectation::new(42, "x").to_display_containing("4").is_ok());
    }

    #[test]
    fn to_display_containing_fail() {
        assert!(Expectation::new(42, "x")
            .to_display_containing("9")
            .is_err());
    }

    #[test]
    fn to_display_containing_negated() {
        assert!(Expectation::new(42, "x")
            .negate()
            .to_display_containing("9")
            .is_ok());
    }

    // --- to_debug_containing ---

    #[test]
    fn to_debug_containing_pass() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_debug_containing("[1, 2, 3]")
            .is_ok());
    }

    #[test]
    fn to_debug_containing_fail() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .to_debug_containing("[9]")
            .is_err());
    }

    #[test]
    fn to_debug_containing_negated() {
        assert!(Expectation::new(vec![1, 2, 3], "v")
            .negate()
            .to_debug_containing("[9]")
            .is_ok());
    }
}

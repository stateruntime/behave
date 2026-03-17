//! Error source chain matchers.

use core::fmt::Debug;

use crate::error::MatchError;
use crate::expectation::Expectation;

impl<T: std::error::Error + Debug> Expectation<T> {
    /// Asserts the error has a source (i.e. [`std::error::Error::source`] returns `Some`).
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the error has no source.
    ///
    /// ```text
    /// expect!(err)
    ///   actual: MyError("oops")
    /// expected: to have a source error
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use std::fmt;
    /// use std::io;
    ///
    /// #[derive(Debug)]
    /// struct Wrapper(io::Error);
    ///
    /// impl fmt::Display for Wrapper {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "wrapper: {}", self.0)
    ///     }
    /// }
    ///
    /// impl std::error::Error for Wrapper {
    ///     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    ///         Some(&self.0)
    ///     }
    /// }
    ///
    /// let err = Wrapper(io::Error::other("inner"));
    /// let result = Expectation::new(err, "err").to_have_source();
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_source(&self) -> Result<(), MatchError> {
        let is_match = self.value().source().is_some();
        self.check(is_match, "to have a source error")
    }

    /// Asserts the error's source message contains the given substring.
    ///
    /// First checks that [`std::error::Error::source`] returns `Some`, then checks
    /// that the source's [`Display`](std::fmt::Display) output contains
    /// the substring.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the error has no source or the source
    /// message does not contain the substring.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    /// use std::fmt;
    /// use std::io;
    ///
    /// #[derive(Debug)]
    /// struct Wrapper(io::Error);
    ///
    /// impl fmt::Display for Wrapper {
    ///     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    ///         write!(f, "wrapper: {}", self.0)
    ///     }
    /// }
    ///
    /// impl std::error::Error for Wrapper {
    ///     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
    ///         Some(&self.0)
    ///     }
    /// }
    ///
    /// let err = Wrapper(io::Error::other("connection timeout"));
    /// let result = Expectation::new(err, "err")
    ///     .to_have_source_containing("timeout");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_source_containing(&self, substring: &str) -> Result<(), MatchError> {
        let is_match = self
            .value()
            .source()
            .is_some_and(|src| src.to_string().contains(substring));
        self.check(
            is_match,
            format!("to have source error containing {substring:?}"),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;
    use std::fmt;
    use std::io;

    #[derive(Debug)]
    struct Wrapper(io::Error);

    impl fmt::Display for Wrapper {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "wrapper: {}", self.0)
        }
    }

    impl std::error::Error for Wrapper {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            Some(&self.0)
        }
    }

    #[derive(Debug)]
    struct NoSource;

    impl fmt::Display for NoSource {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "no source")
        }
    }

    impl std::error::Error for NoSource {}

    // --- to_have_source ---

    #[test]
    fn to_have_source_pass() {
        let err = Wrapper(io::Error::other("inner"));
        assert!(Expectation::new(err, "x").to_have_source().is_ok());
    }

    #[test]
    fn to_have_source_fail() {
        assert!(Expectation::new(NoSource, "x").to_have_source().is_err());
    }

    #[test]
    fn to_have_source_negated() {
        assert!(Expectation::new(NoSource, "x")
            .negate()
            .to_have_source()
            .is_ok());
    }

    // --- to_have_source_containing ---

    #[test]
    fn to_have_source_containing_pass() {
        let err = Wrapper(io::Error::other("connection timeout"));
        assert!(Expectation::new(err, "x")
            .to_have_source_containing("timeout")
            .is_ok());
    }

    #[test]
    fn to_have_source_containing_fail_no_source() {
        assert!(Expectation::new(NoSource, "x")
            .to_have_source_containing("anything")
            .is_err());
    }

    #[test]
    fn to_have_source_containing_fail_wrong_msg() {
        let err = Wrapper(io::Error::other("permission denied"));
        assert!(Expectation::new(err, "x")
            .to_have_source_containing("timeout")
            .is_err());
    }

    #[test]
    fn to_have_source_containing_negated() {
        let err = Wrapper(io::Error::other("permission denied"));
        assert!(Expectation::new(err, "x")
            .negate()
            .to_have_source_containing("timeout")
            .is_ok());
    }
}

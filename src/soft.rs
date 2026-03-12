//! Soft assertion support for collecting multiple failures.
//!
//! Normal assertions with `?` stop at the first failure. Soft assertions
//! let you check several expectations and see *all* failures at once —
//! useful when validating multiple fields of a struct or multiple
//! conditions that are independently meaningful.
//!
//! Use [`SoftErrors::check`] instead of `?` for each assertion, then call
//! [`SoftErrors::finish`] at the end to either succeed or report every
//! collected failure with numbered output.
//!
//! # When to use soft assertions
//!
//! - Validating multiple independent properties of a value
//! - Form validation tests where you want to see all failing fields
//! - Integration tests checking several response fields at once
//!
//! # When to use hard assertions (`?`)
//!
//! - When later assertions depend on earlier ones succeeding
//! - When a single failure makes the rest meaningless
//!
//! # Example output on failure
//!
//! ```text
//! 2 soft assertions failed:
//!
//! [1] expect!(name)
//!       actual: ""
//!     expected: to not be empty
//!
//! [2] expect!(age)
//!       actual: -1
//!     expected: to be greater than 0
//! ```

use std::fmt;

use crate::MatchError;

/// Collects [`MatchError`]s from soft assertions and reports them together.
///
/// Instead of propagating each assertion failure immediately with `?`, pass
/// results to [`check`](Self::check). At the end of the test, call
/// [`finish`](Self::finish) to either succeed or return all collected errors
/// as a [`SoftMatchError`].
///
/// You can freely mix hard assertions (`?`) and soft assertions in the same
/// test. Hard assertions still fail immediately; soft assertions are deferred.
///
/// # Examples
///
/// All assertions pass:
///
/// ```
/// use behave::prelude::*;
///
/// fn demo() -> Result<(), Box<dyn std::error::Error>> {
///     let mut errors = SoftErrors::new();
///     errors.check(expect!(2 + 2).to_equal(4));
///     errors.check(expect!(true).to_be_true());
///     errors.finish()?;
///     Ok(())
/// }
///
/// assert!(demo().is_ok());
/// ```
///
/// Failures are collected and reported together:
///
/// ```
/// use behave::prelude::*;
///
/// let mut errors = SoftErrors::new();
/// errors.check(expect!(1).to_equal(1));   // passes — not collected
/// errors.check(expect!(2).to_equal(99));  // fails — collected
/// errors.check(expect!(3).to_equal(88));  // fails — collected
///
/// let result = errors.finish();
/// assert!(result.is_err());
///
/// let msg = result.unwrap_err().to_string();
/// assert!(msg.contains("2 soft assertions failed"));
/// assert!(msg.contains("[1]"));
/// assert!(msg.contains("[2]"));
/// ```
#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub struct SoftErrors {
    collected: Vec<MatchError>,
}

impl SoftErrors {
    /// Creates an empty soft error collector.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::SoftErrors;
    ///
    /// let errors = SoftErrors::new();
    /// assert!(errors.is_empty());
    /// ```
    pub const fn new() -> Self {
        Self {
            collected: Vec::new(),
        }
    }

    /// Records a matcher result, collecting any error for later reporting.
    ///
    /// Pass the return value of any matcher method directly — no `?` needed.
    /// Passing results silently collect failures; successes are ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::prelude::*;
    ///
    /// let mut errors = SoftErrors::new();
    /// errors.check(expect!(1).to_equal(1));   // passes — not collected
    /// errors.check(expect!(2).to_equal(99));  // fails — collected
    /// assert_eq!(errors.len(), 1);
    /// ```
    pub fn check(&mut self, result: Result<(), MatchError>) {
        if let Err(e) = result {
            self.collected.push(e);
        }
    }

    /// Finishes the soft assertion block and reports all collected failures.
    ///
    /// Returns `Ok(())` if every checked assertion passed. Returns
    /// [`SoftMatchError`] if any assertions failed — use `?` to propagate
    /// it as a test failure.
    ///
    /// This method consumes the collector. Create a new [`SoftErrors`] for
    /// each independent validation block.
    ///
    /// # Errors
    ///
    /// Returns [`SoftMatchError`] when one or more assertions failed.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::prelude::*;
    ///
    /// // All pass — finish() returns Ok
    /// let mut errors = SoftErrors::new();
    /// errors.check(expect!(1).to_equal(1));
    /// assert!(errors.finish().is_ok());
    ///
    /// // One fails — finish() returns Err
    /// let mut errors = SoftErrors::new();
    /// errors.check(expect!(1).to_equal(99));
    /// assert!(errors.finish().is_err());
    /// ```
    pub fn finish(self) -> Result<(), SoftMatchError> {
        if self.collected.is_empty() {
            return Ok(());
        }
        Err(SoftMatchError {
            errors: self.collected,
        })
    }

    /// Returns `true` if no failures have been collected.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::SoftErrors;
    ///
    /// let errors = SoftErrors::new();
    /// assert!(errors.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.collected.is_empty()
    }

    /// Returns the number of collected failures.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::prelude::*;
    ///
    /// let mut errors = SoftErrors::new();
    /// errors.check(expect!(1).to_equal(99));
    /// assert_eq!(errors.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.collected.len()
    }

    /// Returns a slice of all collected errors.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::prelude::*;
    ///
    /// let mut errors = SoftErrors::new();
    /// errors.check(expect!(1).to_equal(99));
    /// assert_eq!(errors.errors().len(), 1);
    /// ```
    pub fn errors(&self) -> &[MatchError] {
        &self.collected
    }
}

impl Default for SoftErrors {
    fn default() -> Self {
        Self::new()
    }
}

/// Error returned by [`SoftErrors::finish`] when assertions failed.
///
/// Contains all [`MatchError`]s that were collected during the soft
/// assertion block. The [`Display`](fmt::Display) output numbers each
/// failure for easy identification in test output.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
///
/// let mut errors = SoftErrors::new();
/// errors.check(expect!(1).to_equal(99));
/// let result = errors.finish();
/// assert!(result.is_err());
///
/// let msg = result.err().map(|e| e.to_string()).unwrap_or_default();
/// assert!(msg.contains("1 soft assertion failed"));
/// ```
#[derive(Debug)]
#[non_exhaustive]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub struct SoftMatchError {
    /// The collected match errors.
    pub errors: Vec<MatchError>,
}

impl fmt::Display for SoftMatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let count = self.errors.len();
        let noun = if count == 1 {
            "assertion"
        } else {
            "assertions"
        };
        writeln!(f, "{count} soft {noun} failed:")?;
        for (i, err) in self.errors.iter().enumerate() {
            write!(f, "\n[{}] {err}", i + 1)?;
            if i + 1 < count {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for SoftMatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_is_empty() {
        let errors = SoftErrors::new();
        assert!(errors.is_empty());
        assert_eq!(errors.len(), 0);
    }

    #[test]
    fn check_collects_errors() {
        let mut errors = SoftErrors::new();
        errors.check(Ok(()));
        assert!(errors.is_empty());

        let err = MatchError::new("x".to_string(), "1".to_string(), "2".to_string(), false);
        errors.check(Err(err));
        assert_eq!(errors.len(), 1);
        assert!(!errors.is_empty());
    }

    #[test]
    fn finish_ok_when_empty() {
        let errors = SoftErrors::new();
        assert!(errors.finish().is_ok());
    }

    #[test]
    fn finish_err_when_failures() {
        let mut errors = SoftErrors::new();
        let err = MatchError::new("x".to_string(), "1".to_string(), "2".to_string(), false);
        errors.check(Err(err));
        let result = errors.finish();
        assert!(result.is_err());
    }

    #[test]
    fn display_format() {
        let mut errors = SoftErrors::new();
        errors.check(Err(MatchError::new(
            "name".to_string(),
            "to not be empty".to_string(),
            "\"\"".to_string(),
            false,
        )));
        errors.check(Err(MatchError::new(
            "age".to_string(),
            "to be greater than 0".to_string(),
            "-1".to_string(),
            false,
        )));
        let result = errors.finish();
        assert!(result.is_err());
        let msg = result.err().map(|e| e.to_string()).unwrap_or_default();
        assert!(msg.contains("2 soft assertions failed:"));
        assert!(msg.contains("[1]"));
        assert!(msg.contains("[2]"));
        assert!(msg.contains("expect!(name)"));
        assert!(msg.contains("expect!(age)"));
    }

    #[test]
    fn errors_returns_slice() {
        let mut errors = SoftErrors::new();
        errors.check(Err(MatchError::new(
            "x".to_string(),
            "a".to_string(),
            "b".to_string(),
            false,
        )));
        assert_eq!(errors.errors().len(), 1);
        assert_eq!(errors.errors()[0].expression, "x");
    }

    #[test]
    fn default_is_empty() {
        let errors = SoftErrors::default();
        assert!(errors.is_empty());
    }
}

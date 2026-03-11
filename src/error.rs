//! Match error type for failed expectations.

use std::fmt;

/// Error returned when an expectation matcher fails.
///
/// Contains structured information about what was expected vs what was found,
/// enabling clear diagnostic output.
///
/// # Examples
///
/// ```
/// use behave::MatchError;
///
/// let err = MatchError::new(
///     "value".to_string(),
///     "42".to_string(),
///     "99".to_string(),
///     false,
/// );
/// assert!(err.to_string().contains("42"));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct MatchError {
    /// The stringified expression that was tested.
    pub expression: String,
    /// What the matcher expected to find.
    pub expected: String,
    /// What the matcher actually found.
    pub actual: String,
    /// Whether the expectation was negated with `.negate()`.
    pub negated: bool,
}

impl MatchError {
    /// Creates a new match error with the given details.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::MatchError;
    ///
    /// let err = MatchError::new(
    ///     "x".to_string(),
    ///     "true".to_string(),
    ///     "false".to_string(),
    ///     false,
    /// );
    /// assert_eq!(err.expression, "x");
    /// ```
    pub const fn new(expression: String, expected: String, actual: String, negated: bool) -> Self {
        Self {
            expression,
            expected,
            actual,
            negated,
        }
    }
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let negation = if self.negated { " not" } else { "" };
        write!(
            f,
            "expect!({})\n  actual: {}\nexpected{}: {}",
            self.expression, self.actual, negation, self.expected
        )
    }
}

impl std::error::Error for MatchError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_fields() {
        let err = MatchError::new(
            "x + 1".to_string(),
            "42".to_string(),
            "99".to_string(),
            false,
        );
        assert_eq!(err.expression, "x + 1");
        assert_eq!(err.expected, "42");
        assert_eq!(err.actual, "99");
        assert!(!err.negated);
    }

    #[test]
    fn display_normal() {
        let err = MatchError::new("val".to_string(), "42".to_string(), "99".to_string(), false);
        let msg = err.to_string();
        assert!(msg.contains("expect!(val)"));
        assert!(msg.contains("actual: 99"));
        assert!(msg.contains("expected: 42"));
        assert!(!msg.contains("not"));
    }

    #[test]
    fn display_negated() {
        let err = MatchError::new("val".to_string(), "42".to_string(), "42".to_string(), true);
        let msg = err.to_string();
        assert!(msg.contains("expected not: 42"));
    }

    #[test]
    fn error_source_is_none() {
        let err = MatchError::new("x".to_string(), "a".to_string(), "b".to_string(), false);
        assert!(std::error::Error::source(&err).is_none());
    }
}

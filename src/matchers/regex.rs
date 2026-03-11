//! Regex string matchers.
//!
//! Requires the `regex` feature flag.

use crate::error::MatchError;
use crate::expectation::Expectation;

/// Compiles a regex pattern, mapping compilation errors to [`MatchError`].
///
/// The `display_pattern` is shown in the error message (the user's original
/// pattern), while `raw_pattern` is what gets compiled (which may include
/// auto-anchoring).
fn compile_regex(
    expression: &str,
    raw_pattern: &str,
    display_pattern: &str,
) -> Result<::regex::Regex, MatchError> {
    ::regex::Regex::new(raw_pattern).map_err(|e| {
        MatchError::new(
            expression.to_string(),
            format!("valid regex pattern /{display_pattern}/"),
            format!("regex compilation failed: {e}"),
            false,
        )
    })
}

#[cfg_attr(docsrs, doc(cfg(feature = "regex")))]
impl<T: AsRef<str> + core::fmt::Debug> Expectation<T> {
    /// Asserts the string fully matches the given regex pattern.
    ///
    /// The pattern is auto-anchored with `^(?:...)$` so it must match the
    /// entire string, not just a substring. Use [`to_contain_regex`](Self::to_contain_regex)
    /// for substring matching.
    ///
    /// Invalid regex patterns produce a [`MatchError`] instead of panicking.
    ///
    /// Requires the `regex` feature.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the string does not match, or if the
    /// pattern is not a valid regular expression.
    ///
    /// ```text
    /// expect!(input)
    ///   actual: "abc"
    /// expected: to fully match /\d+/
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new("hello123", "s")
    ///     .to_match_regex(r"hello\d+");
    /// assert!(result.is_ok());
    ///
    /// // Partial matches fail — the entire string must match:
    /// let result = Expectation::new("abc123def", "s")
    ///     .to_match_regex(r"\d+");
    /// assert!(result.is_err());
    /// ```
    pub fn to_match_regex(&self, pattern: &str) -> Result<(), MatchError> {
        let anchored = format!("^(?:{pattern})$");
        let re = compile_regex(self.expression, &anchored, pattern)?;
        let is_match = re.is_match(self.value().as_ref());
        self.check(is_match, format!("to fully match /{pattern}/"))
    }

    /// Asserts the string contains a substring matching the given regex.
    ///
    /// The pattern is used unanchored, so it matches any substring.
    /// Use [`to_match_regex`](Self::to_match_regex) to require a full-string match.
    ///
    /// Requires the `regex` feature.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if no substring matches, or if the
    /// pattern is not a valid regular expression.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new("hello world 42", "s")
    ///     .to_contain_regex(r"\d+");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_regex(&self, pattern: &str) -> Result<(), MatchError> {
        let re = compile_regex(self.expression, pattern, pattern)?;
        let is_match = re.is_match(self.value().as_ref());
        self.check(is_match, format!("to contain match for /{pattern}/"))
    }
}

#[cfg(test)]
mod tests {
    use crate::error::MatchError;
    use crate::Expectation;

    #[test]
    fn to_match_regex_pass() {
        assert!(Expectation::new("hello123", "s")
            .to_match_regex(r"hello\d+")
            .is_ok());
    }

    #[test]
    fn to_match_regex_fail() {
        assert!(Expectation::new("hello", "s")
            .to_match_regex(r"^\d+$")
            .is_err());
    }

    #[test]
    fn to_match_regex_negated() {
        assert!(Expectation::new("hello", "s")
            .negate()
            .to_match_regex(r"\d+")
            .is_ok());
    }

    #[test]
    fn to_match_regex_requires_full_match() {
        assert!(Expectation::new("abc123def", "s")
            .to_match_regex(r"\d+")
            .is_err());
    }

    #[test]
    fn to_match_regex_already_anchored() {
        // User-provided anchors are redundant but harmless
        assert!(Expectation::new("hello", "s")
            .to_match_regex("^hello$")
            .is_ok());
    }

    #[test]
    fn to_contain_regex_pass() {
        assert!(Expectation::new("abc123def", "s")
            .to_contain_regex(r"\d+")
            .is_ok());
    }

    #[test]
    fn to_contain_regex_fail() {
        assert!(Expectation::new("hello", "s")
            .to_contain_regex(r"\d+")
            .is_err());
    }

    #[test]
    fn to_contain_regex_negated() {
        assert!(Expectation::new("hello", "s")
            .negate()
            .to_contain_regex(r"\d+")
            .is_ok());
    }

    #[test]
    fn invalid_regex_returns_error() {
        let result = Expectation::new("hello", "s").to_match_regex(r"[invalid");
        assert!(result.is_err());
        let err = result
            .err()
            .unwrap_or_else(|| MatchError::new(String::new(), String::new(), String::new(), false));
        assert!(err.actual.contains("regex compilation failed"));
        // Error should show user's original pattern, not the auto-anchored version
        assert!(err.expected.contains("/[invalid/"));
        assert!(!err.expected.contains("^(?:"));
    }

    #[test]
    fn invalid_regex_contain_returns_error() {
        let result = Expectation::new("hello", "s").to_contain_regex(r"[invalid");
        assert!(result.is_err());
    }

    #[test]
    fn to_match_regex_error_says_fully_match() {
        let result = Expectation::new("abc", "s").to_match_regex(r"\d+");
        let err = result
            .err()
            .unwrap_or_else(|| MatchError::new(String::new(), String::new(), String::new(), false));
        assert!(err.expected.contains("to fully match"));
    }

    #[test]
    fn to_match_regex_empty_pattern_matches_empty_string() {
        assert!(Expectation::new("", "s").to_match_regex("").is_ok());
    }

    #[test]
    fn to_contain_regex_empty_pattern_matches_anything() {
        assert!(Expectation::new("hello", "s").to_contain_regex("").is_ok());
    }
}

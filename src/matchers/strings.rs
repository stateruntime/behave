//! String matchers.

use crate::error::MatchError;
use crate::expectation::Expectation;

impl<T: AsRef<str> + core::fmt::Debug> Expectation<T> {
    /// Asserts the string starts with the given prefix.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the string does not start with the prefix.
    ///
    /// ```text
    /// expect!(greeting)
    ///   actual: "goodbye world"
    /// expected: to start with "hello"
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new("hello world", "s")
    ///     .to_start_with("hello");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_start_with(&self, prefix: &str) -> Result<(), MatchError> {
        let is_match = self.value().as_ref().starts_with(prefix);
        self.check(is_match, format!("to start with {prefix:?}"))
    }

    /// Asserts the string ends with the given suffix.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the string does not end with the suffix.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new("hello world", "s")
    ///     .to_end_with("world");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_end_with(&self, suffix: &str) -> Result<(), MatchError> {
        let is_match = self.value().as_ref().ends_with(suffix);
        self.check(is_match, format!("to end with {suffix:?}"))
    }

    /// Asserts the string contains the given substring.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the substring is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new("hello world", "s")
    ///     .to_contain_substr("lo wo");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_contain_substr(&self, substr: &str) -> Result<(), MatchError> {
        let is_match = self.value().as_ref().contains(substr);
        self.check(is_match, format!("to contain {substr:?}"))
    }

    /// Asserts the string has exactly the given byte length.
    ///
    /// Measures byte length ([`str::len`]), not character count. For ASCII
    /// strings, byte length equals character count. For multi-byte
    /// characters (e.g. emoji), byte length will be larger.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the length does not match.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// // ASCII: 1 byte per character
    /// let result = Expectation::new("abc", "s")
    ///     .to_have_str_length(3);
    /// assert!(result.is_ok());
    ///
    /// // Emoji: 4 bytes each
    /// let result = Expectation::new("\u{1F600}\u{1F601}", "s")
    ///     .to_have_str_length(8);
    /// assert!(result.is_ok());
    /// ```
    pub fn to_have_str_length(&self, expected: usize) -> Result<(), MatchError> {
        let actual_len = self.value().as_ref().len();
        let is_match = actual_len == expected;
        self.check(is_match, format!("to have length {expected}"))
    }
}

#[cfg(test)]
mod tests {
    use crate::Expectation;

    #[test]
    fn to_start_with_pass() {
        assert!(Expectation::new("hello world", "s")
            .to_start_with("hello")
            .is_ok());
    }

    #[test]
    fn to_start_with_fail() {
        assert!(Expectation::new("hello world", "s")
            .to_start_with("world")
            .is_err());
    }

    #[test]
    fn to_start_with_negated() {
        assert!(Expectation::new("hello", "s")
            .negate()
            .to_start_with("world")
            .is_ok());
    }

    #[test]
    fn to_start_with_empty_string() {
        assert!(Expectation::new("", "s").to_start_with("a").is_err());
    }

    #[test]
    fn to_start_with_empty_prefix() {
        assert!(Expectation::new("hello", "s").to_start_with("").is_ok());
    }

    #[test]
    fn to_end_with_pass() {
        assert!(Expectation::new("hello world", "s")
            .to_end_with("world")
            .is_ok());
    }

    #[test]
    fn to_end_with_fail() {
        assert!(Expectation::new("hello world", "s")
            .to_end_with("hello")
            .is_err());
    }

    #[test]
    fn to_end_with_negated() {
        assert!(Expectation::new("hello", "s")
            .negate()
            .to_end_with("world")
            .is_ok());
    }

    #[test]
    fn to_end_with_empty_string() {
        assert!(Expectation::new("", "s").to_end_with("a").is_err());
    }

    #[test]
    fn to_contain_substr_pass() {
        assert!(Expectation::new("hello world", "s")
            .to_contain_substr("lo wo")
            .is_ok());
    }

    #[test]
    fn to_contain_substr_fail() {
        assert!(Expectation::new("hello world", "s")
            .to_contain_substr("xyz")
            .is_err());
    }

    #[test]
    fn to_contain_substr_negated() {
        assert!(Expectation::new("hello", "s")
            .negate()
            .to_contain_substr("xyz")
            .is_ok());
    }

    #[test]
    fn to_contain_substr_empty() {
        assert!(Expectation::new("hello", "s").to_contain_substr("").is_ok());
    }

    #[test]
    fn to_have_str_length_pass() {
        assert!(Expectation::new("abc", "s").to_have_str_length(3).is_ok());
    }

    #[test]
    fn to_have_str_length_fail() {
        assert!(Expectation::new("abc", "s").to_have_str_length(5).is_err());
    }

    #[test]
    fn to_have_str_length_negated() {
        assert!(Expectation::new("abc", "s")
            .negate()
            .to_have_str_length(5)
            .is_ok());
    }

    #[test]
    fn to_have_str_length_zero() {
        assert!(Expectation::new("", "s").to_have_str_length(0).is_ok());
    }

    #[test]
    fn to_have_str_length_unicode() {
        // Multi-byte: each emoji is 4 bytes
        let s = "\u{1F600}\u{1F601}";
        assert!(Expectation::new(s, "s").to_have_str_length(8).is_ok());
    }
}

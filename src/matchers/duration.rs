//! Duration comparison matchers.

use std::time::Duration;

use crate::error::MatchError;
use crate::expectation::Expectation;

impl Expectation<Duration> {
    /// Asserts the duration is shorter than the given bound.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the duration is greater than or equal to the bound.
    ///
    /// ```text
    /// expect!(elapsed)
    ///   actual: 5s
    /// expected: to be shorter than 1s
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(Duration::from_millis(500), "d")
    ///     .to_be_shorter_than(Duration::from_secs(1));
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_shorter_than(&self, bound: Duration) -> Result<(), MatchError> {
        let is_match = *self.value() < bound;
        self.check(is_match, format!("to be shorter than {}", fmt_dur(bound)))
    }

    /// Asserts the duration is longer than the given bound.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the duration is less than or equal to the bound.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(Duration::from_secs(2), "d")
    ///     .to_be_longer_than(Duration::from_secs(1));
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_longer_than(&self, bound: Duration) -> Result<(), MatchError> {
        let is_match = *self.value() > bound;
        self.check(is_match, format!("to be longer than {}", fmt_dur(bound)))
    }

    /// Asserts the duration is within `tolerance` of the expected value.
    ///
    /// Checks that `|actual - expected| <= tolerance` using saturating
    /// arithmetic to avoid underflow.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the difference exceeds the tolerance.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(Duration::from_millis(1050), "d")
    ///     .to_be_close_to_duration(
    ///         Duration::from_secs(1),
    ///         Duration::from_millis(100),
    ///     );
    /// assert!(result.is_ok());
    /// ```
    pub fn to_be_close_to_duration(
        &self,
        expected: Duration,
        tolerance: Duration,
    ) -> Result<(), MatchError> {
        let actual = *self.value();
        let diff = abs_diff(actual, expected);
        let is_match = diff <= tolerance;
        self.check(
            is_match,
            format!(
                "to be within {} of {}",
                fmt_dur(tolerance),
                fmt_dur(expected),
            ),
        )
    }
}

/// Absolute difference between two durations using saturating subtraction.
fn abs_diff(a: Duration, b: Duration) -> Duration {
    if a >= b {
        a - b
    } else {
        b - a
    }
}

/// Formats a duration for human-readable error messages.
fn fmt_dur(d: Duration) -> String {
    format!("{d:?}")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::Expectation;

    // --- to_be_shorter_than ---

    #[test]
    fn shorter_than_pass() {
        assert!(Expectation::new(Duration::from_millis(500), "d")
            .to_be_shorter_than(Duration::from_secs(1))
            .is_ok());
    }

    #[test]
    fn shorter_than_fail_equal() {
        assert!(Expectation::new(Duration::from_secs(1), "d")
            .to_be_shorter_than(Duration::from_secs(1))
            .is_err());
    }

    #[test]
    fn shorter_than_fail_greater() {
        assert!(Expectation::new(Duration::from_secs(2), "d")
            .to_be_shorter_than(Duration::from_secs(1))
            .is_err());
    }

    #[test]
    fn shorter_than_negated() {
        assert!(Expectation::new(Duration::from_secs(2), "d")
            .negate()
            .to_be_shorter_than(Duration::from_secs(1))
            .is_ok());
    }

    // --- to_be_longer_than ---

    #[test]
    fn longer_than_pass() {
        assert!(Expectation::new(Duration::from_secs(2), "d")
            .to_be_longer_than(Duration::from_secs(1))
            .is_ok());
    }

    #[test]
    fn longer_than_fail_equal() {
        assert!(Expectation::new(Duration::from_secs(1), "d")
            .to_be_longer_than(Duration::from_secs(1))
            .is_err());
    }

    #[test]
    fn longer_than_fail_less() {
        assert!(Expectation::new(Duration::from_millis(500), "d")
            .to_be_longer_than(Duration::from_secs(1))
            .is_err());
    }

    #[test]
    fn longer_than_negated() {
        assert!(Expectation::new(Duration::from_millis(500), "d")
            .negate()
            .to_be_longer_than(Duration::from_secs(1))
            .is_ok());
    }

    // --- to_be_close_to_duration ---

    #[test]
    fn close_to_pass() {
        assert!(Expectation::new(Duration::from_millis(1050), "d")
            .to_be_close_to_duration(Duration::from_secs(1), Duration::from_millis(100),)
            .is_ok());
    }

    #[test]
    fn close_to_fail() {
        assert!(Expectation::new(Duration::from_millis(1200), "d")
            .to_be_close_to_duration(Duration::from_secs(1), Duration::from_millis(100),)
            .is_err());
    }

    #[test]
    fn close_to_exact() {
        assert!(Expectation::new(Duration::from_secs(1), "d")
            .to_be_close_to_duration(Duration::from_secs(1), Duration::from_millis(0),)
            .is_ok());
    }

    #[test]
    fn close_to_negated() {
        assert!(Expectation::new(Duration::from_millis(1200), "d")
            .negate()
            .to_be_close_to_duration(Duration::from_secs(1), Duration::from_millis(100),)
            .is_ok());
    }
}

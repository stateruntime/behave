//! Custom matcher trait for user-defined matchers.

use crate::error::MatchError;
use crate::expectation::Expectation;

/// Trait for implementing custom matchers.
///
/// Implement this trait on your own types to create reusable matchers
/// that work with [`Expectation::to_match`].
///
/// For one-off checks, prefer
/// [`to_satisfy(predicate, description)`](crate::Expectation::to_satisfy)
/// which takes a closure. Use `BehaveMatch` when you want a named,
/// reusable matcher — especially for domain rules used across many tests.
///
/// # Examples
///
/// ```
/// use behave::{BehaveMatch, Expectation, MatchError};
///
/// struct IsEven;
///
/// impl BehaveMatch<i32> for IsEven {
///     fn matches(&self, actual: &i32) -> bool {
///         actual % 2 == 0
///     }
///
///     fn description(&self) -> &str {
///         "to be even"
///     }
/// }
///
/// let result = Expectation::new(4, "4").to_match(IsEven);
/// assert!(result.is_ok());
/// ```
pub trait BehaveMatch<T> {
    /// Returns `true` if the actual value satisfies the matcher.
    fn matches(&self, actual: &T) -> bool;

    /// Returns a human-readable description of what the matcher expects.
    fn description(&self) -> &str;
}

impl<T> BehaveMatch<T> for Box<dyn BehaveMatch<T>> {
    fn matches(&self, actual: &T) -> bool {
        (**self).matches(actual)
    }

    fn description(&self) -> &str {
        (**self).description()
    }
}

impl<T: core::fmt::Debug> Expectation<T> {
    /// Asserts the value satisfies a custom [`BehaveMatch`] matcher.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the value does not match (or matches when negated).
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::{BehaveMatch, Expectation, MatchError};
    ///
    /// struct IsPositive;
    /// impl BehaveMatch<i32> for IsPositive {
    ///     fn matches(&self, actual: &i32) -> bool { *actual > 0 }
    ///     fn description(&self) -> &str { "to be positive" }
    /// }
    ///
    /// let result = Expectation::new(5, "5").to_match(IsPositive);
    /// assert!(result.is_ok());
    /// ```
    #[allow(clippy::needless_pass_by_value)]
    pub fn to_match(&self, custom: impl BehaveMatch<T>) -> Result<(), MatchError> {
        let is_match = custom.matches(self.value());
        self.check(is_match, custom.description())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IsEven;

    #[allow(clippy::unnecessary_literal_bound)]
    impl BehaveMatch<i32> for IsEven {
        fn matches(&self, actual: &i32) -> bool {
            actual % 2 == 0
        }

        fn description(&self) -> &str {
            "to be even"
        }
    }

    #[test]
    fn custom_matcher_pass() {
        assert!(Expectation::new(4, "4").to_match(IsEven).is_ok());
    }

    #[test]
    fn custom_matcher_fail() {
        assert!(Expectation::new(3, "3").to_match(IsEven).is_err());
    }

    #[test]
    fn custom_matcher_negated_pass() {
        assert!(Expectation::new(3, "3").negate().to_match(IsEven).is_ok());
    }

    #[test]
    fn custom_matcher_negated_fail() {
        assert!(Expectation::new(4, "4").negate().to_match(IsEven).is_err());
    }
}

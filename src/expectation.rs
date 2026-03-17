//! Core expectation type that wraps a value for matcher assertions.

use core::fmt;

use crate::error::MatchError;

/// Wraps a value with metadata for expressive assertions.
///
/// Created by the [`expect!`](crate::expect) macro. Chain matchers to assert
/// properties of the wrapped value.
/// Use [`.not()`](Self::not) or [`.negate()`](Self::negate) to invert any
/// matcher.
///
/// All matchers return `Result<(), MatchError>`, so use `?` to propagate
/// failures with clear diagnostics showing the expression, actual value,
/// and expected description.
///
/// # Examples
///
/// ```
/// use behave::Expectation;
///
/// let exp = Expectation::new(42, "42");
/// assert_eq!(*exp.value(), 42);
/// ```
#[derive(Debug)]
#[non_exhaustive]
pub struct Expectation<T> {
    value: T,
    /// The stringified source expression.
    pub expression: &'static str,
    /// Whether this expectation is negated.
    pub negated: bool,
    file: Option<&'static str>,
    line: Option<u32>,
}

impl<T> Expectation<T> {
    /// Creates a new expectation wrapping the given value.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let exp = Expectation::new(true, "true");
    /// assert!(!exp.negated);
    /// ```
    pub const fn new(value: T, expression: &'static str) -> Self {
        Self {
            value,
            expression,
            negated: false,
            file: None,
            line: None,
        }
    }

    /// Creates a new expectation with source location tracking.
    ///
    /// Prefer the [`expect!`](crate::expect) macro which calls this
    /// automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let exp = Expectation::new_located(true, "true", "test.rs", 1);
    /// assert!(!exp.negated);
    /// ```
    pub const fn new_located(
        value: T,
        expression: &'static str,
        file: &'static str,
        line: u32,
    ) -> Self {
        Self {
            value,
            expression,
            negated: false,
            file: Some(file),
            line: Some(line),
        }
    }

    /// Negates the expectation, flipping the pass/fail logic of matchers.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let exp = Expectation::new(true, "true").negate();
    /// assert!(exp.negated);
    /// ```
    #[must_use]
    pub const fn negate(mut self) -> Self {
        self.negated = !self.negated;
        self
    }

    /// Alias for [`negate`](Self::negate) for readability.
    ///
    /// **Note:** Double negation (`.not().not()`) cancels out and returns
    /// the original pass/fail semantics. This is rarely what you want —
    /// use a single `.not()` or omit it entirely.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let exp = Expectation::new(true, "true").not();
    /// assert!(exp.negated);
    /// ```
    #[must_use]
    pub const fn not(self) -> Self {
        self.negate()
    }

    /// Returns a reference to the wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let exp = Expectation::new(5, "5");
    /// assert_eq!(*exp.value(), 5);
    /// ```
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Consumes the expectation and returns the wrapped value.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let exp = Expectation::new(42, "42");
    /// assert_eq!(exp.into_value(), 42);
    /// ```
    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T: fmt::Debug> Expectation<T> {
    /// Checks the match result, applying negation logic and building a
    /// [`MatchError`] on failure.
    ///
    /// This is the shared core used by all matchers. The `expected`
    /// description is only evaluated (via `Display`) on the error path.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] when the (possibly negated) assertion fails.
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let exp = Expectation::new(42, "42");
    /// assert!(exp.check(true, "42").is_ok());
    /// assert!(exp.check(false, "99").is_err());
    /// ```
    pub fn check(&self, is_match: bool, expected: impl fmt::Display) -> Result<(), MatchError> {
        let pass = if self.negated { !is_match } else { is_match };

        if pass {
            return Ok(());
        }

        Err(MatchError::new(
            self.expression.to_string(),
            expected.to_string(),
            format!("{:?}", self.value()),
            self.negated,
        )
        .with_location(self.file, self.line))
    }

    /// Asserts the value satisfies a predicate.
    ///
    /// Use this for one-off checks when no built-in matcher fits.
    /// For reusable domain rules, consider implementing [`BehaveMatch`](crate::BehaveMatch)
    /// instead.
    ///
    /// The `description` appears in error messages using the standard
    /// "to ..." format (e.g. `"to be even"`, `"to be positive"`).
    /// Any type implementing [`Display`](core::fmt::Display) is accepted.
    ///
    /// # Errors
    ///
    /// Returns [`MatchError`] if the predicate returns `false`
    /// (or `true` when negated).
    ///
    /// ```text
    /// expect!(count)
    ///   actual: 7
    /// expected: to be even
    /// ```
    ///
    /// # Examples
    ///
    /// ```
    /// use behave::Expectation;
    ///
    /// let result = Expectation::new(7, "7").to_satisfy(|x| x % 2 != 0, "to be odd");
    /// assert!(result.is_ok());
    /// ```
    pub fn to_satisfy(
        &self,
        predicate: impl FnOnce(&T) -> bool,
        description: impl fmt::Display,
    ) -> Result<(), MatchError> {
        self.check(predicate(self.value()), description)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_stores_value_and_expression() {
        let exp = Expectation::new(42, "42");
        assert_eq!(*exp.value(), 42);
        assert_eq!(exp.expression, "42");
        assert!(!exp.negated);
    }

    #[test]
    fn negate_toggles_flag() {
        let exp = Expectation::new(1, "1").negate();
        assert!(exp.negated);
    }

    #[test]
    fn double_negate_returns_to_original() {
        let exp = Expectation::new(1, "1").negate().negate();
        assert!(!exp.negated);
    }

    #[test]
    fn not_is_alias_for_negate() {
        let exp = Expectation::new(1, "1").not();
        assert!(exp.negated);
    }

    #[test]
    fn value_returns_reference() {
        let exp = Expectation::new("hello", "s");
        assert_eq!(*exp.value(), "hello");
    }

    #[test]
    fn check_pass() {
        let exp = Expectation::new(42, "42");
        assert!(exp.check(true, "42").is_ok());
    }

    #[test]
    fn check_fail() {
        let exp = Expectation::new(42, "42");
        assert!(exp.check(false, "99").is_err());
    }

    #[test]
    fn check_negated_pass() {
        let exp = Expectation::new(42, "42").negate();
        assert!(exp.check(false, "99").is_ok());
    }

    #[test]
    fn check_negated_fail() {
        let exp = Expectation::new(42, "42").negate();
        assert!(exp.check(true, "42").is_err());
    }

    #[test]
    fn to_satisfy_pass() {
        assert!(Expectation::new(7, "7")
            .to_satisfy(|x| x % 2 != 0, "to be odd")
            .is_ok());
    }

    #[test]
    fn to_satisfy_fail() {
        assert!(Expectation::new(4, "4")
            .to_satisfy(|x| x % 2 != 0, "to be odd")
            .is_err());
    }

    #[test]
    fn to_satisfy_negated_pass() {
        assert!(Expectation::new(4, "4")
            .negate()
            .to_satisfy(|x| x % 2 != 0, "to be odd")
            .is_ok());
    }

    #[test]
    fn to_satisfy_negated_fail() {
        assert!(Expectation::new(7, "7")
            .negate()
            .to_satisfy(|x| x % 2 != 0, "to be odd")
            .is_err());
    }

    #[test]
    fn into_value_returns_inner() {
        let exp = Expectation::new(42, "42");
        assert_eq!(exp.into_value(), 42);
    }

    #[test]
    fn new_located_sets_fields() {
        let exp = Expectation::new_located(1, "1", "test.rs", 10);
        assert_eq!(*exp.value(), 1);
        assert_eq!(exp.expression, "1");
        assert!(!exp.negated);
    }

    #[test]
    fn to_satisfy_accepts_display_description() {
        let desc = format!("to be divisible by {}", 3);
        assert!(Expectation::new(9, "9")
            .to_satisfy(|x| x % 3 == 0, desc)
            .is_ok());
    }
}

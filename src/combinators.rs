//! Matcher combinators for composing multiple matchers.
//!
//! Combinators let you build complex assertions from simple matchers.
//! All three types implement [`BehaveMatch<T>`], so they compose recursively.
//!
//! | Function | Semantics |
//! |----------|-----------|
//! | [`all_of`] | All matchers must pass (empty = pass) |
//! | [`any_of`] | At least one must pass (empty = fail) |
//! | [`not_matching`] | Inverts a single matcher |
//!
//! # Examples
//!
//! ```
//! use behave::prelude::*;
//! use behave::combinators::{all_of, any_of, not_matching};
//!
//! struct GreaterThan(i32);
//! # #[allow(clippy::unnecessary_literal_bound)]
//! impl BehaveMatch<i32> for GreaterThan {
//!     fn matches(&self, actual: &i32) -> bool { *actual > self.0 }
//!     fn description(&self) -> &str { "to be greater than threshold" }
//! }
//!
//! struct LessThan(i32);
//! # #[allow(clippy::unnecessary_literal_bound)]
//! impl BehaveMatch<i32> for LessThan {
//!     fn matches(&self, actual: &i32) -> bool { *actual < self.0 }
//!     fn description(&self) -> &str { "to be less than threshold" }
//! }
//!
//! let matcher = all_of(vec![
//!     Box::new(GreaterThan(0)) as Box<dyn BehaveMatch<i32>>,
//!     Box::new(LessThan(100)),
//! ]);
//!
//! let result = Expectation::new(42, "42").to_match(matcher);
//! assert!(result.is_ok());
//! ```

use crate::custom::BehaveMatch;

/// Builds a bullet-list description from a prefix and matcher descriptions.
///
/// Produces output like:
/// ```text
/// to match all of:
///   - to be positive
///   - to be less than 100
/// ```
fn build_list_description(prefix: &str, matchers: &[Box<dyn BehaveMatch<impl Sized>>]) -> String {
    let mut desc = format!("{prefix}:");
    for m in matchers {
        let sub = m.description();
        desc.push_str("\n  - ");
        desc.push_str(&indent_subsequent_lines(sub, "    "));
    }
    desc
}

/// Indents all lines after the first by the given prefix.
///
/// This handles nested combinator descriptions where sub-descriptions
/// are multi-line (e.g. an `any_of` inside an `all_of`).
fn indent_subsequent_lines(text: &str, indent: &str) -> String {
    let mut lines = text.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let mut result = first.to_string();
    for line in lines {
        result.push('\n');
        result.push_str(indent);
        result.push_str(line);
    }
    result
}

// ---------------------------------------------------------------------------
// AllOf
// ---------------------------------------------------------------------------

/// Matcher that passes when **all** inner matchers pass.
///
/// Created by [`all_of`]. An empty list passes (vacuous truth).
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
/// use behave::combinators::all_of;
///
/// struct IsPositive;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsPositive {
///     fn matches(&self, actual: &i32) -> bool { *actual > 0 }
///     fn description(&self) -> &str { "to be positive" }
/// }
///
/// let matcher = all_of(vec![Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>]);
/// let result = Expectation::new(5, "5").to_match(matcher);
/// assert!(result.is_ok());
/// ```
#[non_exhaustive]
pub struct AllOf<T> {
    matchers: Vec<Box<dyn BehaveMatch<T>>>,
    description: String,
}

impl<T> core::fmt::Debug for AllOf<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AllOf")
            .field("description", &self.description)
            .field("matcher_count", &self.matchers.len())
            .finish()
    }
}

impl<T> BehaveMatch<T> for AllOf<T> {
    fn matches(&self, actual: &T) -> bool {
        self.matchers.iter().all(|m| m.matches(actual))
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Creates a matcher that passes when **all** inner matchers pass.
///
/// An empty list passes (vacuous truth), matching the semantics of
/// `Iterator::all` on an empty iterator.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
/// use behave::combinators::all_of;
///
/// struct IsPositive;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsPositive {
///     fn matches(&self, actual: &i32) -> bool { *actual > 0 }
///     fn description(&self) -> &str { "to be positive" }
/// }
///
/// struct IsEven;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsEven {
///     fn matches(&self, actual: &i32) -> bool { actual % 2 == 0 }
///     fn description(&self) -> &str { "to be even" }
/// }
///
/// let matcher = all_of(vec![
///     Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
///     Box::new(IsEven),
/// ]);
/// let result = Expectation::new(4, "4").to_match(matcher);
/// assert!(result.is_ok());
/// ```
pub fn all_of<T>(matchers: Vec<Box<dyn BehaveMatch<T>>>) -> AllOf<T> {
    let description = build_list_description("to match all of", &matchers);
    AllOf {
        matchers,
        description,
    }
}

// ---------------------------------------------------------------------------
// AnyOf
// ---------------------------------------------------------------------------

/// Matcher that passes when **at least one** inner matcher passes.
///
/// Created by [`any_of`]. An empty list fails.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
/// use behave::combinators::any_of;
///
/// struct IsZero;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsZero {
///     fn matches(&self, actual: &i32) -> bool { *actual == 0 }
///     fn description(&self) -> &str { "to be zero" }
/// }
///
/// struct IsPositive;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsPositive {
///     fn matches(&self, actual: &i32) -> bool { *actual > 0 }
///     fn description(&self) -> &str { "to be positive" }
/// }
///
/// let matcher = any_of(vec![
///     Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
///     Box::new(IsPositive),
/// ]);
/// let result = Expectation::new(0, "0").to_match(matcher);
/// assert!(result.is_ok());
/// ```
#[non_exhaustive]
pub struct AnyOf<T> {
    matchers: Vec<Box<dyn BehaveMatch<T>>>,
    description: String,
}

impl<T> core::fmt::Debug for AnyOf<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AnyOf")
            .field("description", &self.description)
            .field("matcher_count", &self.matchers.len())
            .finish()
    }
}

impl<T> BehaveMatch<T> for AnyOf<T> {
    fn matches(&self, actual: &T) -> bool {
        self.matchers.iter().any(|m| m.matches(actual))
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Creates a matcher that passes when **at least one** inner matcher passes.
///
/// An empty list fails, matching the semantics of `Iterator::any` on an
/// empty iterator.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
/// use behave::combinators::any_of;
///
/// struct IsZero;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsZero {
///     fn matches(&self, actual: &i32) -> bool { *actual == 0 }
///     fn description(&self) -> &str { "to be zero" }
/// }
///
/// struct IsNegative;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsNegative {
///     fn matches(&self, actual: &i32) -> bool { *actual < 0 }
///     fn description(&self) -> &str { "to be negative" }
/// }
///
/// let matcher = any_of(vec![
///     Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
///     Box::new(IsNegative),
/// ]);
/// let result = Expectation::new(-3, "-3").to_match(matcher);
/// assert!(result.is_ok());
/// ```
pub fn any_of<T>(matchers: Vec<Box<dyn BehaveMatch<T>>>) -> AnyOf<T> {
    let description = build_list_description("to match any of", &matchers);
    AnyOf {
        matchers,
        description,
    }
}

// ---------------------------------------------------------------------------
// NotMatching
// ---------------------------------------------------------------------------

/// Matcher that inverts a single inner matcher.
///
/// Created by [`not_matching`]. Use this inside combinators to negate
/// one matcher without affecting the outer expectation chain.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
/// use behave::combinators::not_matching;
///
/// struct IsEven;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsEven {
///     fn matches(&self, actual: &i32) -> bool { actual % 2 == 0 }
///     fn description(&self) -> &str { "to be even" }
/// }
///
/// let matcher = not_matching(Box::new(IsEven));
/// let result = Expectation::new(3, "3").to_match(matcher);
/// assert!(result.is_ok());
/// ```
#[non_exhaustive]
pub struct NotMatching<T> {
    inner: Box<dyn BehaveMatch<T>>,
    description: String,
}

impl<T> core::fmt::Debug for NotMatching<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("NotMatching")
            .field("description", &self.description)
            .finish_non_exhaustive()
    }
}

impl<T> BehaveMatch<T> for NotMatching<T> {
    fn matches(&self, actual: &T) -> bool {
        !self.inner.matches(actual)
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Creates a matcher that inverts a single inner matcher.
///
/// Use `not_matching` inside combinators to negate one matcher without
/// affecting the outer expectation. For negating the whole chain, use
/// [`.not()`](crate::Expectation::not) instead.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
/// use behave::combinators::not_matching;
///
/// struct IsEven;
/// # #[allow(clippy::unnecessary_literal_bound)]
/// impl BehaveMatch<i32> for IsEven {
///     fn matches(&self, actual: &i32) -> bool { actual % 2 == 0 }
///     fn description(&self) -> &str { "to be even" }
/// }
///
/// let matcher = not_matching(Box::new(IsEven));
/// let result = Expectation::new(7, "7").to_match(matcher);
/// assert!(result.is_ok());
/// ```
pub fn not_matching<T>(matcher: Box<dyn BehaveMatch<T>>) -> NotMatching<T> {
    let description = format!("not {}", matcher.description());
    NotMatching {
        inner: matcher,
        description,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Expectation;

    struct IsPositive;

    #[allow(clippy::unnecessary_literal_bound)]
    impl BehaveMatch<i32> for IsPositive {
        fn matches(&self, actual: &i32) -> bool {
            *actual > 0
        }
        fn description(&self) -> &str {
            "to be positive"
        }
    }

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

    struct IsZero;

    #[allow(clippy::unnecessary_literal_bound)]
    impl BehaveMatch<i32> for IsZero {
        fn matches(&self, actual: &i32) -> bool {
            *actual == 0
        }
        fn description(&self) -> &str {
            "to be zero"
        }
    }

    // -- all_of --

    #[test]
    fn all_of_all_pass() {
        let m = all_of(vec![
            Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsEven),
        ]);
        assert!(Expectation::new(4, "4").to_match(m).is_ok());
    }

    #[test]
    fn all_of_one_fails() {
        let m = all_of(vec![
            Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsEven),
        ]);
        assert!(Expectation::new(3, "3").to_match(m).is_err());
    }

    #[test]
    fn all_of_empty_passes() {
        let m: AllOf<i32> = all_of(vec![]);
        assert!(Expectation::new(99, "99").to_match(m).is_ok());
    }

    #[test]
    fn all_of_description_format() {
        let m = all_of(vec![
            Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsEven),
        ]);
        let desc = m.description();
        assert!(desc.contains("to match all of:"));
        assert!(desc.contains("- to be positive"));
        assert!(desc.contains("- to be even"));
    }

    // -- any_of --

    #[test]
    fn any_of_one_passes() {
        let m = any_of(vec![
            Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsPositive),
        ]);
        assert!(Expectation::new(5, "5").to_match(m).is_ok());
    }

    #[test]
    fn any_of_none_pass() {
        let m = any_of(vec![
            Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsEven),
        ]);
        assert!(Expectation::new(3, "3").to_match(m).is_err());
    }

    #[test]
    fn any_of_empty_fails() {
        let m: AnyOf<i32> = any_of(vec![]);
        assert!(Expectation::new(1, "1").to_match(m).is_err());
    }

    #[test]
    fn any_of_description_format() {
        let m = any_of(vec![
            Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsPositive),
        ]);
        let desc = m.description();
        assert!(desc.contains("to match any of:"));
        assert!(desc.contains("- to be zero"));
        assert!(desc.contains("- to be positive"));
    }

    // -- not_matching --

    #[test]
    fn not_matching_inverts_pass() {
        let m = not_matching(Box::new(IsEven));
        assert!(Expectation::new(3, "3").to_match(m).is_ok());
    }

    #[test]
    fn not_matching_inverts_fail() {
        let m = not_matching(Box::new(IsEven));
        assert!(Expectation::new(4, "4").to_match(m).is_err());
    }

    #[test]
    fn not_matching_description() {
        let m = not_matching(Box::new(IsEven));
        assert_eq!(m.description(), "not to be even");
    }

    // -- nested composition --

    #[test]
    fn nested_all_of_inside_any_of() {
        let inner = all_of(vec![
            Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsEven),
        ]);
        let m = any_of(vec![
            Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
            Box::new(inner),
        ]);
        // 4 is positive and even → inner passes → any_of passes
        assert!(Expectation::new(4, "4").to_match(m).is_ok());
    }

    #[test]
    fn nested_description_indentation() {
        let inner = all_of(vec![
            Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsEven),
        ]);
        let m = any_of(vec![
            Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
            Box::new(inner),
        ]);
        let desc = m.description();
        // The nested all_of description should be indented
        assert!(desc.contains("to match any of:"));
        assert!(desc.contains("- to match all of:"));
    }

    #[test]
    fn not_matching_inside_all_of_pass() {
        let m = all_of(vec![
            Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
            Box::new(not_matching(Box::new(IsEven))),
        ]);
        // 3 is positive and odd → passes
        assert!(Expectation::new(3, "3").to_match(m).is_ok());
    }

    #[test]
    fn not_matching_inside_all_of_fail() {
        let m = all_of(vec![
            Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
            Box::new(not_matching(Box::new(IsEven))),
        ]);
        // 4 is positive but even → fails
        assert!(Expectation::new(4, "4").to_match(m).is_err());
    }

    #[test]
    fn all_of_negated_via_expectation() {
        let m = all_of(vec![
            Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
            Box::new(IsEven),
        ]);
        // 3 fails all_of, so negated passes
        assert!(Expectation::new(3, "3").negate().to_match(m).is_ok());
    }

    #[test]
    fn boxed_matcher_delegates() {
        let boxed: Box<dyn BehaveMatch<i32>> = Box::new(IsEven);
        assert!(Expectation::new(4, "4").to_match(boxed).is_ok());
    }

    #[test]
    fn debug_impls() {
        let a = all_of::<i32>(vec![]);
        let formatted = format!("{a:?}");
        assert!(formatted.contains("AllOf"));

        let b = any_of::<i32>(vec![]);
        let formatted = format!("{b:?}");
        assert!(formatted.contains("AnyOf"));

        let c = not_matching(Box::new(IsEven));
        let formatted = format!("{c:?}");
        assert!(formatted.contains("NotMatching"));
    }
}

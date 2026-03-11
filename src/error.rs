//! Match error type for failed expectations.

use std::fmt;

/// Error returned when an expectation matcher fails.
///
/// Contains structured information about what was expected vs what was found.
/// The [`Display`](fmt::Display) output uses a three-line format:
///
/// ```text
/// expect!(expression)
///   actual: <what was found>
/// expected: <what was expected>
/// ```
///
/// When negated, the expected line reads `expected: not <description>`.
///
/// When the `color` feature is enabled, actual values appear in red and
/// expected values in green. Multiline values get a line-by-line diff
/// with `+`/`-` markers. The `NO_COLOR` environment variable disables
/// ANSI codes while preserving the diff format.
///
/// # Examples
///
/// ```
/// use behave::MatchError;
///
/// let err = MatchError::new(
///     "score".to_string(),
///     "to equal 100".to_string(),
///     "42".to_string(),
///     false,
/// );
/// let msg = err.to_string();
/// assert!(msg.contains("expect!(score)"));
/// assert!(msg.contains("42"));
/// assert!(msg.contains("to equal 100"));
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
        #[cfg(feature = "color")]
        {
            fmt_enhanced(f, self)
        }
        #[cfg(not(feature = "color"))]
        {
            fmt_plain(f, self)
        }
    }
}

#[cfg(not(feature = "color"))]
fn fmt_plain(f: &mut fmt::Formatter<'_>, err: &MatchError) -> fmt::Result {
    let negation = if err.negated { "not " } else { "" };
    write!(
        f,
        "expect!({})\n  actual: {}\nexpected: {}{}",
        err.expression, err.actual, negation, err.expected
    )
}

// --- color feature: structured diff with optional ANSI codes ---

#[cfg(feature = "color")]
mod ansi {
    pub(super) const RED: &str = "\x1b[31m";
    pub(super) const GREEN: &str = "\x1b[32m";
    pub(super) const RESET: &str = "\x1b[0m";
}

/// Returns `(red, green, reset)` ANSI codes, or empty strings when
/// color is disabled.
#[cfg(feature = "color")]
const fn color_codes(colorize: bool) -> (&'static str, &'static str, &'static str) {
    if colorize {
        (ansi::RED, ansi::GREEN, ansi::RESET)
    } else {
        ("", "", "")
    }
}

/// Checks the `NO_COLOR` environment variable per <https://no-color.org/>.
///
/// Returns `true` when ANSI codes should be emitted.
#[cfg(feature = "color")]
fn should_colorize() -> bool {
    std::env::var("NO_COLOR").map_or(true, |v| v.is_empty())
}

#[cfg(feature = "color")]
fn is_multiline(actual: &str, expected: &str) -> bool {
    actual.contains('\n') || expected.contains('\n')
}

#[cfg(feature = "color")]
fn fmt_enhanced(f: &mut fmt::Formatter<'_>, err: &MatchError) -> fmt::Result {
    let colorize = should_colorize();
    if err.negated || !is_multiline(&err.actual, &err.expected) {
        return fmt_single_line(f, err, colorize);
    }
    fmt_diff_header(f, err, colorize)?;
    fmt_diff_body(f, &err.actual, &err.expected, colorize)
}

#[cfg(feature = "color")]
fn fmt_single_line(f: &mut fmt::Formatter<'_>, err: &MatchError, colorize: bool) -> fmt::Result {
    let (red, green, reset) = color_codes(colorize);
    let negation = if err.negated { "not " } else { "" };
    write!(
        f,
        "expect!({})\n  actual: {red}{}{reset}\nexpected: {}{green}{}{reset}",
        err.expression, err.actual, negation, err.expected,
    )
}

#[cfg(feature = "color")]
fn fmt_diff_header(f: &mut fmt::Formatter<'_>, err: &MatchError, colorize: bool) -> fmt::Result {
    let (red, green, reset) = color_codes(colorize);
    writeln!(f, "expect!({})", err.expression)?;
    writeln!(f, "{red}--- actual{reset}")?;
    write!(f, "{green}+++ expected{reset}")
}

#[cfg(feature = "color")]
fn fmt_diff_body(
    f: &mut fmt::Formatter<'_>,
    actual: &str,
    expected: &str,
    colorize: bool,
) -> fmt::Result {
    let (red, green, reset) = color_codes(colorize);
    let diff = similar::TextDiff::from_lines(actual, expected);
    for change in diff.iter_all_changes() {
        let is_changed = match change.tag() {
            similar::ChangeTag::Delete => {
                write!(f, "\n{red}-")?;
                true
            }
            similar::ChangeTag::Insert => {
                write!(f, "\n{green}+")?;
                true
            }
            similar::ChangeTag::Equal => {
                write!(f, "\n ")?;
                false
            }
        };
        write!(f, "{}", change.value().trim_end_matches('\n'))?;
        if is_changed {
            write!(f, "{reset}")?;
        }
        if change.missing_newline() {
            write!(f, "\n\\ No newline at end")?;
        }
    }
    Ok(())
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

    #[cfg(not(feature = "color"))]
    #[test]
    fn display_normal() {
        let err = MatchError::new("val".to_string(), "42".to_string(), "99".to_string(), false);
        let msg = err.to_string();
        assert!(msg.contains("expect!(val)"));
        assert!(msg.contains("actual: 99"));
        assert!(msg.contains("expected: 42"));
        assert!(!msg.contains("not"));
    }

    #[cfg(not(feature = "color"))]
    #[test]
    fn display_negated() {
        let err = MatchError::new("val".to_string(), "42".to_string(), "42".to_string(), true);
        let msg = err.to_string();
        assert!(msg.contains("expected: not 42"));
    }

    #[test]
    fn error_source_is_none() {
        let err = MatchError::new("x".to_string(), "a".to_string(), "b".to_string(), false);
        assert!(std::error::Error::source(&err).is_none());
    }

    #[cfg(feature = "color")]
    mod color_tests {
        use super::*;

        // Note: these tests verify ANSI output and assume NO_COLOR is not set.
        // Run with `NO_COLOR=` (unset) to ensure they pass.

        #[test]
        fn single_line_has_ansi_codes() {
            let err = MatchError::new("val".to_string(), "42".to_string(), "99".to_string(), false);
            let msg = err.to_string();
            // Structural content always present
            assert!(msg.contains("expect!(val)"));
            assert!(msg.contains("actual:"));
            assert!(msg.contains("99"));
            assert!(msg.contains("expected:"));
            assert!(msg.contains("42"));
            // ANSI codes present when NO_COLOR is not set
            if should_colorize() {
                assert!(msg.contains("\x1b[31m99\x1b[0m"));
                assert!(msg.contains("\x1b[32m42\x1b[0m"));
            }
        }

        #[test]
        fn negated_uses_single_line_format() {
            let err = MatchError::new("val".to_string(), "42".to_string(), "42".to_string(), true);
            let msg = err.to_string();
            assert!(msg.contains("expected: not"));
            if should_colorize() {
                assert!(msg.contains("\x1b[31m"));
            }
        }

        #[test]
        fn multiline_shows_diff_markers() {
            let err = MatchError::new(
                "text".to_string(),
                "line1\nline2\n".to_string(),
                "line1\nchanged\n".to_string(),
                false,
            );
            let msg = err.to_string();
            // Structural diff markers always present
            assert!(msg.contains("--- actual"));
            assert!(msg.contains("+++ expected"));
            assert!(msg.contains("-changed"));
            assert!(msg.contains("+line2"));
            assert!(msg.contains(" line1"));
        }

        #[test]
        fn multiline_equal_lines_have_no_color() {
            let err = MatchError::new(
                "text".to_string(),
                "same\ndiff_expected\n".to_string(),
                "same\ndiff_actual\n".to_string(),
                false,
            );
            let msg = err.to_string();
            // Equal lines should appear with space prefix, no ANSI
            assert!(msg.contains("\n same"));
        }
    }
}

//! Parser for `cargo test` output.

use serde::Serialize;

/// The outcome of a single test.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::parser::TestOutcome;
///
/// let outcome = TestOutcome::Pass;
/// assert!(matches!(outcome, TestOutcome::Pass));
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[non_exhaustive]
pub enum TestOutcome {
    /// Test passed.
    Pass,
    /// Test failed.
    Fail,
    /// Test was ignored.
    Ignored,
}

/// A single parsed test result.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::parser::{TestResult, TestOutcome};
///
/// let result = TestResult::new("math::adds_numbers".to_string(), TestOutcome::Pass);
/// assert_eq!(result.full_name, "math::adds_numbers");
/// # }
/// ```
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct TestResult {
    /// The fully qualified test name (e.g. `module::test_name`).
    pub full_name: String,
    /// The outcome of the test.
    pub outcome: TestOutcome,
}

impl TestResult {
    /// Creates a new test result.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "cli")]
    /// # {
    /// use behave::cli::parser::{TestResult, TestOutcome};
    ///
    /// let result = TestResult::new("my_test".to_string(), TestOutcome::Pass);
    /// assert_eq!(result.full_name, "my_test");
    /// # }
    /// ```
    pub const fn new(full_name: String, outcome: TestOutcome) -> Self {
        Self { full_name, outcome }
    }
}

/// Parses lines of `cargo test` output into test results.
///
/// Recognizes lines matching the pattern `test <name> ... <ok|FAILED|ignored>`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::parser::parse_test_output;
///
/// let output = "test math::adds ... ok\ntest math::fails ... FAILED\n";
/// let results = parse_test_output(output);
/// assert_eq!(results.len(), 2);
/// # }
/// ```
pub fn parse_test_output(output: &str) -> Vec<TestResult> {
    output.lines().filter_map(parse_test_line).collect()
}

fn parse_test_line(line: &str) -> Option<TestResult> {
    let line = line.trim();

    if !line.starts_with("test ") {
        return None;
    }

    let rest = &line[5..];
    let (full_name, outcome) = parse_name_and_outcome(rest)?;

    Some(TestResult::new(full_name, outcome))
}

fn parse_name_and_outcome(rest: &str) -> Option<(String, TestOutcome)> {
    if let Some(name) = rest.strip_suffix(" ... ok") {
        return Some((name.trim().to_string(), TestOutcome::Pass));
    }
    if let Some(name) = rest.strip_suffix(" ... FAILED") {
        return Some((name.trim().to_string(), TestOutcome::Fail));
    }
    if let Some(name) = rest.strip_suffix(" ... ignored") {
        return Some((name.trim().to_string(), TestOutcome::Ignored));
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_passing_test() {
        let results = parse_test_output("test my_mod::my_test ... ok\n");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].full_name, "my_mod::my_test");
        assert_eq!(results[0].outcome, TestOutcome::Pass);
    }

    #[test]
    fn parse_failing_test() {
        let results = parse_test_output("test broken ... FAILED\n");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].outcome, TestOutcome::Fail);
    }

    #[test]
    fn parse_ignored_test() {
        let results = parse_test_output("test pending ... ignored\n");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].outcome, TestOutcome::Ignored);
    }

    #[test]
    fn skips_non_test_lines() {
        let output = "running 1 test\ntest foo ... ok\ntest result: ok.\n";
        let results = parse_test_output(output);
        assert_eq!(results.len(), 1);
    }
}

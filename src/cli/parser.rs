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
    /// Test was skipped at runtime via `skip_when!`.
    Skipped,
    /// Test initially failed but passed on retry.
    Flaky,
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
    /// The skip reason, if the test was skipped via `skip_when!`.
    pub skip_reason: Option<String>,
    /// Test duration in seconds (if available).
    pub duration_secs: Option<f64>,
    /// Captured failure message from test stdout (if any).
    pub failure_message: Option<String>,
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
        Self {
            full_name,
            outcome,
            skip_reason: None,
            duration_secs: None,
            failure_message: None,
        }
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

/// Scans `--show-output` stdout for `BEHAVE_SKIP` sentinels and
/// reclassifies matching `Pass` results as `Skipped`.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::parser::{TestResult, TestOutcome, reclassify_skipped};
///
/// let mut results = vec![TestResult::new("my::test".to_string(), TestOutcome::Pass)];
/// let stdout = "---- my::test stdout ----\nBEHAVE_SKIP: not on CI\n";
/// reclassify_skipped(&mut results, stdout);
/// assert!(matches!(results[0].outcome, TestOutcome::Skipped));
/// # }
/// ```
pub fn reclassify_skipped(results: &mut [TestResult], stdout: &str) {
    let sentinels = extract_skip_sentinels(stdout);
    let failures = extract_failure_messages(stdout);
    for result in results {
        if result.outcome == TestOutcome::Pass {
            if let Some(reason) = sentinels.get(result.full_name.as_str()) {
                result.outcome = TestOutcome::Skipped;
                result.skip_reason = Some((*reason).to_string());
            }
        }
        if result.outcome == TestOutcome::Fail {
            if let Some(msg) = failures.get(result.full_name.as_str()) {
                result.failure_message = Some((*msg).to_string());
            }
        }
    }
}

/// Extracts failure messages from `---- <name> stdout ----` blocks.
fn extract_failure_messages(stdout: &str) -> std::collections::HashMap<&str, &str> {
    let mut messages = std::collections::HashMap::new();
    let lines: Vec<&str> = stdout.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        if let Some(name) = lines[i]
            .strip_prefix("---- ")
            .and_then(|rest| rest.strip_suffix(" stdout ----"))
        {
            let start = i + 1;
            let mut end = start;
            while end < lines.len()
                && !lines[end].starts_with("---- ")
                && !lines[end].starts_with("failures:")
            {
                end += 1;
            }
            if end > start {
                let block = &stdout[line_offset(stdout, start)..line_offset(stdout, end)];
                let trimmed = block.trim();
                if !trimmed.is_empty() {
                    messages.insert(name, trimmed);
                }
            }
            i = end;
        } else {
            i += 1;
        }
    }
    messages
}

fn line_offset(text: &str, line_idx: usize) -> usize {
    text.lines()
        .take(line_idx)
        .map(|l| l.len() + 1) // +1 for newline
        .sum()
}

/// Extracts `(test_name, reason)` pairs from `--show-output` stdout blocks.
///
/// Looks for `---- <name> stdout ----` headers followed by `BEHAVE_SKIP: <reason>` lines.
fn extract_skip_sentinels(stdout: &str) -> std::collections::HashMap<&str, &str> {
    let mut sentinels = std::collections::HashMap::new();
    let mut current_test: Option<&str> = None;

    for line in stdout.lines() {
        if let Some(name) = line
            .strip_prefix("---- ")
            .and_then(|rest| rest.strip_suffix(" stdout ----"))
        {
            current_test = Some(name);
        } else if let Some(reason) = line.strip_prefix("BEHAVE_SKIP: ") {
            if let Some(name) = current_test {
                sentinels.insert(name, reason);
            }
        }
    }

    sentinels
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

    #[test]
    fn reclassify_with_sentinel() {
        let mut results = vec![TestResult::new("my::test".to_string(), TestOutcome::Pass)];
        let stdout = "---- my::test stdout ----\nBEHAVE_SKIP: not on CI\n";
        reclassify_skipped(&mut results, stdout);
        assert_eq!(results[0].outcome, TestOutcome::Skipped);
        assert_eq!(results[0].skip_reason.as_deref(), Some("not on CI"));
    }

    #[test]
    fn reclassify_without_sentinel_no_change() {
        let mut results = vec![TestResult::new("my::test".to_string(), TestOutcome::Pass)];
        let stdout = "---- my::test stdout ----\nsome other output\n";
        reclassify_skipped(&mut results, stdout);
        assert_eq!(results[0].outcome, TestOutcome::Pass);
        assert!(results[0].skip_reason.is_none());
    }

    #[test]
    fn reclassify_does_not_touch_failures() {
        let mut results = vec![TestResult::new("my::test".to_string(), TestOutcome::Fail)];
        let stdout = "---- my::test stdout ----\nBEHAVE_SKIP: reason\n";
        reclassify_skipped(&mut results, stdout);
        assert_eq!(results[0].outcome, TestOutcome::Fail);
    }

    #[test]
    fn reclassify_captures_reason() {
        let mut results = vec![TestResult::new("a::b".to_string(), TestOutcome::Pass)];
        let stdout = "---- a::b stdout ----\nBEHAVE_SKIP: requires feature X\n";
        reclassify_skipped(&mut results, stdout);
        assert_eq!(
            results[0].skip_reason.as_deref(),
            Some("requires feature X")
        );
    }

    #[test]
    fn reclassify_handles_multiple_sentinels() {
        let mut results = vec![
            TestResult::new("test::a".to_string(), TestOutcome::Pass),
            TestResult::new("test::b".to_string(), TestOutcome::Pass),
        ];
        let stdout = "---- test::a stdout ----\nBEHAVE_SKIP: reason a\n\
                       ---- test::b stdout ----\nBEHAVE_SKIP: reason b\n";
        reclassify_skipped(&mut results, stdout);
        assert_eq!(results[0].outcome, TestOutcome::Skipped);
        assert_eq!(results[0].skip_reason.as_deref(), Some("reason a"));
        assert_eq!(results[1].outcome, TestOutcome::Skipped);
        assert_eq!(results[1].skip_reason.as_deref(), Some("reason b"));
    }

    #[test]
    fn reclassify_ignores_malformed_sentinel() {
        let mut results = vec![TestResult::new("my::test".to_string(), TestOutcome::Pass)];
        let stdout = "---- my::test stdout ----\nBEHAVE_SKIP missing colon\n";
        reclassify_skipped(&mut results, stdout);
        assert_eq!(results[0].outcome, TestOutcome::Pass);
    }

    #[test]
    fn reclassify_empty_stdout_is_noop() {
        let mut results = vec![TestResult::new("my::test".to_string(), TestOutcome::Pass)];
        reclassify_skipped(&mut results, "");
        assert_eq!(results[0].outcome, TestOutcome::Pass);
    }

    #[test]
    fn reclassify_does_not_touch_ignored() {
        let mut results = vec![TestResult::new(
            "my::test".to_string(),
            TestOutcome::Ignored,
        )];
        let stdout = "---- my::test stdout ----\nBEHAVE_SKIP: reason\n";
        reclassify_skipped(&mut results, stdout);
        assert_eq!(results[0].outcome, TestOutcome::Ignored);
    }

    #[test]
    fn reclassify_reason_with_special_chars() {
        let mut results = vec![TestResult::new("my::test".to_string(), TestOutcome::Pass)];
        let stdout = "---- my::test stdout ----\nBEHAVE_SKIP: requires env $CI_TOKEN & API key\n";
        reclassify_skipped(&mut results, stdout);
        assert_eq!(
            results[0].skip_reason.as_deref(),
            Some("requires env $CI_TOKEN & API key")
        );
    }
}

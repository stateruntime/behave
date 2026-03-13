//! Machine-readable output formats for `cargo-behave`.

use std::io::Write;

use serde::Serialize;

use super::history::FlakyTest;
use super::parser::{TestOutcome, TestResult};
use super::tree::TreeNode;

/// A complete CLI report that can be rendered in multiple formats.
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct Report {
    /// Whether the underlying `cargo test` command succeeded.
    pub command_success: bool,
    /// Parsed test results in stable name order.
    pub tests: Vec<TestResult>,
    /// Hierarchical tree view of the parsed test results.
    pub tree: Vec<TreeNode>,
    /// Aggregated summary counts.
    pub summary: Summary,
    /// Flaky tests detected during this run.
    pub flaky_tests: Vec<FlakyTest>,
    /// Stderr captured from `cargo test`.
    pub stderr: String,
}

impl Report {
    /// Creates a CLI report with the required fields.
    #[must_use]
    pub const fn new(command_success: bool, tests: Vec<TestResult>, summary: Summary) -> Self {
        Self {
            command_success,
            tests,
            tree: Vec::new(),
            summary,
            flaky_tests: Vec::new(),
            stderr: String::new(),
        }
    }

    /// Adds a tree view to the report.
    #[must_use]
    pub fn with_tree(mut self, tree: Vec<TreeNode>) -> Self {
        self.tree = tree;
        self
    }

    /// Adds flaky test results to the report.
    #[must_use]
    pub fn with_flaky_tests(mut self, flaky: Vec<FlakyTest>) -> Self {
        self.flaky_tests = flaky;
        self
    }

    /// Adds captured stderr to the report.
    #[must_use]
    pub fn with_stderr(mut self, stderr: String) -> Self {
        self.stderr = stderr;
        self
    }
}

/// Aggregate counts for a report.
#[derive(Debug, Serialize)]
#[non_exhaustive]
pub struct Summary {
    /// Number of passed tests.
    pub passed: usize,
    /// Number of failed tests.
    pub failed: usize,
    /// Number of ignored tests.
    pub ignored: usize,
    /// Number of skipped tests (via `skip_when!`).
    pub skipped: usize,
    /// Total number of parsed tests.
    pub total: usize,
}

impl Summary {
    /// Creates a summary from raw counts.
    #[must_use]
    pub const fn new(
        passed: usize,
        failed: usize,
        ignored: usize,
        skipped: usize,
        total: usize,
    ) -> Self {
        Self {
            passed,
            failed,
            ignored,
            skipped,
            total,
        }
    }

    /// Builds a summary from parsed test results.
    #[must_use]
    pub fn from_results(results: &[TestResult]) -> Self {
        let passed = count_by_outcome(results, &TestOutcome::Pass);
        let failed = count_by_outcome(results, &TestOutcome::Fail);
        let ignored = count_by_outcome(results, &TestOutcome::Ignored);
        let skipped = count_by_outcome(results, &TestOutcome::Skipped);

        Self::new(passed, failed, ignored, skipped, results.len())
    }
}

/// Renders a report as JSON.
///
/// # Errors
///
/// Returns an IO error if writing fails.
pub fn render_json(writer: &mut impl Write, report: &Report) -> std::io::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, report)
        .map_err(|err| std::io::Error::other(err.to_string()))?;
    writeln!(writer)
}

/// Renders a report as `JUnit` XML.
///
/// # Errors
///
/// Returns an IO error if writing fails.
pub fn render_junit(writer: &mut impl Write, report: &Report) -> std::io::Result<()> {
    write_suite_open(writer, report)?;

    if report.tests.is_empty() && !report.command_success {
        render_command_failure_case(writer, &report.stderr)?;
    } else {
        for test in &report.tests {
            render_testcase(writer, test)?;
        }
    }

    write_system_err(writer, &report.stderr)?;
    writeln!(writer, "</testsuite>")
}

fn count_by_outcome(results: &[TestResult], expected: &TestOutcome) -> usize {
    results
        .iter()
        .filter(|result| &result.outcome == expected)
        .count()
}

fn write_suite_open(writer: &mut impl Write, report: &Report) -> std::io::Result<()> {
    let errors = usize::from(!report.command_success && report.tests.is_empty());
    let xml_skipped = report.summary.ignored + report.summary.skipped;

    writeln!(writer, r#"<?xml version="1.0" encoding="UTF-8"?>"#)?;
    writeln!(
        writer,
        r#"<testsuite name="cargo-behave" tests="{}" failures="{}" errors="{}" skipped="{}">"#,
        report.summary.total, report.summary.failed, errors, xml_skipped
    )
}

fn render_command_failure_case(writer: &mut impl Write, stderr: &str) -> std::io::Result<()> {
    writeln!(
        writer,
        r#"  <testcase classname="cargo-behave" name="cargo test">"#
    )?;
    writeln!(
        writer,
        r#"    <error message="cargo test failed before any test results were parsed">{}</error>"#,
        escape_xml(stderr)
    )?;
    writeln!(writer, "  </testcase>")
}

fn render_testcase(writer: &mut impl Write, test: &TestResult) -> std::io::Result<()> {
    let (classname, name) = split_test_name(&test.full_name);
    writeln!(
        writer,
        r#"  <testcase classname="{}" name="{}">"#,
        escape_xml(&classname),
        escape_xml(&name)
    )?;

    match test.outcome {
        TestOutcome::Pass => {}
        TestOutcome::Fail => {
            writeln!(
                writer,
                r#"    <failure message="test failed">test {} failed</failure>"#,
                escape_xml(&test.full_name)
            )?;
        }
        TestOutcome::Ignored => {
            writeln!(writer, r#"    <skipped message="ignored or pending" />"#)?;
        }
        TestOutcome::Skipped => {
            let reason = test.skip_reason.as_deref().unwrap_or("skipped at runtime");
            writeln!(
                writer,
                r#"    <skipped message="skipped: {}" />"#,
                escape_xml(reason)
            )?;
        }
    }

    writeln!(writer, "  </testcase>")
}

fn write_system_err(writer: &mut impl Write, stderr: &str) -> std::io::Result<()> {
    if stderr.trim().is_empty() {
        return Ok(());
    }

    writeln!(writer, "  <system-err>{}</system-err>", escape_xml(stderr))
}

fn split_test_name(full_name: &str) -> (String, String) {
    let mut parts: Vec<String> = full_name.split("::").map(strip_marker_prefixes).collect();
    let name = parts
        .pop()
        .unwrap_or_else(|| strip_marker_prefixes(full_name));

    if parts.is_empty() {
        return (String::new(), name);
    }

    (parts.join("::"), name)
}

fn strip_marker_prefixes(segment: &str) -> String {
    let mut s = segment
        .strip_prefix("__FOCUS__")
        .or_else(|| segment.strip_prefix("__PENDING__"))
        .unwrap_or(segment);

    // Strip `__TAG_xxx__` prefixes in a loop
    while let Some(rest) = s.strip_prefix("__TAG_") {
        if let Some(end_pos) = rest.find("__") {
            s = &rest[end_pos + 2..];
        } else {
            break;
        }
    }

    s.to_string()
}

fn escape_xml(input: &str) -> String {
    let mut escaped = String::new();

    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            _ => escaped.push(ch),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_report() -> Report {
        Report::new(
            true,
            vec![
                TestResult::new("suite::a".to_string(), TestOutcome::Pass),
                TestResult::new("suite::b".to_string(), TestOutcome::Ignored),
                TestResult::new("suite::c".to_string(), TestOutcome::Fail),
            ],
            Summary::new(1, 1, 1, 0, 3),
        )
        .with_flaky_tests(vec![FlakyTest::new("suite::c".to_string(), 8)])
    }

    #[test]
    fn summary_counts_results() {
        let report = sample_report();
        let summary = Summary::from_results(&report.tests);

        assert_eq!(summary.passed, 1);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.ignored, 1);
        assert_eq!(summary.total, 3);
    }

    #[test]
    fn renders_json_report() {
        let report = sample_report();
        let mut buffer = Vec::new();
        let result = render_json(&mut buffer, &report);

        assert!(result.is_ok());

        let output = String::from_utf8(buffer).unwrap_or_default();
        assert!(output.contains("\"command_success\": true"));
        assert!(output.contains("\"flaky_tests\""));
        assert!(output.contains("\"suite::c\""));
    }

    #[test]
    fn renders_junit_failures_and_skips() {
        let report = sample_report();
        let mut buffer = Vec::new();
        let result = render_junit(&mut buffer, &report);

        assert!(result.is_ok());

        let output = String::from_utf8(buffer).unwrap_or_default();
        assert!(output.contains(r#"<failure message="test failed">"#));
        assert!(output.contains(r#"<skipped message="ignored or pending" />"#));
        assert!(output.contains(r#"tests="3""#));
    }

    #[test]
    fn summary_counts_skipped_results() {
        let results = vec![
            TestResult::new("suite::a".to_string(), TestOutcome::Pass),
            TestResult::new("suite::b".to_string(), TestOutcome::Skipped),
            TestResult::new("suite::c".to_string(), TestOutcome::Skipped),
        ];
        let summary = Summary::from_results(&results);
        assert_eq!(summary.passed, 1);
        assert_eq!(summary.skipped, 2);
        assert_eq!(summary.total, 3);
    }

    #[test]
    fn renders_junit_skipped_testcase() {
        let mut test = TestResult::new("suite::skip_me".to_string(), TestOutcome::Skipped);
        test.skip_reason = Some("not on CI".to_string());
        let report = Report::new(true, vec![test], Summary::new(0, 0, 0, 1, 1));
        let mut buffer = Vec::new();
        let result = render_junit(&mut buffer, &report);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap_or_default();
        assert!(output.contains(r#"<skipped message="skipped: not on CI" />"#));
    }

    #[test]
    fn renders_json_skipped_outcome() {
        let report = Report::new(
            true,
            vec![TestResult::new(
                "suite::a".to_string(),
                TestOutcome::Skipped,
            )],
            Summary::new(0, 0, 0, 1, 1),
        );
        let mut buffer = Vec::new();
        let result = render_json(&mut buffer, &report);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap_or_default();
        assert!(output.contains("\"Skipped\""));
    }

    #[test]
    fn renders_junit_command_failure_case() {
        let report = Report::new(false, Vec::new(), Summary::new(0, 0, 0, 0, 0))
            .with_stderr("compile error".to_string());
        let mut buffer = Vec::new();
        let result = render_junit(&mut buffer, &report);

        assert!(result.is_ok());

        let output = String::from_utf8(buffer).unwrap_or_default();
        assert!(output.contains(
            r#"<error message="cargo test failed before any test results were parsed">"#
        ));
        assert!(output.contains("compile error"));
        assert!(output.contains(r#"errors="1""#));
    }

    #[test]
    fn escapes_xml_characters() {
        let escaped = escape_xml(r#"<tag attr="x&y">"#);
        assert_eq!(escaped, "&lt;tag attr=&quot;x&amp;y&quot;&gt;");
    }

    #[test]
    fn splits_test_name_without_internal_markers() {
        let (classname, name) = split_test_name("__FOCUS__checkout::__PENDING__alpha_case");

        assert_eq!(classname, "checkout");
        assert_eq!(name, "alpha_case");
    }

    #[test]
    fn strips_tag_prefixes_from_test_name() {
        let (classname, name) =
            split_test_name("__TAG_slow____TAG_integration__suite::__TAG_unit__my_test");

        assert_eq!(classname, "suite");
        assert_eq!(name, "my_test");
    }

    #[test]
    fn strips_focus_and_tag_combined() {
        let (classname, name) = split_test_name("__FOCUS____TAG_critical__suite::my_test");

        assert_eq!(classname, "suite");
        assert_eq!(name, "my_test");
    }

    #[test]
    fn junit_skipped_count_includes_ignored_and_skipped() {
        let report = Report::new(
            true,
            vec![
                TestResult::new("a".to_string(), TestOutcome::Ignored),
                TestResult::new("b".to_string(), TestOutcome::Skipped),
            ],
            Summary::new(0, 0, 1, 1, 2),
        );
        let mut buffer = Vec::new();
        let result = render_junit(&mut buffer, &report);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap_or_default();
        assert!(output.contains(r#"skipped="2""#));
    }

    #[test]
    fn junit_skip_reason_escapes_xml() {
        let mut test = TestResult::new("suite::t".to_string(), TestOutcome::Skipped);
        test.skip_reason = Some("needs <env> & $TOKEN".to_string());
        let report = Report::new(true, vec![test], Summary::new(0, 0, 0, 1, 1));
        let mut buffer = Vec::new();
        let result = render_junit(&mut buffer, &report);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap_or_default();
        assert!(output.contains("needs &lt;env&gt; &amp; $TOKEN"));
    }

    #[test]
    fn summary_from_results_counts_all_outcomes() {
        let results = vec![
            TestResult::new("a".to_string(), TestOutcome::Pass),
            TestResult::new("b".to_string(), TestOutcome::Pass),
            TestResult::new("c".to_string(), TestOutcome::Fail),
            TestResult::new("d".to_string(), TestOutcome::Ignored),
            TestResult::new("e".to_string(), TestOutcome::Skipped),
            TestResult::new("f".to_string(), TestOutcome::Skipped),
        ];
        let summary = Summary::from_results(&results);
        assert_eq!(summary.passed, 2);
        assert_eq!(summary.failed, 1);
        assert_eq!(summary.ignored, 1);
        assert_eq!(summary.skipped, 2);
        assert_eq!(summary.total, 6);
    }

    #[test]
    fn renders_json_with_skip_reason() {
        let mut test = TestResult::new("suite::a".to_string(), TestOutcome::Skipped);
        test.skip_reason = Some("not on CI".to_string());
        let report = Report::new(true, vec![test], Summary::new(0, 0, 0, 1, 1));
        let mut buffer = Vec::new();
        let result = render_json(&mut buffer, &report);
        assert!(result.is_ok());
        let output = String::from_utf8(buffer).unwrap_or_default();
        assert!(output.contains("\"not on CI\""));
    }

    #[test]
    fn split_test_name_single_segment() {
        let (classname, name) = split_test_name("standalone_test");
        assert_eq!(classname, "");
        assert_eq!(name, "standalone_test");
    }

    #[test]
    fn split_test_name_deep_nesting() {
        let (classname, name) = split_test_name("a::b::c::d::leaf");
        assert_eq!(classname, "a::b::c::d");
        assert_eq!(name, "leaf");
    }

    #[test]
    fn escape_xml_handles_apostrophes() {
        assert_eq!(escape_xml("it's"), "it&apos;s");
    }
}

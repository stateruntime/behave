//! Test runner that invokes `cargo test` and captures output.

use std::process::{Command, Output};

use super::error::CliError;

/// Runs `cargo test` with forwarded cargo and libtest arguments.
///
/// # Errors
///
/// Returns [`CliError::CargoInvocation`] if the process cannot be spawned.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::runner::run_cargo_test;
///
/// let output = run_cargo_test(&["my_test"], &["--nocapture"]);
/// # }
/// ```
pub fn run_cargo_test(cargo_args: &[&str], test_args: &[&str]) -> Result<Output, CliError> {
    validate_test_args(test_args)?;

    let mut cmd = Command::new("cargo");
    for arg in build_cargo_test_args(cargo_args, test_args) {
        cmd.arg(arg);
    }

    cmd.output()
        .map_err(|source| CliError::CargoInvocation { source })
}

/// Lists all test names by running `cargo test -- --list --format terse`.
///
/// # Errors
///
/// Returns [`CliError::CargoInvocation`] if the process cannot be spawned.
///
/// # Examples
///
/// ```no_run
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::runner::list_tests;
///
/// let names = list_tests(&[]);
/// # }
/// ```
pub fn list_tests(cargo_args: &[&str]) -> Result<Vec<String>, CliError> {
    let mut cmd = Command::new("cargo");
    cmd.arg("test").arg("--all-features");

    for arg in cargo_args {
        cmd.arg(*arg);
    }

    cmd.arg("--").arg("--list").arg("--format").arg("terse");

    let output = cmd
        .output()
        .map_err(|source| CliError::CargoInvocation { source })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let tests = stdout
        .lines()
        .filter_map(|line| line.strip_suffix(": test").map(str::to_string))
        .collect();

    Ok(tests)
}

/// Filters test names to those containing the `__FOCUS__` marker.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "cli")]
/// # {
/// use behave::cli::runner::find_focused_tests;
///
/// let names = vec![
///     "suite::__FOCUS__test_a".to_string(),
///     "suite::test_b".to_string(),
/// ];
/// let focused = find_focused_tests(&names);
/// assert_eq!(focused.len(), 1);
/// # }
/// ```
pub fn find_focused_tests(test_names: &[String]) -> Vec<String> {
    test_names
        .iter()
        .filter(|name| name.contains("__FOCUS__"))
        .cloned()
        .collect()
}

fn validate_test_args(test_args: &[&str]) -> Result<(), CliError> {
    for arg in test_args {
        if *arg == "--format" || arg.starts_with("--format=") {
            return Err(CliError::UnsupportedLibtestArg {
                arg: (*arg).to_string(),
            });
        }
    }

    Ok(())
}

fn build_cargo_test_args(cargo_args: &[&str], test_args: &[&str]) -> Vec<String> {
    let mut args = vec!["test".to_string(), "--all-features".to_string()];

    for arg in cargo_args {
        args.push((*arg).to_string());
    }

    args.push("--".to_string());
    args.push("--format".to_string());
    // `pretty` preserves per-test lines like `test suite::case ... ok`, which
    // the CLI needs for tree, JSON, and JUnit reports.
    args.push("pretty".to_string());
    // `--show-output` captures passing test stdout so skip_when! sentinels
    // can be detected by `reclassify_skipped`.
    args.push("--show-output".to_string());

    for arg in test_args {
        args.push((*arg).to_string());
    }

    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_default_cargo_test_args() {
        let args = build_cargo_test_args(&[], &[]);
        assert_eq!(
            args,
            vec![
                "test",
                "--all-features",
                "--",
                "--format",
                "pretty",
                "--show-output"
            ]
        );
    }

    #[test]
    fn builds_args_with_filter_and_test_flags() {
        let args = build_cargo_test_args(&["checkout"], &["--nocapture"]);
        assert_eq!(
            args,
            vec![
                "test",
                "--all-features",
                "checkout",
                "--",
                "--format",
                "pretty",
                "--show-output",
                "--nocapture",
            ]
        );
    }

    #[test]
    fn preserves_cargo_level_flags() {
        let args = build_cargo_test_args(&["--package", "demo"], &["--ignored"]);
        assert_eq!(
            args,
            vec![
                "test",
                "--all-features",
                "--package",
                "demo",
                "--",
                "--format",
                "pretty",
                "--show-output",
                "--ignored",
            ]
        );
    }

    #[test]
    fn find_focused_tests_returns_matching() {
        let names = vec![
            "suite::__FOCUS__test_a".to_string(),
            "suite::test_b".to_string(),
            "__FOCUS__suite::test_c".to_string(),
        ];
        let focused = find_focused_tests(&names);
        assert_eq!(focused.len(), 2);
    }

    #[test]
    fn find_focused_tests_returns_empty_when_none() {
        let names = vec!["suite::test_a".to_string(), "suite::test_b".to_string()];
        let focused = find_focused_tests(&names);
        assert!(focused.is_empty());
    }

    #[test]
    fn rejects_format_override_flag() {
        let result = validate_test_args(&["--format", "terse"]);
        assert!(matches!(
            result,
            Err(CliError::UnsupportedLibtestArg { arg }) if arg == "--format"
        ));
    }

    #[test]
    fn rejects_inline_format_override_flag() {
        let result = validate_test_args(&["--format=terse"]);
        assert!(matches!(
            result,
            Err(CliError::UnsupportedLibtestArg { arg }) if arg == "--format=terse"
        ));
    }

    #[test]
    fn find_focused_tests_with_tags_and_focus() {
        let names = vec![
            "__FOCUS____TAG_slow__suite::test_a".to_string(),
            "__TAG_slow__suite::test_b".to_string(),
        ];
        let focused = find_focused_tests(&names);
        assert_eq!(focused.len(), 1);
        assert_eq!(focused[0], "__FOCUS____TAG_slow__suite::test_a");
    }

    #[test]
    fn find_focused_tests_empty_input() {
        let names: Vec<String> = vec![];
        let focused = find_focused_tests(&names);
        assert!(focused.is_empty());
    }

    #[test]
    fn default_args_include_show_output() {
        let args = build_cargo_test_args(&[], &[]);
        assert!(args.iter().any(|a| a == "--show-output"));
    }
}

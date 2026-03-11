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
            vec!["test", "--all-features", "--", "--format", "pretty"]
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
                "--ignored",
            ]
        );
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
}

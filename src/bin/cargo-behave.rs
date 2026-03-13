//! `cargo-behave` binary entry point.
//!
//! Runs `cargo test` and renders results in tree, JSON, or `JUnit` form.
#![allow(clippy::print_stdout, clippy::print_stderr)]

use std::io::IsTerminal;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

use behave::cli::config::load_config;
use behave::cli::context::{resolve_project_context, SelectedPackage};
use behave::cli::error::CliError;
use behave::cli::history::{hash_source, load_history, save_history, update_and_detect, FlakyTest};
use behave::cli::output::{render_json, render_junit, Report, Summary};
use behave::cli::parser::{parse_test_output, reclassify_skipped, TestResult};
use behave::cli::render::{render_summary, render_tree};
use behave::cli::runner::{find_focused_tests, list_tests, run_cargo_test};
use behave::cli::tree::build_tree;
use behave::cli::watch::watch_loop;

/// Supported output formats for `cargo-behave`.
#[derive(Copy, Clone, Debug, Eq, PartialEq, ValueEnum)]
enum OutputFormat {
    /// Render the human-friendly tree output.
    Tree,
    /// Render a JSON report to stdout.
    Json,
    /// Render a `JUnit` XML report to stdout.
    Junit,
}

/// A BDD test runner for Rust.
#[derive(Parser, Debug)]
#[command(name = "cargo-behave", version, about)]
#[allow(clippy::struct_excessive_bools)]
struct Args {
    /// Subcommand name (always "behave" when invoked as `cargo behave`).
    #[arg(hide = true, default_value = "behave")]
    _subcmd: String,

    /// Disable colored output.
    #[arg(long)]
    no_color: bool,

    /// Output format for the test report.
    #[arg(long, value_enum, default_value_t = OutputFormat::Tree)]
    output: OutputFormat,

    /// Only run tests with at least one of these tags.
    #[arg(long = "tag", num_args = 1)]
    tags: Vec<String>,

    /// Exclude tests with any of these tags.
    #[arg(long = "exclude-tag", num_args = 1)]
    exclude_tags: Vec<String>,

    /// Only run focused tests. If no tests are focused, runs all.
    #[arg(long, conflicts_with = "fail_on_focus")]
    focus: bool,

    /// Fail if any focused tests are found (CI guard).
    #[arg(long, conflicts_with = "focus")]
    fail_on_focus: bool,

    /// Re-run tests on file changes.
    #[arg(long, conflicts_with = "fail_on_focus")]
    watch: bool,

    /// Extra arguments passed to `cargo test`.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    extra: Vec<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    if args.watch {
        return run_watch(&args);
    }

    match run(&args) {
        Ok(has_failures) => {
            if has_failures {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run_watch(args: &Args) -> ExitCode {
    let cwd = match std::env::current_dir() {
        Ok(d) => d,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let result = watch_loop(&cwd, || {
        match run(args) {
            Ok(_) | Err(_) => {}
        }
        Ok(())
    });

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &Args) -> Result<bool, CliError> {
    let cwd = std::env::current_dir().map_err(|source| CliError::Io { source })?;
    let (cargo_args, test_args) = split_extra_args(&args.extra);

    // --fail-on-focus: list tests, check for focused, fail if any
    if args.fail_on_focus {
        let test_names = list_tests(&cargo_args)?;
        let focused = find_focused_tests(&test_names);
        if !focused.is_empty() {
            for name in &focused {
                eprintln!("  focus: {name}");
            }
            return Err(CliError::FocusedTestsFound {
                count: focused.len(),
            });
        }
    }

    // --focus: add filter for focused tests only
    let cargo_args = if args.focus {
        let test_names = list_tests(&cargo_args)?;
        let focused = find_focused_tests(&test_names);
        if focused.is_empty() {
            cargo_args
        } else {
            let mut new_args = cargo_args;
            new_args.push("__FOCUS__");
            new_args
        }
    } else {
        cargo_args
    };

    let cargo_output = run_cargo_test(&cargo_args, &test_args)?;
    let mut report = build_report(&cwd, &cargo_args, &cargo_output)?;

    let tag_filtering = !args.tags.is_empty() || !args.exclude_tags.is_empty();
    if tag_filtering {
        report.tests = filter_results_by_tags(&report.tests, &args.tags, &args.exclude_tags);
        report.summary = Summary::from_results(&report.tests);
        report.tree = build_tree(&report.tests);

        if report.tests.is_empty() && cargo_output.status.success() {
            eprintln!("no tests matched the specified tags");
        }
    }

    render_output(args, &report)?;

    if args.output == OutputFormat::Tree {
        print_flaky_tests(&report.flaky_tests);
    }

    print_stderr_if_failed(report.command_success, &report.stderr);

    Ok(command_failed(
        report.command_success,
        report.summary.failed,
    ))
}

fn build_report(
    cwd: &Path,
    cargo_args: &[&str],
    cargo_output: &std::process::Output,
) -> Result<Report, CliError> {
    let stdout = String::from_utf8_lossy(&cargo_output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&cargo_output.stderr).to_string();
    let tests = collect_results(&stdout, &stderr);

    let tree = build_tree(&tests);
    let flaky_tests = detect_flaky_tests(cwd, cargo_args, &tests)?;
    let summary = Summary::from_results(&tests);

    Ok(Report::new(cargo_output.status.success(), tests, summary)
        .with_tree(tree)
        .with_flaky_tests(flaky_tests)
        .with_stderr(stderr))
}

fn render_output(args: &Args, report: &Report) -> Result<(), CliError> {
    let mut out = std::io::stdout().lock();

    match args.output {
        OutputFormat::Tree => render_tree_report(&mut out, args.no_color, report)?,
        OutputFormat::Json => render_json(&mut out, report).map_err(io_error)?,
        OutputFormat::Junit => render_junit(&mut out, report).map_err(io_error)?,
    }

    Ok(())
}

fn render_tree_report(
    writer: &mut impl std::io::Write,
    no_color: bool,
    report: &Report,
) -> Result<(), CliError> {
    if !should_render_results(report.command_success, &report.tests) {
        return Ok(());
    }

    let use_color = !no_color && atty_stdout();
    render_tree(writer, &report.tree, use_color).map_err(io_error)?;
    render_summary(writer, &report.summary, use_color).map_err(io_error)
}

fn detect_flaky_tests(
    cwd: &Path,
    cargo_args: &[&str],
    results: &[TestResult],
) -> Result<Vec<FlakyTest>, CliError> {
    if results.is_empty() {
        return Ok(Vec::new());
    }

    let context = resolve_project_context(cwd, cargo_args)?;
    let config = load_config(&context.config_dir)?;
    let Some(ref flaky_config) = config.flaky_detection else {
        return Ok(Vec::new());
    };

    if !flaky_config.enabled {
        return Ok(Vec::new());
    }

    let history_path = resolve_history_path(&context.config_dir, &flaky_config.history_file);
    let mut history = load_history(&history_path)?;
    let source_hash = compute_project_source_hash(&context.selected_packages);
    let flaky_tests = update_and_detect(&mut history, results, flaky_config, &source_hash);

    save_history(&history_path, &history)?;
    Ok(flaky_tests)
}

fn resolve_history_path(config_dir: &Path, history_file: &str) -> PathBuf {
    let path = Path::new(history_file);

    if path.is_absolute() {
        return path.to_path_buf();
    }

    config_dir.join(path)
}

fn compute_project_source_hash(packages: &[SelectedPackage]) -> String {
    let mut tracked_files = Vec::new();

    for package in packages {
        collect_package_files(package, &mut tracked_files);
    }

    tracked_files.sort();
    tracked_files.dedup();

    let mut combined = Vec::new();

    for path in tracked_files {
        if let Ok(content) = std::fs::read(&path) {
            combined.extend_from_slice(path.to_string_lossy().as_bytes());
            combined.push(0);
            combined.extend_from_slice(&content);
        }
    }

    hash_source(&combined)
}

fn collect_package_files(package: &SelectedPackage, tracked_files: &mut Vec<PathBuf>) {
    tracked_files.push(package.manifest_path.clone());
    push_if_exists(&package.root_dir.join("build.rs"), tracked_files);
    collect_rust_files(&package.root_dir.join("src"), tracked_files);
    collect_rust_files(&package.root_dir.join("tests"), tracked_files);
    collect_rust_files(&package.root_dir.join("examples"), tracked_files);
    collect_rust_files(&package.root_dir.join("benches"), tracked_files);
}

fn push_if_exists(path: &Path, tracked_files: &mut Vec<PathBuf>) {
    if path.is_file() {
        tracked_files.push(path.to_path_buf());
    }
}

fn collect_rust_files(dir: &Path, tracked_files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if file_type.is_symlink() {
            continue;
        }

        if file_type.is_dir() {
            collect_rust_files(&path, tracked_files);
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
            tracked_files.push(path);
        }
    }
}

fn sort_results(results: &mut [TestResult]) {
    results.sort_by(|left, right| left.full_name.cmp(&right.full_name));
}

fn collect_results(stdout: &str, stderr: &str) -> Vec<TestResult> {
    let mut results = parse_test_output(stdout);
    results.extend(parse_test_output(stderr));
    sort_results(&mut results);
    results
        .dedup_by(|left, right| left.full_name == right.full_name && left.outcome == right.outcome);
    reclassify_skipped(&mut results, stdout);
    results
}

fn filter_results_by_tags(
    results: &[TestResult],
    include_tags: &[String],
    exclude_tags: &[String],
) -> Vec<TestResult> {
    results
        .iter()
        .filter(|result| {
            let name = &result.full_name;

            // Exclude applied first: test excluded if ANY exclude tag found
            for tag in exclude_tags {
                let marker = format!("__TAG_{tag}__");
                if name.contains(&marker) {
                    return false;
                }
            }

            // Include: test matches if ANY include tag found (union)
            if include_tags.is_empty() {
                return true;
            }

            include_tags.iter().any(|tag| {
                let marker = format!("__TAG_{tag}__");
                name.contains(&marker)
            })
        })
        .cloned()
        .collect()
}

const fn command_failed(command_success: bool, failed_tests: usize) -> bool {
    !command_success || failed_tests > 0
}

const fn should_render_results(command_success: bool, results: &[TestResult]) -> bool {
    command_success || !results.is_empty()
}

fn split_extra_args(extra: &[String]) -> (Vec<&str>, Vec<&str>) {
    let split_at = extra.iter().position(|arg| arg == "--");

    split_at.map_or_else(
        || (extra.iter().map(String::as_str).collect(), Vec::new()),
        |index| {
            (
                extra[..index].iter().map(String::as_str).collect(),
                extra[index + 1..].iter().map(String::as_str).collect(),
            )
        },
    )
}

fn print_flaky_tests(flaky_tests: &[FlakyTest]) {
    if flaky_tests.is_empty() {
        return;
    }

    eprintln!("\n\u{26a0} Flaky tests detected:");
    for flaky in flaky_tests {
        eprintln!(
            "  {} \u{2014} failed after {} consecutive passes (source unchanged)",
            flaky.name, flaky.consecutive_passes
        );
    }
}

fn print_stderr_if_failed(command_success: bool, stderr: &str) {
    if !command_success && !stderr.trim().is_empty() {
        eprint!("{stderr}");
    }
}

const fn io_error(source: std::io::Error) -> CliError {
    CliError::Io { source }
}

fn atty_stdout() -> bool {
    std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_project_dir(test_name: &str) -> PathBuf {
        let unique = format!(
            "behave-{test_name}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_or(0, |duration| duration.as_nanos())
        );
        std::env::temp_dir().join(unique)
    }

    fn selected_package(project_dir: &Path, package_name: &str) -> SelectedPackage {
        SelectedPackage::new(
            package_name.to_string(),
            project_dir.join(package_name).join("Cargo.toml"),
            project_dir.join(package_name),
        )
    }

    #[test]
    fn command_failed_when_cargo_status_fails_even_without_failed_tests() {
        assert!(command_failed(false, 0));
    }

    #[test]
    fn command_failed_when_tests_fail() {
        assert!(command_failed(true, 1));
    }

    #[test]
    fn command_succeeds_when_status_is_ok_and_no_tests_failed() {
        assert!(!command_failed(true, 0));
    }

    #[test]
    fn does_not_render_results_for_pre_test_failure() {
        let results: Vec<TestResult> = Vec::new();
        assert!(!should_render_results(false, &results));
    }

    #[test]
    fn renders_results_when_command_succeeds_without_tests() {
        let results: Vec<TestResult> = Vec::new();
        assert!(should_render_results(true, &results));
    }

    #[test]
    fn renders_results_when_failures_are_parsed() {
        let results = vec![TestResult::new(
            "suite::case".to_string(),
            behave::cli::parser::TestOutcome::Fail,
        )];
        assert!(should_render_results(false, &results));
    }

    #[test]
    fn split_extra_args_without_separator() {
        let extra = vec![
            "checkout".to_string(),
            "--package".to_string(),
            "demo".to_string(),
        ];
        let (cargo_args, test_args) = split_extra_args(&extra);
        assert_eq!(cargo_args, vec!["checkout", "--package", "demo"]);
        assert!(test_args.is_empty());
    }

    #[test]
    fn split_extra_args_with_separator() {
        let extra = vec![
            "checkout".to_string(),
            "--".to_string(),
            "--nocapture".to_string(),
            "--ignored".to_string(),
        ];
        let (cargo_args, test_args) = split_extra_args(&extra);
        assert_eq!(cargo_args, vec!["checkout"]);
        assert_eq!(test_args, vec!["--nocapture", "--ignored"]);
    }

    #[test]
    fn clap_parses_hyphenated_cargo_args() {
        let parsed = Args::try_parse_from([
            "cargo-behave",
            "behave",
            "--output",
            "json",
            "--manifest-path",
            "examples/cli-workspace/Cargo.toml",
            "--package",
            "cli-fixture-api",
        ]);

        assert!(parsed.is_ok());
        if let Ok(args) = parsed {
            assert_eq!(
                args.extra,
                vec![
                    "--manifest-path",
                    "examples/cli-workspace/Cargo.toml",
                    "--package",
                    "cli-fixture-api",
                ]
            );
        }
    }

    #[test]
    fn collects_results_from_stderr_when_stdout_is_empty() {
        let results = collect_results("", "test checkout::alpha_case ... ok\n");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].full_name, "checkout::alpha_case");
    }

    #[test]
    fn collect_results_deduplicates_identical_entries() {
        let results = collect_results(
            "test checkout::alpha_case ... ok\n",
            "test checkout::alpha_case ... ok\n",
        );
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn clap_parses_focus_flag() {
        let parsed = Args::try_parse_from(["cargo-behave", "behave", "--focus"]);
        assert!(parsed.is_ok());
        if let Ok(args) = parsed {
            assert!(args.focus);
            assert!(!args.fail_on_focus);
        }
    }

    #[test]
    fn clap_parses_fail_on_focus_flag() {
        let parsed = Args::try_parse_from(["cargo-behave", "behave", "--fail-on-focus"]);
        assert!(parsed.is_ok());
        if let Ok(args) = parsed {
            assert!(!args.focus);
            assert!(args.fail_on_focus);
        }
    }

    #[test]
    fn clap_rejects_focus_and_fail_on_focus_together() {
        let parsed = Args::try_parse_from(["cargo-behave", "behave", "--focus", "--fail-on-focus"]);
        assert!(parsed.is_err());
    }

    #[test]
    fn clap_parses_watch_flag() {
        let parsed = Args::try_parse_from(["cargo-behave", "behave", "--watch"]);
        assert!(parsed.is_ok());
        if let Ok(args) = parsed {
            assert!(args.watch);
        }
    }

    #[test]
    fn clap_rejects_watch_and_fail_on_focus_together() {
        let parsed = Args::try_parse_from(["cargo-behave", "behave", "--watch", "--fail-on-focus"]);
        assert!(parsed.is_err());
    }

    #[test]
    fn clap_parses_tag_args() {
        let parsed = Args::try_parse_from([
            "cargo-behave",
            "behave",
            "--tag",
            "slow",
            "--exclude-tag",
            "flaky",
        ]);
        assert!(parsed.is_ok());
        if let Ok(args) = parsed {
            assert_eq!(args.tags, vec!["slow"]);
            assert_eq!(args.exclude_tags, vec!["flaky"]);
        }
    }

    #[test]
    fn clap_parses_libtest_separator() {
        let parsed =
            Args::try_parse_from(["cargo-behave", "behave", "checkout", "--", "--nocapture"]);

        assert!(parsed.is_ok());
        if let Ok(args) = parsed {
            assert_eq!(args.extra, vec!["checkout", "--", "--nocapture"]);
        }
    }

    #[test]
    fn source_hash_includes_nested_rust_files() {
        let project_dir = temp_project_dir("source-hash");
        let package_dir = project_dir.join("pkg-a");
        let nested_dir = package_dir.join("src/matchers");

        assert!(std::fs::create_dir_all(&nested_dir).is_ok());
        assert!(std::fs::write(
            package_dir.join("Cargo.toml"),
            "[package]\nname=\"pkg-a\"\n"
        )
        .is_ok());
        assert!(std::fs::write(package_dir.join("src/lib.rs"), "pub fn top() {}\n").is_ok());
        assert!(std::fs::write(nested_dir.join("mod.rs"), "pub fn nested() {}\n").is_ok());

        let original = compute_project_source_hash(&[selected_package(&project_dir, "pkg-a")]);

        assert!(std::fs::write(
            nested_dir.join("mod.rs"),
            "pub fn nested() { let _x = 1; }\n",
        )
        .is_ok());

        let updated = compute_project_source_hash(&[selected_package(&project_dir, "pkg-a")]);

        assert!(std::fs::remove_dir_all(&project_dir).is_ok());
        assert_ne!(original, updated);
    }

    #[test]
    fn source_hash_tracks_selected_packages_only() {
        let project_dir = temp_project_dir("selected-packages");
        let package_a = project_dir.join("pkg-a");
        let package_b = project_dir.join("pkg-b");

        assert!(std::fs::create_dir_all(package_a.join("src")).is_ok());
        assert!(std::fs::create_dir_all(package_b.join("src")).is_ok());
        assert!(
            std::fs::write(package_a.join("Cargo.toml"), "[package]\nname=\"pkg-a\"\n").is_ok()
        );
        assert!(
            std::fs::write(package_b.join("Cargo.toml"), "[package]\nname=\"pkg-b\"\n").is_ok()
        );
        assert!(std::fs::write(package_a.join("src/lib.rs"), "pub fn alpha() {}\n").is_ok());
        assert!(std::fs::write(package_b.join("src/lib.rs"), "pub fn beta() {}\n").is_ok());

        let selected = vec![selected_package(&project_dir, "pkg-a")];
        let original = compute_project_source_hash(&selected);

        assert!(std::fs::write(
            package_b.join("src/lib.rs"),
            "pub fn beta() { let _ = 1; }\n"
        )
        .is_ok());

        let updated = compute_project_source_hash(&selected);

        assert!(std::fs::remove_dir_all(&project_dir).is_ok());
        assert_eq!(original, updated);
    }

    #[test]
    fn source_hash_includes_manifest_changes() {
        let project_dir = temp_project_dir("manifest-hash");
        let package_dir = project_dir.join("pkg-a");

        assert!(std::fs::create_dir_all(package_dir.join("src")).is_ok());
        assert!(std::fs::write(
            package_dir.join("Cargo.toml"),
            "[package]\nname=\"pkg-a\"\n"
        )
        .is_ok());
        assert!(std::fs::write(package_dir.join("src/lib.rs"), "pub fn top() {}\n").is_ok());

        let original = compute_project_source_hash(&[selected_package(&project_dir, "pkg-a")]);

        assert!(std::fs::write(
            package_dir.join("Cargo.toml"),
            "[package]\nname=\"pkg-a\"\nversion=\"0.1.0\"\n",
        )
        .is_ok());

        let updated = compute_project_source_hash(&[selected_package(&project_dir, "pkg-a")]);

        assert!(std::fs::remove_dir_all(&project_dir).is_ok());
        assert_ne!(original, updated);
    }

    #[cfg(unix)]
    #[test]
    fn source_hash_skips_symlinked_directories() {
        use std::os::unix::fs::symlink;

        let project_dir = temp_project_dir("symlink-dir");
        let package_dir = project_dir.join("pkg-a");
        let src_dir = package_dir.join("src");
        let external_dir = package_dir.join("external");

        assert!(std::fs::create_dir_all(&src_dir).is_ok());
        assert!(std::fs::create_dir_all(&external_dir).is_ok());
        assert!(std::fs::write(
            package_dir.join("Cargo.toml"),
            "[package]\nname=\"pkg-a\"\n"
        )
        .is_ok());
        assert!(std::fs::write(src_dir.join("lib.rs"), "pub fn top() {}\n").is_ok());
        assert!(std::fs::write(external_dir.join("outside.rs"), "pub fn outside() {}\n").is_ok());
        assert!(symlink(&external_dir, src_dir.join("linked")).is_ok());

        let mut tracked_files = Vec::new();
        collect_package_files(&selected_package(&project_dir, "pkg-a"), &mut tracked_files);
        tracked_files.sort();

        assert_eq!(
            tracked_files,
            vec![package_dir.join("Cargo.toml"), src_dir.join("lib.rs")]
        );
        assert!(std::fs::remove_dir_all(&project_dir).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn source_hash_skips_symlinked_files() {
        use std::os::unix::fs::symlink;

        let project_dir = temp_project_dir("symlink-file");
        let package_dir = project_dir.join("pkg-a");
        let src_dir = package_dir.join("src");
        let external_dir = package_dir.join("external");

        assert!(std::fs::create_dir_all(&src_dir).is_ok());
        assert!(std::fs::create_dir_all(&external_dir).is_ok());
        assert!(std::fs::write(
            package_dir.join("Cargo.toml"),
            "[package]\nname=\"pkg-a\"\n"
        )
        .is_ok());
        assert!(std::fs::write(src_dir.join("lib.rs"), "pub fn top() {}\n").is_ok());
        assert!(std::fs::write(external_dir.join("outside.rs"), "pub fn outside() {}\n").is_ok());
        assert!(symlink(external_dir.join("outside.rs"), src_dir.join("linked.rs")).is_ok());

        let mut tracked_files = Vec::new();
        collect_package_files(&selected_package(&project_dir, "pkg-a"), &mut tracked_files);
        tracked_files.sort();

        assert_eq!(
            tracked_files,
            vec![package_dir.join("Cargo.toml"), src_dir.join("lib.rs")]
        );
        assert!(std::fs::remove_dir_all(&project_dir).is_ok());
    }

    #[test]
    fn filter_by_include_tag() {
        let results = vec![
            TestResult::new(
                "__TAG_slow__test_a".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
            TestResult::new("test_b".to_string(), behave::cli::parser::TestOutcome::Pass),
        ];
        let filtered = filter_results_by_tags(&results, &["slow".to_string()], &[]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].full_name, "__TAG_slow__test_a");
    }

    #[test]
    fn filter_by_exclude_tag() {
        let results = vec![
            TestResult::new(
                "__TAG_slow__test_a".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
            TestResult::new("test_b".to_string(), behave::cli::parser::TestOutcome::Pass),
        ];
        let filtered = filter_results_by_tags(&results, &[], &["slow".to_string()]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].full_name, "test_b");
    }

    #[test]
    fn filter_exclude_applied_before_include() {
        let results = vec![
            TestResult::new(
                "__TAG_slow____TAG_integration__test".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
            TestResult::new(
                "__TAG_integration__other".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
        ];
        let filtered = filter_results_by_tags(
            &results,
            &["integration".to_string()],
            &["slow".to_string()],
        );
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].full_name, "__TAG_integration__other");
    }

    #[test]
    fn filter_no_tags_returns_all() {
        let results = vec![
            TestResult::new("test_a".to_string(), behave::cli::parser::TestOutcome::Pass),
            TestResult::new("test_b".to_string(), behave::cli::parser::TestOutcome::Pass),
        ];
        let filtered = filter_results_by_tags(&results, &[], &[]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn sorts_results_by_full_name() {
        let mut results = vec![
            TestResult::new(
                "suite::zeta".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
            TestResult::new(
                "suite::alpha".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
        ];

        sort_results(&mut results);

        assert_eq!(results[0].full_name, "suite::alpha");
        assert_eq!(results[1].full_name, "suite::zeta");
    }

    #[test]
    fn resolves_relative_history_paths_from_config_dir() {
        let path = resolve_history_path(Path::new("/workspace/pkg-a"), ".behave/history.json");
        assert_eq!(path, PathBuf::from("/workspace/pkg-a/.behave/history.json"));
    }

    #[test]
    fn keeps_absolute_history_paths() {
        let path = resolve_history_path(Path::new("/workspace/pkg-a"), "/tmp/history.json");
        assert_eq!(path, PathBuf::from("/tmp/history.json"));
    }

    #[test]
    fn filter_multiple_include_tags_union() {
        let results = vec![
            TestResult::new(
                "__TAG_slow__test_a".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
            TestResult::new(
                "__TAG_fast__test_b".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
            TestResult::new("test_c".to_string(), behave::cli::parser::TestOutcome::Pass),
        ];
        let filtered =
            filter_results_by_tags(&results, &["slow".to_string(), "fast".to_string()], &[]);
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn filter_include_no_match_returns_empty() {
        let results = vec![TestResult::new(
            "test_a".to_string(),
            behave::cli::parser::TestOutcome::Pass,
        )];
        let filtered = filter_results_by_tags(&results, &["nonexistent".to_string()], &[]);
        assert!(filtered.is_empty());
    }

    #[test]
    fn filter_multiple_exclude_tags() {
        let results = vec![
            TestResult::new(
                "__TAG_slow__test_a".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
            TestResult::new(
                "__TAG_flaky__test_b".to_string(),
                behave::cli::parser::TestOutcome::Pass,
            ),
            TestResult::new("test_c".to_string(), behave::cli::parser::TestOutcome::Pass),
        ];
        let filtered =
            filter_results_by_tags(&results, &[], &["slow".to_string(), "flaky".to_string()]);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].full_name, "test_c");
    }

    #[test]
    fn filter_tag_in_nested_module_path() {
        let results = vec![TestResult::new(
            "suite::__TAG_unit__inner::test_a".to_string(),
            behave::cli::parser::TestOutcome::Pass,
        )];
        let filtered = filter_results_by_tags(&results, &["unit".to_string()], &[]);
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn clap_parses_multiple_tag_args() {
        let parsed = Args::try_parse_from([
            "cargo-behave",
            "behave",
            "--tag",
            "slow",
            "--tag",
            "integration",
            "--exclude-tag",
            "flaky",
            "--exclude-tag",
            "unstable",
        ]);
        assert!(parsed.is_ok());
        if let Ok(args) = parsed {
            assert_eq!(args.tags, vec!["slow", "integration"]);
            assert_eq!(args.exclude_tags, vec!["flaky", "unstable"]);
        }
    }

    #[test]
    fn clap_watch_compatible_with_focus() {
        let parsed = Args::try_parse_from(["cargo-behave", "behave", "--watch", "--focus"]);
        assert!(parsed.is_ok());
    }

    #[test]
    fn clap_watch_compatible_with_tags() {
        let parsed = Args::try_parse_from(["cargo-behave", "behave", "--watch", "--tag", "slow"]);
        assert!(parsed.is_ok());
    }
}

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Every PR with user-facing changes must add an entry under `[Unreleased]`.**

## [Unreleased]

### Added

- `behave!` proc macro for writing BDD-style test suites with zero-keyword DSL
- `expect!` macro for expressive assertions returning `Result<(), MatchError>`
- `expect_panic!` and `expect_no_panic!` macros for panic assertions
- `Expectation<T>` wrapper with `.negate()` for negated matching
- `BehaveMatch<T>` trait for custom matchers
- Built-in matchers: equality, boolean, ordering, option, result, collections, strings, float
- Setup blocks with automatic inheritance through nesting
- `pending` keyword for ignored/pending tests
- `focus` keyword for scenario markers in generated names and CLI output
- Optional `cargo-behave` CLI behind `cli` feature flag
- Single crate with internal `behave-macros` proc-macro subcrate
- Flaky test detection via `behave.toml` config and test history tracking
- `behave.toml` configuration file support for CLI settings
- `cargo-behave --output json` and `cargo-behave --output junit` machine-readable reports
- Workspace-aware flaky detection for `cargo-behave` with `--package`, `--workspace`, and `--manifest-path`
- A committed CLI workspace fixture plus CI coverage for real JSON and `JUnit` runs
- `teardown` blocks with panic-safe cleanup (sync) and error-safe cleanup (async)
- `tokio;` group declaration for async test generation (behind `tokio` feature)
- "Deliberate Omissions" section in ARCHITECTURE.md
- Collection matchers now work on `&[T]` slices in addition to `Vec<T>`
- `to_satisfy(predicate, description)` predicate matcher on `Expectation<T>`
- Parser rejects duplicate `setup`/`teardown` blocks and enforces DSL ordering

### Changed

- Public docs now describe `behave!` as compiling to ordinary `#[test]` functions with no custom test runtime, instead of claiming literal zero runtime overhead
- `cargo-behave` tree output is now sorted deterministically by test name
- `cargo-behave` now forces parseable libtest `pretty` output for report generation and reserves the libtest `--format` flag
- JUnit output now strips internal `__FOCUS__` / `__PENDING__` prefixes from displayed test names

<!-- [Unreleased]: https://github.com/stateruntime/behave/compare/v0.1.0...HEAD -->

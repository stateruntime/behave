# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Every PR with user-facing changes must add an entry under `[Unreleased]`.**

## [Unreleased]

## [0.6.0] - 2026-03-12

### Added

- **Test matrix (Cartesian product)** — `matrix [a, b] x [c, d] |p1, p2| { body }` generates tests for all combinations
  - Supports 2+ dimensions separated by `x`
  - Generates `case_I_J` (or `case_I_J_K`, etc.) function names from dimension indices
  - Inherits setup, teardown, tokio, timeout, and focus from parent context
  - Compatible with `xfail` for expected-failure matrix tests
- **Named test cases in `each`** — optional string label as first tuple element becomes the test function name
  - `each [("ok", 200, true), ("not_found", 404, false)] |name, code, ok| { ... }` generates `ok` and `not_found` instead of `case_0` and `case_1`
  - Labels are slugified to valid Rust identifiers; Rust keywords use raw identifiers (`r#type`)
  - Falls back to `case_N` when no label is provided
- **`xfail` keyword** — mark a test as expected-to-fail
  - Test passes when the body returns `Err`; fails loudly if the body unexpectedly passes
  - Works on individual tests, `each` blocks, and `matrix` blocks
  - Catches `Result::Err` (from `expect!` / `?`); panics still propagate as real failures
  - Cannot be combined with `pending` (compile error)
  - Cannot be applied to groups (compile error)

## [0.5.0] - 2026-03-11

### Added

- **Soft assertions** — collect multiple failures in a single test and report them together at the end
  - `SoftErrors::new()` creates a collector, `.check()` records results, `.finish()` returns all failures
  - `SoftMatchError` error type with numbered failure output
  - Gated on `std` feature, available via `behave::prelude::*`
- **Test timeout** — `timeout <ms>;` DSL keyword to prevent hanging tests
  - Sync tests spawn a thread with `recv_timeout` for deadline enforcement
  - Async tests use `tokio::time::timeout` (requires `tokio` feature)
  - Timeout inherits through nesting (inner overrides outer)
  - Teardown still runs inside the spawned thread (sync) or after timeout wrapper (async)

## [0.4.4] - 2026-03-11

### Fixed

- Fix macros crate exceeding crates.io 10MB upload limit by excluding `target/` directory
- Remove accidentally tracked `macros/target/` from git

## [0.4.3] - 2026-03-11

### Changed

- Enable `behave-macros` publishing to crates.io (previously had `publish = false`)
- Add `exclude` patterns to both crate manifests to reduce package size
- Release workflow now publishes `behave-macros` before the main crate

## [0.4.1]

### Fixed

- Fix clippy `use_self` lint in `TreeNode` struct definition

## [0.4.0]

### Added

- Matcher combinators for composing multiple matchers
  - `all_of(matchers)` — passes when all inner matchers pass (empty = vacuous truth)
  - `any_of(matchers)` — passes when at least one inner matcher passes (empty = fail)
  - `not_matching(matcher)` — inverts a single matcher inside a composition
  - Combinators implement `BehaveMatch<T>` and nest recursively
  - Multi-line failure descriptions with bullet lists and indented sub-matchers
- `BehaveMatch<T>` impl for `Box<dyn BehaveMatch<T>>` — enables passing boxed matchers to `to_match()`
- `HashMap` and `BTreeMap` matchers (behind `std` feature)
  - `to_contain_key(k)` — map has key
  - `to_contain_value(v)` — map has value
  - `to_contain_entry(k, v)` — map has key-value pair
  - `to_be_empty()` — no entries
  - `to_not_be_empty()` — has entries
  - `to_have_length(n)` — exact entry count

## [0.3.0]

### Added

- `color` feature flag for enhanced assertion failure output
  - Single-line values: red for actual, green for expected
  - Multiline values: line-by-line diff with `+`/`-` markers via the `similar` crate
  - Respects `NO_COLOR` environment variable per <https://no-color.org/>
  - Structured diff format preserved even when `NO_COLOR` disables ANSI codes
- `regex` feature flag with two new string matchers
  - `to_match_regex(pattern)` — full-string match (auto-anchored with `^(?:...)$`)
  - `to_contain_regex(pattern)` — substring match (unanchored)
  - Invalid regex patterns produce `MatchError` instead of panicking

## [0.2.0]

### Added

- `each` blocks for parameterized/table-driven test generation
  - Multi-param tuple syntax: `each [(a, b, c), ...] |x, y, z| { ... }`
  - Single-param syntax: `each [1, 2, 3] |n| { ... }`
  - Each case generates a separate `case_N` test function in a module
  - Inherits `setup`, `teardown`, `tokio;`, and `focus` from parent context

## [0.1.0]

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

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

**Every PR with user-facing changes must add an entry under `[Unreleased]`.**

## [Unreleased]

## [0.8.0] - 2026-03-13

### Added

- **Float shape matchers** — `to_be_nan()`, `to_be_finite()`, `to_be_infinite()`, `to_be_positive()`, `to_be_negative()` for `f32` and `f64`
  - NaN fails both positive and negative, `-0.0` fails both, `INFINITY` is positive+infinite
- **String QoL matchers** — `to_be_empty()`, `to_not_be_empty()`, `to_have_char_count(n)` for `String` and `&str`
  - `to_have_char_count` counts Unicode scalar values, not bytes
- **Sequence matchers** — ordered collection assertions for `Vec<T>` and `&[T]`
  - `to_contain_exactly(&[T])` — exact ordered match
  - `to_contain_exactly_in_any_order(&[T])` — same elements, any order (handles duplicates)
  - `to_start_with_elements(&[T])` — prefix match
  - `to_end_with_elements(&[T])` — suffix match
  - `to_be_sorted()` — non-descending order
- **Set matchers** — `HashSet` and `BTreeSet` assertions *(requires `std` feature)*
  - `to_contain(&T)`, `to_be_empty()`, `to_not_be_empty()`, `to_have_length(n)`
  - `to_be_subset_of(&Set)`, `to_be_superset_of(&Set)`
- **Path matchers** — filesystem path assertions for `PathBuf` and `&Path` *(requires `std` feature)*
  - `to_exist()`, `to_be_a_file()`, `to_be_a_directory()`
  - `to_have_extension(ext)`, `to_have_file_name(name)`
- **JSON matchers** — `serde_json::Value` assertions *(requires `json` feature)*
  - `to_have_field(field)` — key exists in object
  - `to_have_field_value(field, value)` — key has specific value
  - `to_be_json_superset_of(expected)` — recursive partial match (like Jest's `toMatchObject`)
- **HTTP matchers** — status code and header assertions *(requires `http` feature)*
  - `to_be_success()` (2xx), `to_be_redirect()` (3xx), `to_be_client_error()` (4xx), `to_be_server_error()` (5xx)
  - `to_have_status_code(code)`, `to_have_header(name)`, `to_have_header_value(name, value)`
- **URL matchers** — `url::Url` assertions *(requires `url` feature)*
  - `to_have_scheme(s)`, `to_have_host(h)`, `to_have_path(p)`
  - `to_have_query_param(key)`, `to_have_query_param_value(key, value)`, `to_have_fragment(f)`
- **`expect_match!` macro** — pattern matching assertions with optional guard
  - `expect_match!(expr, Pattern)` and `expect_match!(expr, Pattern if guard)`
  - Available in prelude
- **`each_type` DSL keyword** — typed test generation across multiple types
  - `each_type [i32, f64, u8] { "test" { ... } }` generates a module per type with `type T = ConcreteType;`
  - Inherits setup, teardown, tokio, timeout, focus, and tags from parent context
- **New feature flags:** `http`, `url`, `json` for domain-specific matcher packs

## [0.7.0] - 2026-03-13

### Added

- **Tag-based test metadata** — `tag "name1", "name2"` keyword on groups, tests, `each`, and `matrix` blocks
  - Tags encode as `__TAG_xxx__` prefixes in generated module/function names
  - Tag inheritance is automatic through module path (no explicit propagation needed)
  - `cargo-behave --tag slow` runs only tests with the `slow` tag (union matching)
  - `cargo-behave --exclude-tag flaky` excludes tests with the `flaky` tag
  - Both flags can be combined; exclude is applied first, then include
  - Tags displayed as `[slow, integration]` in tree output
  - JUnit and JSON output strips tag prefixes from displayed names
- **Focus-only mode** — `cargo-behave --focus` runs only focused tests
  - Lists all tests via `cargo test -- --list`, filters for `__FOCUS__` marker
  - If no tests are focused, runs all tests
- **CI focus guard** — `cargo-behave --fail-on-focus` exits non-zero if any focused tests exist
  - Prints focused test names to stderr before failing
  - Mutually exclusive with `--focus`
- **Runtime conditional skip** — `skip_when!(condition, "reason")` macro
  - Prints `BEHAVE_SKIP: reason` sentinel and returns early when condition is true
  - `cargo-behave` detects sentinel in `--show-output` and reclassifies `Pass` → `Skipped`
  - `Skipped` outcome with `⊘` symbol in Cyan in tree output
  - JUnit maps `Skipped` to `<skipped message="skipped: reason" />`
  - Summary line shows skipped count
- **Watch mode** — `cargo-behave --watch` re-runs tests on file changes
  - Watches `src/` and `tests/` recursively for `.rs` file changes
  - Debounces rapid changes (200ms)
  - Clears terminal between runs
  - Compatible with `--tag`, `--exclude-tag`, `--focus`, `--output`
  - Incompatible with `--fail-on-focus`

## [0.6.2] - 2026-03-12

### Fixed

- Release workflow now skips publishing `behave-macros` when that exact version already exists on crates.io, while still publishing `behave`

## [0.6.1] - 2026-03-12

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

[Unreleased]: https://github.com/stateruntime/behave/compare/v0.8.0...HEAD
[0.8.0]: https://github.com/stateruntime/behave/compare/v0.7.0...v0.8.0
[0.7.0]: https://github.com/stateruntime/behave/compare/v0.6.2...v0.7.0
[0.6.2]: https://github.com/stateruntime/behave/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/stateruntime/behave/compare/v0.6.0...v0.6.1

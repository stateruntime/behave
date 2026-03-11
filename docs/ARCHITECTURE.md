# Architecture

High-level overview of the behave crate's design and structure.

## Overview

`behave` is a BDD testing framework for Rust. It provides a `behave!` macro for
writing readable test suites and an `expect!` macro for expressive assertions.
Test suites compile to standard `#[test]` functions with no custom test runtime.

## Crate Structure

The project is a single published crate with an internal proc-macro subcrate:

- **`behave`** - The user-facing library at the project root. Contains matchers,
  the `Expectation` type, declarative macros (`expect!`, `expect_panic!`,
  `expect_no_panic!`), and the optional CLI module.
- **`behave-macros`** (`macros/`) - Internal proc-macro subcrate, not published
  separately. Contains the `behave!` proc macro that parses the DSL and generates
  `#[test]` functions. Users never depend on this crate directly.

## Design Principles

1. **No custom test runtime.** The `behave!` macro expands to standard `#[test]`
   functions at compile time. The DSL does not install its own runner or executor.
2. **Dependencies point inward.** `behave` depends on `behave-macros`. Matchers depend
   only on `Expectation` and `MatchError`. The CLI module is optional.
3. **Parse, don't validate.** The proc macro parses the DSL into a typed AST before
   generating code. Invalid DSL produces `compile_error!`, not runtime failures.
4. **Modular by default.** The CLI is behind a feature flag. The core library has zero
   dependencies beyond `behave-macros`.

## Module Map

```
src/
  lib.rs              # Public API, re-exports, expect!/expect_panic!/expect_no_panic! macros
  error.rs            # MatchError - structured error from failed assertions
  expectation.rs      # Expectation<T> - wraps values for matcher chains
  custom.rs           # BehaveMatch<T> trait for user-defined matchers
  matchers/           # Built-in matcher implementations (one file per category)
  cli/                # [feature = "cli"] cargo-behave binary support
    config.rs         # BehaveConfig - behave.toml parsing
    context.rs        # Workspace/package resolution via cargo metadata
    error.rs          # CliError - CLI error types
    history.rs        # TestHistory - flaky test detection via history tracking
    output.rs         # JSON and JUnit report rendering
    parser.rs         # TestResult - cargo test output parser
    render.rs         # Tree rendering with colors
    runner.rs         # Spawns cargo test
    tree.rs           # TreeNode - builds tree from flat test names

macros/src/
  lib.rs              # #[proc_macro] behave entry point
  parse.rs            # DSL parser (syn-based, BehaveInput → BehaveNode AST)
  codegen.rs          # AST → #[test] fn TokenStream
  slug.rs             # Human label → Rust identifier conversion
```

## Data Flow

```
User writes:           behave! { "suite" { "test" { expect!(x).to_equal(1)?; } } }
                                    |
                        behave-macros parses DSL
                                    |
                        BehaveInput (AST)
                                    |
                        codegen generates:
                                    |
                        mod suite { #[test] fn test() -> Result<...> { ... } }
                                    |
                        cargo test runs standard #[test] functions
```

## Flaky Test Detection Flow

```
cargo behave
    |
    ├── resolve workspace/package context from cargo metadata
    ├── load behave.toml (config)
    ├── run cargo test, parse output
    ├── load package-aware history file
    ├── for each test result:
    │     pass  → increment consecutive_passes, update source_hash
    │     fail  → if consecutive_passes ≥ threshold AND source unchanged → FLAKY
    │     fail  → reset consecutive_passes
    ├── save updated history
    ├── render tree / JSON / JUnit output
    └── print flaky warnings in tree mode
```

## Key Design Decisions

1. **Separate proc-macro crate** - Rust requires proc macros in their own crate.
   `behave-macros` is an implementation detail; users only depend on `behave`.
2. **`Result<(), MatchError>` return type** - Matchers return `Result` so `?` works
   in test bodies. Generated test functions return `Result<(), Box<dyn Error>>`.
3. **`#[non_exhaustive]` on all public types** - Allows adding fields/variants in
   minor versions without breaking downstream.
4. **Manual `Display` + `Error` impls** - No derive macro dependencies for errors.
5. **`.not()` for negation** - Flips the expectation so any matcher checks the opposite condition.

## Deliberate Omissions

Decisions that were considered and intentionally rejected:

1. **Full attribute-style API** — dilutes DSL identity; the string-label tree is
   the core differentiator.
2. **Inference-based context injection** — breaks the simple expansion model
   where setup code is pasted into the test function.
3. **Heavy lifecycle machinery (before-all, after-all hooks)** — fights the
   "ordinary tests" promise; `setup` and `teardown` per-test is sufficient.
4. **Custom test runners** — keeps tests compatible with `cargo test`, IDEs,
   and CI without additional configuration.

Ideas that may be revisited in the future:

1. Suite-level shared setup (expensive one-time resources)
2. Context-returning hooks (setup returns values to tests)
3. Parameterized tests (data-driven test generation)
4. Snapshot testing (integrate with `insta` rather than reinvent)

## Dependencies

| Crate | Purpose |
|-------|---------|
| `syn` | Parsing the `behave!` DSL token stream |
| `quote` | Generating Rust code from AST |
| `proc-macro2` | Token manipulation in proc macros |
| `clap` | CLI argument parsing (optional, `cli` feature) |
| `crossterm` | Terminal colors (optional, `cli` feature) |
| `serde` | Serialization for config and history (optional, `cli` feature) |
| `serde_json` | JSON history file format and JSON report output (optional, `cli` feature) |
| `tokio` | Async test runtime (optional, `tokio` feature) |
| `toml` | TOML config file parsing (optional, `cli` feature) |

## Feature Flags

| Feature | Default | Purpose |
|---------|---------|---------|
| `std` | Yes | Standard library support |
| `cli` | No | Enables `cargo-behave` binary with flaky detection and report output |
| `tokio` | No | Re-exports `tokio` for `tokio;` async test generation |

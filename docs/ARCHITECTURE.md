# Architecture

High-level overview of the behave crate's design and structure.

## Overview

`behave` is a BDD testing framework for Rust. It provides a `behave!` macro for
writing readable test suites and an `expect!` macro for expressive assertions.
Test suites compile to standard `#[test]` functions with no custom test runtime.

## Crate Structure

The project is published as two crates:

- **`behave`** - The user-facing library at the project root. Contains matchers,
  the `Expectation` type, declarative macros (`expect!`, `expect_panic!`,
  `expect_no_panic!`), and the optional CLI module.
- **`behave-macros`** (`macros/`) - Proc-macro crate that implements the `behave!`
  DSL and code generation. It is published because `behave` depends on it, but it
  is still an implementation detail: users should treat it as internal and depend
  on `behave` instead.

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
  combinators.rs      # all_of, any_of, not_matching — matcher composition
  custom.rs           # BehaveMatch<T> trait for user-defined matchers
  matchers/           # Built-in matcher implementations (one file per category)
    hashmap.rs        # [feature = "std"] HashMap/BTreeMap matchers
    regex.rs          # [feature = "regex"] to_match_regex, to_contain_regex
  cli/                # [feature = "cli"] cargo-behave binary support
    config.rs         # BehaveConfig - behave.toml parsing
    context.rs        # Workspace/package resolution via cargo metadata
    error.rs          # CliError - CLI error types
    history.rs        # TestHistory - flaky test detection via history tracking
    output.rs         # JSON and JUnit report rendering
    parser.rs         # TestResult - cargo test output parser, skip_when! reclassification
    render.rs         # Tree rendering with colors and tag display
    runner.rs         # Spawns cargo test, list_tests, find_focused_tests
    tree.rs           # TreeNode - builds tree from flat test names, tag detection
    watch.rs          # File-watching loop for --watch mode (uses notify crate)

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
                        BehaveInput (AST: Group, Test, Pending, Each, Matrix nodes)
                                    |
                        codegen generates (with tag/focus/xfail prefixes):
                                    |
                        mod suite { #[test] fn test() -> Result<...> { ... } }
                        (each → mod label { fn case_0, ... })
                        (matrix → mod label { fn case_0_0, fn case_0_1, ... })
                        (tags → __TAG_name__ prefixes on module/function names)
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

Ideas that were evaluated and deliberately rejected (see [ROADMAP.md](ROADMAP.md)
for full rationale with competitor evidence):

1. **Suite-level shared setup** — rstest's `#[once]` never drops, breaks
   nextest. pytest session scopes break in parallel. Use `OnceLock` in user code.
2. **Lazy bindings (`let`-style)** — RSpec's most debated feature. Creates
   "mystery guests" where setup is invisible at the point of use.
3. **Shared examples/contexts** — widely considered an anti-pattern: ghost
   variables, exponential test growth, debugging nightmares.
4. **Snapshot testing (built-in)** — "engineers begin blindly updating."
   Document insta integration instead.
5. **Fixture injection (pytest-style)** — pytest maintainers acknowledge the
   indirection and coupling problems at scale.

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
| `regex` | Regex engine for pattern matching (optional, `regex` feature) |
| `similar` | Text diffing for colored output (optional, `color` feature) |
| `tokio` | Async test runtime (optional, `tokio` feature) |
| `toml` | TOML config file parsing (optional, `cli` feature) |

## Feature Flags

| Feature | Default | Purpose |
|---------|---------|---------|
| `std` | Yes | Standard library support |
| `cli` | No | Enables `cargo-behave` binary with flaky detection and report output |
| `color` | No | ANSI-colored diff output for assertion failures (requires `std`) |
| `regex` | No | `to_match_regex` and `to_contain_regex` string matchers |
| `tokio` | No | Re-exports `tokio` for `tokio;` async test generation |

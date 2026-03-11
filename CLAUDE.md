# CLAUDE.md

Instructions for AI agents working on this codebase.

## Build & Test Commands

- **All checks:** `just check` (requires just)
- **Build:** `cargo build`
- **Test all:** `cargo test --all-features`
- **Test single:** `cargo test test_name`
- **Test minimal:** `cargo test --no-default-features`
- **Lint:** `cargo clippy --all-features --all-targets -- -D warnings`
- **Format:** `cargo fmt --all`
- **Format check:** `cargo fmt --all -- --check`
- **Doc check:** `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`

## Versioning

- `VERSION` file is the single source of truth
- Must match `version` in `Cargo.toml` - CI enforces this
- Follow SemVer strictly
- Every PR with user-facing changes must update CHANGELOG.md under [Unreleased]

## Code Rules (Summary)

- Edition 2021, MSRV 1.75
- `unsafe` is **forbidden**
- **Banned:** `unwrap()`, `expect()`, `panic!()`, `todo!()`, `dbg!()`, `println!()`
- Clippy `pedantic` + `nursery` enforced
- All public items must have `///` doc comments with `# Examples`
- Manual `Display` + `Error` impls for error types, `#[non_exhaustive]` on public enums/structs
- Guard clauses over nested conditionals
- Max function body: 40 lines, max nesting: 3 levels, max params: 4
- Feature flags must be additive
- Parse, don't validate: use newtypes for domain concepts
- No global mutable state, no hidden side effects
- Prefer duplication over the wrong abstraction (Rule of Three)

## Project Structure

```
VERSION                          # Single source of truth for version
Cargo.toml                      # Main "behave" crate (no workspace)
deny.toml                       # Dependency audit config
justfile                        # Task runner recipes
rustfmt.toml                    # Formatter config
src/
  lib.rs                         # Public API surface, re-exports, macros
  error.rs                       # MatchError type
  expectation.rs                 # Expectation<T> wrapper
  custom.rs                      # BehaveMatch<T> trait
  matchers/
    mod.rs                       # Matcher module index
    equality.rs                  # to_equal, to_not_equal
    boolean.rs                   # to_be_true, to_be_false
    ordering.rs                  # to_be_greater_than, to_be_less_than, etc.
    option.rs                    # to_be_some, to_be_none, etc.
    result.rs                    # to_be_ok, to_be_err, etc.
    collections.rs               # to_contain, to_be_empty, to_have_length, etc.
    strings.rs                   # to_start_with, to_end_with, etc.
    float.rs                     # to_approximately_equal
  cli/                           # Behind "cli" feature flag
    mod.rs
    runner.rs                    # Spawns cargo test
    parser.rs                    # Parses test output
    tree.rs                      # Builds tree hierarchy
    render.rs                    # Colored tree output
    output.rs                    # Report, Summary, JSON/JUnit rendering
    error.rs                     # CliError type
    config.rs                    # BehaveConfig - behave.toml parsing
    context.rs                   # Project context resolution
    history.rs                   # TestHistory - flaky test detection
  bin/
    cargo-behave.rs              # CLI entry point (requires "cli" feature)
tests/
  smoke.rs                       # Integration tests
macros/                          # Internal proc-macro subcrate (NOT published)
  Cargo.toml
  src/
    lib.rs                       # #[proc_macro] behave
    parse.rs                     # DSL parser
    codegen.rs                   # Code generation
    slug.rs                      # Label slugification
docs/
  AGENT.md                       # Full code style guide (READ THIS)
  ARCHITECTURE.md                # Architecture overview
  RELEASE.md                     # Release process

Feature flags: std (default), cli (cargo-behave binary), tokio (async test generation).
```

The full style guide with all rules, examples, and rationale is in [docs/AGENT.md](docs/AGENT.md).

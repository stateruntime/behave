# CLAUDE.md

Instructions for AI agents working on this codebase.

## Build & Test Commands

- **All checks:** `just check` (requires just)
- **Build:** `cargo build`
- **Test all:** `cargo test --all-features`
- **Test single:** `cargo test test_name`
- **Test minimal:** `cargo test --no-default-features`
- **Lint:** `cargo clippy --all-features --all-targets -- -D warnings`
- **Format:** `cargo fmt`
- **Format check:** `cargo fmt --check`
- **Doc check:** `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`

## Versioning

- `VERSION` file is the single source of truth
- Must match `version` in `Cargo.toml` — CI enforces this
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
VERSION               # Single source of truth for version
Cargo.toml            # Crate manifest (version synced from VERSION)
deny.toml             # Dependency audit config
justfile              # Task runner recipes
rustfmt.toml          # Formatter config
src/
  lib.rs              # Public API surface, re-exports
  error.rs            # Crate error types
docs/
  AGENT.md            # Full code style guide (READ THIS)
  ARCHITECTURE.md     # Architecture overview
  RELEASE.md          # Release process
```

The full style guide with all rules, examples, and rationale is in [docs/AGENT.md](docs/AGENT.md).

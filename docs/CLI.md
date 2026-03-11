# CLI Guide

`cargo-behave` is an optional wrapper around `cargo test`. It keeps the normal
Rust test pipeline, but adds:

- hierarchical tree output
- stable sorted results
- workspace-aware flaky-test tracking
- machine-readable JSON and JUnit reports

## Install

```bash
cargo install behave --features cli
```

Then run:

```bash
cargo behave
```

## How It Works

The CLI does five things:

1. runs `cargo test --all-features`
2. forces libtest into parseable `pretty` output mode
3. parses lines like `test suite::case ... ok`
4. sorts parsed test names for deterministic output
5. renders the results as a tree, JSON, or JUnit XML

If flaky detection is enabled, it also resolves the active package set using
Cargo metadata, hashes the selected package inputs, compares outcomes against
history, and writes the updated history back.

## Output Formats

Tree output is the default:

```bash
cargo behave
```

Example shape:

```text
└─ checkout
   ├─ ✓ adds items
   ├─ ✓ [focus] recalculates tax
   └─ ○ [pending] applies coupon codes

2 passed, 1 ignored
```

JSON output is useful for scripts and editors:

```bash
cargo behave --output json
```

JUnit output is useful for CI systems that ingest XML reports:

```bash
cargo behave --output junit > behave-report.xml
```

Both machine-readable formats include:

- command success status
- parsed tests and summary counts
- flaky-test warnings in structured form
- captured stderr from the underlying `cargo test` run

JSON keeps raw libtest names in `tests[*].full_name`, including internal
`__FOCUS__` or `__PENDING__` prefixes when they exist. The `tree` view strips
those internal prefixes and exposes `focused` / `pending` booleans explicitly.
JUnit output also strips the internal prefixes so CI systems show readable test
names.

For a concrete workspace example, see
[`examples/cli-workspace/README.md`](../examples/cli-workspace/README.md). That
fixture is exercised in CI with real `cargo-behave` runs.

## Passing Arguments

Arguments before a literal `--` are forwarded to `cargo test`.

```bash
cargo behave checkout
cargo behave --package my_crate checkout
cargo behave --manifest-path crates/api/Cargo.toml
cargo behave --workspace --exclude experimental
```

Arguments after a literal `--` are forwarded to the libtest binary.

```bash
cargo behave -- --ignored
cargo behave checkout -- --nocapture
```

This means:

- use the left side for package selection, workspace flags, and test-name filters
- use the right side for libtest flags such as `--ignored` and `--nocapture`
- do not pass libtest `--format`; `cargo-behave` reserves it so reports stay parseable

## Workspace Awareness

When flaky detection is enabled, `cargo-behave` resolves the current invocation
through `cargo metadata`.

That means:

- `cargo behave` in a workspace member tracks that member by default
- `cargo behave --package my_crate` hashes the selected package, not the whole workspace
- `cargo behave --manifest-path path/to/Cargo.toml` resolves config and source hashing from that manifest
- relative `history_file` paths in `behave.toml` are resolved from the active config directory

The source hash includes the selected package's `Cargo.toml`, `build.rs`, and
Rust files under `src/`, `tests/`, `examples/`, and `benches/`.

## Disable Color

```bash
cargo behave --no-color
```

Color is enabled only for tree output when stdout is attached to a terminal.

## Flaky-Test Detection

Create `behave.toml`:

```toml
[flaky_detection]
enabled = true
history_file = ".behave/history.json"
consecutive_passes = 5
```

Then run:

```bash
cargo behave
```

If a test fails after enough repeated passes and the selected package hash is
unchanged, the CLI records it as flaky.

Add `.behave/` to `.gitignore`.

## Important Limits

- the CLI still runs tests with `--all-features`
- parsing still depends on the `pretty` libtest output shape produced by `cargo-behave`
- libtest `--format` is reserved by `cargo-behave`
- `focus` is a marker, not focus-only execution control
- flaky detection is still heuristic, not a full semantic dependency graph

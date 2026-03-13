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

## Tag Filtering

Use `--tag` to run only tests with matching tags, and `--exclude-tag` to skip them:

```bash
cargo behave --tag slow
cargo behave --exclude-tag flaky
cargo behave --tag integration --exclude-tag slow
```

Tags are set in the DSL with the `tag` keyword:

```rust,ignore
behave! {
    "database" tag "slow", "integration" {
        "creates a user" {
            expect!(true).to_be_true()?;
        }
    }
}
```

When both `--tag` and `--exclude-tag` are used, exclusion is applied first, then
inclusion. `--tag` uses union matching: a test matches if it has *any* of the
specified tags.

Tags are displayed as `[slow, integration]` in tree output and stripped from
JUnit/JSON names.

## Focus Mode and CI Guard

Run only focused tests:

```bash
cargo behave --focus
```

If no tests have a `focus` marker, all tests run. If any do, only focused tests
run.

Reject focused tests in CI:

```bash
cargo behave --fail-on-focus
```

This lists all tests, and if any contain a `__FOCUS__` marker, prints their
names to stderr and exits non-zero. Use this in CI to prevent focused tests
from being merged.

`--focus` and `--fail-on-focus` are mutually exclusive.

## Watch Mode

Re-run tests automatically when `.rs` files change:

```bash
cargo behave --watch
```

Watch mode:

- watches `src/` and `tests/` recursively for `.rs` file changes
- debounces rapid changes (200ms)
- clears the terminal between runs
- compatible with `--tag`, `--exclude-tag`, `--focus`, and `--output`
- incompatible with `--fail-on-focus`

## Runtime Conditional Skip

Use `skip_when!` inside a test body to skip at runtime:

```rust,ignore
behave! {
    "redis tests" {
        "requires redis" {
            skip_when!(std::env::var("REDIS_URL").is_err(), "REDIS_URL not set");
            expect!(true).to_be_true()?;
        }
    }
}
```

When `cargo-behave` detects the skip sentinel, it reclassifies the test as
`Skipped` with the `⊘` symbol in tree output. Without the CLI, the test simply
passes silently.

## Important Limits

- the CLI still runs tests with `--all-features`
- parsing still depends on the `pretty` libtest output shape produced by `cargo-behave`
- libtest `--format` is reserved by `cargo-behave`
- flaky detection is still heuristic, not a full semantic dependency graph

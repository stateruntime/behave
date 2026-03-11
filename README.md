# behave

[![Crates.io](https://img.shields.io/crates/v/behave.svg)](https://crates.io/crates/behave)
[![Documentation](https://docs.rs/behave/badge.svg)](https://docs.rs/behave)
[![CI](https://github.com/stateruntime/behave/actions/workflows/ci.yml/badge.svg)](https://github.com/stateruntime/behave/actions/workflows/ci.yml)
[![License](https://img.shields.io/crates/l/behave.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/MSRV-1.75-blue.svg)](https://blog.rust-lang.org/)

`behave` is a behavior-driven testing library for Rust. It gives you a `behave!`
macro for nested, readable test suites and an `expect!` API for expressive
assertions, while still compiling down to ordinary `#[test]` functions.

## What It Is

Use `behave` when you want test code that reads like scenarios instead of a flat
list of unrelated unit tests:

- nested groups instead of large test modules
- `setup` blocks that flow into child scenarios
- built-in matchers for equality, strings, collections, options, results, and floats
- `pending` and `focus` markers for test workflow
- optional `cargo-behave` CLI for tree, JSON, and JUnit output plus flaky-test detection

## How It Works

`behave!` is a proc macro. At compile time it turns your scenario tree into
standard Rust test functions, so `cargo test` still runs the suite and there is
no custom runtime to keep alive.

## Start Fast

Add the crate as a dev-dependency:

```bash
cargo add behave --dev
```

Create `tests/behave_smoke.rs`:

```rust
use behave::prelude::*;

behave! {
    "checkout totals" {
        setup {
            let prices = [120, 80, 40];
            let subtotal: i32 = prices.iter().sum();
        }

        "adds line items" {
            expect!(subtotal).to_equal(240)?;
        }

        "renders a receipt line" {
            let receipt = format!("subtotal={subtotal}");
            expect!(receipt).to_contain_substr("240")?;
        }
    }

    pending "applies coupon codes" {}
}
```

Run it:

```bash
cargo test
```

That is the whole onboarding path. The generated tests are normal `#[test]`
items, so you can keep using the usual Rust tooling around them.

## Copy-Paste Commands

Create a new project and try `behave` in one go:

```bash
cargo new behave-demo
cd behave-demo
cargo add behave --dev
mkdir -p tests
```

Then put the Quick Start example above into `tests/behave_smoke.rs` and run
`cargo test`.

Install the optional CLI:

```bash
cargo install behave --features cli
```

Run the suite with tree output:

```bash
cargo behave
```

Emit a machine-readable report:

```bash
cargo behave --output json
cargo behave --output junit
```

## Features

| Feature | Default | Description |
|---------|---------|-------------|
| `std` | Yes | Standard library support |
| `cli` | No | Enables `cargo-behave` and flaky-test utilities |
| `tokio` | No | Enables `tokio;` async test generation |

## Matchers

| Category | Matchers |
|----------|----------|
| Equality | `to_equal`, `to_not_equal` |
| Boolean | `to_be_true`, `to_be_false` |
| Ordering | `to_be_greater_than`, `to_be_less_than`, `to_be_at_least`, `to_be_at_most` |
| Option | `to_be_some`, `to_be_none`, `to_be_some_with` |
| Result | `to_be_ok`, `to_be_err`, `to_be_ok_with`, `to_be_err_with` |
| Collections | `to_contain`, `to_be_empty`, `to_not_be_empty`, `to_have_length`, `to_contain_all_of` |
| Strings | `to_start_with`, `to_end_with`, `to_contain_substr`, `to_have_str_length` |
| Float | `to_approximately_equal`, `to_approximately_equal_within` |
| Panic | `expect_panic!`, `expect_no_panic!` |
| Predicate | `to_satisfy` |
| Custom | `to_match` with `BehaveMatch` |

All matchers respect `.not()`.

The full explanation for every matcher, including what it checks, why you would
choose it, and a working example for each method, is in
[docs/MATCHERS.md](docs/MATCHERS.md).

## Real Examples

- [`examples/quickstart.rs`](examples/quickstart.rs) shows the recommended first suite.
- [`examples/custom_matcher.rs`](examples/custom_matcher.rs) shows a reusable matcher type.
- [`examples/cli-workspace/README.md`](examples/cli-workspace/README.md) shows the workspace fixture used to verify CLI JSON and JUnit output in CI.
- [`tests/smoke.rs`](tests/smoke.rs) exercises the full DSL and matcher surface.

## What You Can And Cannot Do

You can:

- nest groups freely
- share bindings from a parent `setup` into child scenarios
- shadow a setup binding with a later `let` in a child `setup` or scenario body
- use `teardown` blocks for panic-safe cleanup that runs even when tests fail
- declare `tokio;` in a group to generate `#[tokio::test]` async tests (requires `tokio` feature)
- use `cargo test` normally because generated tests are ordinary `#[test]` functions
- use `cargo behave` for tree output, filters, and libtest flags
- use `cargo behave --output json` or `cargo behave --output junit` for CI-friendly reports
- use `cargo behave --manifest-path path/to/Cargo.toml` or `--package name` in workspaces

Current limitations:

- one `setup` block per group, one `teardown` block per group
- DSL order within a group: `tokio;` → `setup {}` → `teardown {}` → children
- `pending` blocks must be empty
- `focus` is a marker shown in generated names and CLI output; it does not automatically skip non-focused tests
- async teardown is error-safe but not panic-safe (no `catch_unwind` across `.await` points)

## Why Rely On It

The current trust signals are intentionally concrete:

- `behave!` compiles to ordinary `#[test]` functions
- runnable examples live in `examples/` and are exercised in tests
- public docs, doctests, Clippy, and rustdoc warnings are checked together
- `unsafe` is forbidden by lint configuration
- limitations are documented explicitly instead of left implicit
- security reporting is documented in [SECURITY.md](SECURITY.md)

For the fuller trust and maintenance picture, see [docs/RELIABILITY.md](docs/RELIABILITY.md).

## Flaky Test Detection

Create `behave.toml` in your project root:

```toml
[flaky_detection]
enabled = true
history_file = ".behave/history.json"
consecutive_passes = 5
```

When enabled, `cargo behave` records past outcomes and warns when a test fails
after many consecutive passes without source changes in the selected package set.

Add `.behave/` to `.gitignore`.

## Documentation

- [User Guide](docs/USER_GUIDE.md)
- [Matcher Reference](docs/MATCHERS.md)
- [CLI Guide](docs/CLI.md)
- [Reliability](docs/RELIABILITY.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Contributing](CONTRIBUTING.md)
- [API docs on docs.rs](https://docs.rs/behave)

## Security

See [SECURITY.md](SECURITY.md) for the reporting process.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

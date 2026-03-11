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
- `each` blocks for parameterized/table-driven tests
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

## Parameterized Tests

Use `each` to generate one test per case. Each case becomes its own `#[test]`
function, so failures tell you exactly which input broke:

```rust
use behave::prelude::*;

behave! {
    "addition" {
        each [
            (2, 2, 4),
            (0, 0, 0),
            (-1, 1, 0),
        ] |a, b, expected| {
            expect!(a + b).to_equal(expected)?;
        }
    }
}
```

This generates `addition::case_0`, `addition::case_1`, and `addition::case_2`.

Single-parameter cases work too:

```rust
use behave::prelude::*;

behave! {
    "Fibonacci numbers are positive" {
        each [1, 1, 2, 3, 5, 8, 13] |n| {
            expect!(n).to_be_greater_than(0)?;
        }
    }
}
```

`each` inherits `setup`, `teardown`, and `tokio;` from the parent group:

```rust
use behave::prelude::*;

behave! {
    "tax calculation" {
        setup {
            let tax_rate = 0.08_f64;
        }

        "computes total with tax" {
            each [
                (100.0_f64, 108.0_f64),
                (50.0_f64, 54.0_f64),
                (0.0_f64, 0.0_f64),
            ] |price, expected| {
                let total = price.mul_add(tax_rate, price);
                expect!(total).to_approximately_equal(expected)?;
            }
        }
    }
}
```

See [`examples/parameterized.rs`](examples/parameterized.rs) for the full
working example.

## Setup Inheritance

`setup` bindings flow from parent groups into child scenarios and child
`setup` blocks. This avoids duplicating shared state:

```rust
use behave::prelude::*;

behave! {
    "order pricing" {
        setup {
            let items = vec![1200, 800, 350];
            let total: i64 = items.iter().sum();
        }

        "subtotal sums line items" {
            expect!(total).to_equal(2350)?;
        }

        "with 10% discount" {
            setup {
                let discounted = total - (total * 10 / 100);
            }

            "applies percentage" {
                expect!(discounted).to_equal(2115)?;
            }

            "with shipping" {
                setup {
                    let final_total = discounted + 500;
                }

                "adds flat fee" {
                    expect!(final_total).to_equal(2615)?;
                }
            }
        }
    }
}
```

See [`examples/setup_inheritance.rs`](examples/setup_inheritance.rs) for
a fuller version with helper functions and shadowing.

## Teardown

`teardown` blocks run after every test in the group, even if the test panics.
Use them for cleanup that must not be skipped:

```rust
use behave::prelude::*;

behave! {
    "database tests" {
        setup {
            let pool = vec!["conn_1"];
        }

        teardown {
            // Runs even if the test panics.
            drop(pool);
        }

        "connection is available" {
            expect!(pool).to_have_length(1)?;
        }
    }
}
```

Inner teardowns run before outer teardowns (like destructors). See
[`examples/teardown.rs`](examples/teardown.rs) for nested teardown patterns.

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

| Example | What it shows |
|---------|---------------|
| [`examples/quickstart.rs`](examples/quickstart.rs) | Recommended first suite with `setup`, matchers, and `pending` |
| [`examples/parameterized.rs`](examples/parameterized.rs) | `each` blocks with multi-param tuples, single params, and inherited setup |
| [`examples/setup_inheritance.rs`](examples/setup_inheritance.rs) | Three levels of nested `setup` with a realistic pricing domain |
| [`examples/teardown.rs`](examples/teardown.rs) | Panic-safe cleanup, nested teardowns, and resource management |
| [`examples/custom_matcher.rs`](examples/custom_matcher.rs) | Reusable `BehaveMatch<T>` matcher type with negation |
| [`tests/smoke.rs`](tests/smoke.rs) | Full DSL and matcher surface coverage |

## What You Can And Cannot Do

You can:

- nest groups freely
- share bindings from a parent `setup` into child scenarios
- shadow a setup binding with a later `let` in a child `setup` or scenario body
- use `each` blocks for parameterized/table-driven test generation
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

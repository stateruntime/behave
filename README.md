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
- `matrix` blocks for Cartesian product test generation
- `xfail` for expected-failure tests
- `tag` metadata for grouping and filtering tests
- `skip_when!` for runtime conditional skipping
- built-in matchers for equality, strings, collections, options, results, and floats
- `pending` and `focus` markers for test workflow
- optional `cargo-behave` CLI with tree/JSON/JUnit output, watch mode, tag filtering, and flaky-test detection

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

```rust,ignore
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

```rust,ignore
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

```rust,ignore
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

`teardown` blocks run after every test in the group, even if the test panics (sync tests) or returns an error (async tests). Use them for cleanup:

```rust,ignore
use behave::prelude::*;

behave! {
    "database tests" {
        setup {
            let pool = vec!["conn_1"];
        }

        teardown {
            // Runs after the test body (panic-safe in sync mode).
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

Run only tests tagged `slow`:

```bash
cargo behave --tag slow
```

Watch for file changes and re-run:

```bash
cargo behave --watch
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
| `color` | No | ANSI-colored diff output for assertion failures |
| `regex` | No | `to_match_regex` and `to_contain_regex` matchers |
| `tokio` | No | Enables `tokio;` async test generation |

## Macros

| Macro | Description | Docs |
|-------|-------------|------|
| [`behave!`](docs/macros/behave.md) | BDD test suite DSL with groups, setup, teardown, each, matrix, tags, and more | [Reference](docs/macros/behave.md) |
| [`expect!`](docs/macros/expect.md) | Wrap a value for matcher assertions with structured error messages | [Reference](docs/macros/expect.md) |
| [`expect_panic!`](docs/macros/expect_panic.md) | Assert that an expression panics | [Reference](docs/macros/expect_panic.md) |
| [`expect_no_panic!`](docs/macros/expect_no_panic.md) | Assert that an expression does not panic | [Reference](docs/macros/expect_no_panic.md) |
| [`skip_when!`](docs/macros/skip_when.md) | Conditionally skip a test at runtime with a reason | [Reference](docs/macros/skip_when.md) |

## Matchers

| Category | Matchers |
|----------|----------|
| Equality | [`to_equal`](docs/matchers/to_equal.md), [`to_not_equal`](docs/matchers/to_not_equal.md) |
| Boolean | [`to_be_true`](docs/matchers/to_be_true.md), [`to_be_false`](docs/matchers/to_be_false.md) |
| Ordering | [`to_be_greater_than`](docs/matchers/to_be_greater_than.md), [`to_be_less_than`](docs/matchers/to_be_less_than.md), [`to_be_at_least`](docs/matchers/to_be_at_least.md), [`to_be_at_most`](docs/matchers/to_be_at_most.md) |
| Option | [`to_be_some`](docs/matchers/to_be_some.md), [`to_be_none`](docs/matchers/to_be_none.md), [`to_be_some_with`](docs/matchers/to_be_some_with.md) |
| Result | [`to_be_ok`](docs/matchers/to_be_ok.md), [`to_be_err`](docs/matchers/to_be_err.md), [`to_be_ok_with`](docs/matchers/to_be_ok_with.md), [`to_be_err_with`](docs/matchers/to_be_err_with.md) |
| Collections | [`to_contain`](docs/matchers/to_contain.md), [`to_be_empty`](docs/matchers/to_be_empty.md), [`to_not_be_empty`](docs/matchers/to_not_be_empty.md), [`to_have_length`](docs/matchers/to_have_length.md), [`to_contain_all_of`](docs/matchers/to_contain_all_of.md) |
| Strings | [`to_start_with`](docs/matchers/to_start_with.md), [`to_end_with`](docs/matchers/to_end_with.md), [`to_contain_substr`](docs/matchers/to_contain_substr.md), [`to_have_str_length`](docs/matchers/to_have_str_length.md) |
| Float | [`to_approximately_equal`](docs/matchers/to_approximately_equal.md), [`to_approximately_equal_within`](docs/matchers/to_approximately_equal_within.md) |
| Panic | [`expect_panic!`](docs/matchers/expect_panic.md), [`expect_no_panic!`](docs/matchers/expect_no_panic.md) |
| Predicate | [`to_satisfy`](docs/matchers/to_satisfy.md) |
| Custom | [`to_match`](docs/matchers/to_match.md) with `BehaveMatch` |
| Regex *(feature)* | [`to_match_regex`](docs/matchers/to_match_regex.md), [`to_contain_regex`](docs/matchers/to_contain_regex.md) |
| Map (`HashMap`, `BTreeMap`) | [`to_contain_key`](docs/matchers/to_contain_key.md), [`to_contain_value`](docs/matchers/to_contain_value.md), [`to_contain_entry`](docs/matchers/to_contain_entry.md), [`to_be_empty`](docs/matchers/to_be_empty.md), [`to_not_be_empty`](docs/matchers/to_not_be_empty.md), [`to_have_length`](docs/matchers/to_have_length.md) |
| Composition | [`all_of`](docs/matchers/all_of.md), [`any_of`](docs/matchers/any_of.md), [`not_matching`](docs/matchers/not_matching.md) |

All matchers respect [`.not()` / `.negate()`](docs/matchers/not.md).

The matcher docs live in [docs/matchers/](docs/matchers/README.md) with one page
per matcher (plus a quick index).

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
- use `teardown` blocks for cleanup after each test (panic-safe in sync, error-safe in async)
- declare `tokio;` in a group to generate `#[tokio::test]` async tests (requires `tokio` feature)
- use `cargo test` normally because generated tests are ordinary `#[test]` functions
- use `cargo behave` for tree output, filters, and libtest flags
- use `cargo behave --output json` or `cargo behave --output junit` for CI-friendly reports
- use `cargo behave --manifest-path path/to/Cargo.toml` or `--package name` in workspaces
- use `cargo behave --tag slow` to run only tagged tests, `--exclude-tag flaky` to exclude
- use `cargo behave --focus` to run only focused tests
- use `cargo behave --fail-on-focus` to reject focused tests in CI
- use `cargo behave --watch` to re-run on file changes
- use `skip_when!(condition, "reason")` to conditionally skip tests at runtime

Current limitations:

- one `setup` block per group, one `teardown` block per group
- DSL order within a group: `tokio;` → `timeout` → `setup {}` → `teardown {}` → children
- `pending` blocks must be empty
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
- [Macro Reference](docs/macros/) — [`behave!`](docs/macros/behave.md) | [`expect!`](docs/macros/expect.md) | [`expect_panic!`](docs/macros/expect_panic.md) | [`expect_no_panic!`](docs/macros/expect_no_panic.md) | [`skip_when!`](docs/macros/skip_when.md)
- [Matcher Reference](docs/matchers/README.md)
- [CLI Guide](docs/CLI.md)
- [Reliability](docs/RELIABILITY.md)
- [Architecture](docs/ARCHITECTURE.md)
- [Contributing](CONTRIBUTING.md)
- [API docs on docs.rs](https://docs.rs/behave)

## Security

See [SECURITY.md](SECURITY.md) for the reporting process.

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE).

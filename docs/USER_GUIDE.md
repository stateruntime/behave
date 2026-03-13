# User Guide

This guide is for two audiences:

- users adding `behave` to a Rust project for the first time
- contributors who need the public model explained before reading internals

## What `behave` Gives You

`behave` is a BDD-style layer on top of Rust's normal test system.

- `behave!` defines nested suites and scenarios
- `expect!` wraps a value so you can apply matchers
- every generated test is still a regular `#[test]`
- failures return structured `MatchError` values, which makes `?` work naturally

The core mental model is simple: write scenarios in a readable tree, compile
them into plain Rust tests, then keep using `cargo test`.

## Start In 60 Seconds

1. Add the crate.

```bash
cargo add behave --dev
```

2. Create `tests/behave_smoke.rs`.

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

        "shows subtotal in the receipt" {
            let receipt = format!("subtotal={subtotal}");
            expect!(receipt).to_contain_substr("240")?;
        }
    }
}
```

3. Run the suite.

```bash
cargo test
```

## DSL Overview

### Groups and scenarios

Groups are string labels containing other groups or tests. Leaf blocks are test
bodies.

```rust
use behave::prelude::*;

behave! {
    "math" {
        "adds numbers" {
            expect!(2 + 2).to_equal(4)?;
        }
    }
}
```

### `setup` inheritance

`setup` runs by expansion: its bindings become available in child scenarios.

```rust
use behave::prelude::*;

behave! {
    "invoice" {
        setup {
            let base = 100;
        }

        "uses parent setup" {
            expect!(base + 20).to_equal(120)?;
        }

        "discount branch" {
            setup {
                let discount = 15;
            }

            "inherits both values" {
                expect!(base - discount).to_equal(85)?;
            }
        }
    }
}
```

### Shadowing setup variables

Yes. A child `setup` can shadow a parent binding, and a scenario body can
shadow a setup binding with a later `let`.

```rust
use behave::prelude::*;

behave! {
    "shadowing" {
        setup {
            let amount = 10;
        }

        "scenario body shadowing" {
            let amount = amount + 1;
            expect!(amount).to_equal(11)?;
        }

        "child setup shadowing" {
            setup {
                let amount = 25;
            }

            "uses child value" {
                expect!(amount).to_equal(25)?;
            }
        }
    }
}
```

The reason this works is that `behave!` pastes parent setup code before child
setup code inside the generated test function, so later `let` bindings shadow
earlier ones using normal Rust rules.

See [`examples/setup_inheritance.rs`](../examples/setup_inheritance.rs) for a
three-level nested setup example with a realistic pricing domain.

### Expectations and matchers

`expect!(value)` returns an `Expectation<T>`. Matchers accept expected values
by value.

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(vec![1, 2, 3]).to_have_length(3)?;
    expect!(Some("ok")).to_be_some()?;
    expect!(Ok::<_, &str>(42)).to_be_ok_with(42)?;
    expect!("hello world").to_start_with("hello")?;
    Ok(())
}
```

### Matcher reference

`behave` has built-in matcher families for equality, booleans, ordering,
options, results, collections, strings, floats, panic behavior, and custom
domain-specific rules.

If you want the full explanation of what each matcher checks, why you would use
it, and a working example for every method, read
[docs/matchers/](matchers/README.md).

### Negation

Every matcher respects `.negate()` (also available as `.not()`).

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(3).negate().to_equal(4)?;
    expect!("hello").not().to_end_with("xyz")?;
    Ok(())
}
```

### `pending` and `focus`

Use `pending` to keep planned scenarios visible without running them yet.

```rust
use behave::prelude::*;

behave! {
    "payments" {
        pending "supports refunds" {}
    }
}
```

Use `focus` to mark important scenarios. With `cargo behave --focus`, only
focused tests run. With `cargo behave --fail-on-focus`, the CLI exits non-zero
if any focused tests exist (useful for CI).

```rust
use behave::prelude::*;

behave! {
    "payments" {
        focus "captures a successful charge" {
            expect!(true).to_be_true()?;
        }
    }
}
```

### Tags

Use `tag` to attach metadata labels to groups, tests, `each`, or `matrix`
blocks. Tags are inherited automatically through module nesting.

```rust
use behave::prelude::*;

behave! {
    "database" tag "slow", "integration" {
        "creates a user" {
            expect!(true).to_be_true()?;
        }
    }

    "parser" tag "unit" {
        "tokenizes input" {
            expect!(true).to_be_true()?;
        }
    }
}
```

Filter tests by tag with the CLI:

```bash
cargo behave --tag slow               # run only tests tagged "slow"
cargo behave --exclude-tag flaky       # exclude tests tagged "flaky"
cargo behave --tag unit --exclude-tag slow  # combine both
```

Tags display as `[slow, integration]` in tree output and are stripped from
JUnit/JSON names.

### Runtime Conditional Skip

Use `skip_when!` to skip a test at runtime based on a condition:

```rust,ignore
use behave::prelude::*;

behave! {
    "optional integration tests" {
        "requires redis" {
            skip_when!(std::env::var("REDIS_URL").is_err(), "REDIS_URL not set");
            expect!(true).to_be_true()?;
        }
    }
}
```

When the condition is true, the test returns early with a skip sentinel.
`cargo-behave` detects this and reports the test as `Skipped` (with the `⊘`
symbol). Without the CLI, the test passes silently.

### Expected Failures

Use `xfail` to mark a test that is expected to fail:

```rust,ignore
use behave::prelude::*;

behave! {
    xfail "known broken behavior" {
        expect!(1).to_equal(2)?;
    }
}
```

The test passes when the body returns an error, and fails loudly if it
unexpectedly passes. Works on individual tests, `each`, and `matrix` blocks.

### Teardown

`teardown` blocks run after every test in the group, even when a test panics
(in sync mode). Use them for cleanup that must not be skipped:

```rust
use behave::prelude::*;

behave! {
    "resource lifecycle" {
        setup {
            let resource = vec![1, 2, 3];
        }

        teardown {
            drop(resource);
        }

        "resource is available" {
            expect!(resource).to_have_length(3)?;
        }
    }
}
```

Nested groups can each have their own `teardown`. Inner teardowns run before
outer teardowns (like Rust destructors).

See [`examples/teardown.rs`](../examples/teardown.rs) for nested teardown
patterns and resource management examples.

### Parameterized Tests

Use `each` inside a labeled block to generate one test per case.

**Multi-parameter (tuple) cases:**

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

Each tuple generates a separate `case_0`, `case_1`, etc. test function inside a
module named after the label. Failures are isolated and identifiable in
`cargo test` output.

**Single-parameter cases:**

```rust
use behave::prelude::*;

behave! {
    "positive numbers" {
        each [1, 2, 3, 5, 8] |n| {
            expect!(n).to_be_greater_than(0)?;
        }
    }
}
```

**With inherited setup:**

`each` blocks inherit `setup`, `teardown`, `tokio;`, and `focus` from
their parent group, just like regular tests.

```rust
use behave::prelude::*;

behave! {
    "offsets" {
        setup {
            let base = 10;
        }

        "addition" {
            each [
                (1, 11),
                (5, 15),
            ] |n, expected| {
                expect!(base + n).to_equal(expected)?;
            }
        }
    }
}
```

See [`examples/parameterized.rs`](../examples/parameterized.rs) for a
complete working example with HTTP status codes, tax calculations, and
Fibonacci numbers.

## Custom Matchers

When built-in matchers are not enough, define a type implementing
`BehaveMatch<T>`.

```rust
use behave::prelude::*;

struct IsSortedAscending;

#[allow(clippy::unnecessary_literal_bound)]
impl BehaveMatch<Vec<i32>> for IsSortedAscending {
    fn matches(&self, actual: &Vec<i32>) -> bool {
        actual.windows(2).all(|window| window[0] <= window[1])
    }

    fn description(&self) -> &str {
        "to be sorted in ascending order"
    }
}

fn main() -> Result<(), behave::MatchError> {
    expect!(vec![1, 2, 3]).to_match(IsSortedAscending)?;
    Ok(())
}
```

See [`examples/custom_matcher.rs`](../examples/custom_matcher.rs) for a working
suite.

## Optional CLI

Install the CLI:

```bash
cargo install behave --features cli
```

Run it from a crate using `behave` tests:

```bash
cargo behave
```

The CLI:

- runs `cargo test`
- parses the resulting test names
- renders tree, JSON, or JUnit output
- optionally records workspace-aware flaky-test history

For machine-readable consumers, JSON keeps raw libtest names in
`tests[*].full_name`, while the tree and JUnit views strip internal
`__FOCUS__` / `__PENDING__` prefixes into cleaner display names.

Useful forms:

```bash
cargo behave
cargo behave --output json
cargo behave --output junit
cargo behave --package my_crate
cargo behave --manifest-path crates/api/Cargo.toml
```

The repository also includes a real CLI workspace fixture at
[`examples/cli-workspace/README.md`](../examples/cli-workspace/README.md). CI
runs `cargo-behave` against it to validate JSON, JUnit, and package-aware
history behavior end to end.

See [CLI Guide](CLI.md) for the full invocation model and examples.

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

If a test fails after repeated passes without source changes, the CLI prints a
warning and updates the history file.

## What Works And What Does Not

Supported today:

- nested groups and nested `setup` inheritance
- `each` blocks for parameterized/table-driven test generation
- `matrix` blocks for Cartesian product test generation
- `xfail` for expected-failure tests
- `tag` metadata for grouping and CLI filtering
- `skip_when!` for runtime conditional skipping
- `teardown` blocks with panic-safe cleanup (sync) or error-safe cleanup (async)
- `tokio;` group declaration for `#[tokio::test]` generation (behind `tokio` feature)
- `timeout` keyword for deadline enforcement (inherits through nesting)
- `expect_panic!` and `expect_no_panic!` macros for panic assertions (behind `std` feature)
- soft assertions with `SoftErrors` for collecting multiple failures
- shadowing setup variables with later `let` bindings
- `pending` tests
- `focus` markers with `--focus` (run only focused) and `--fail-on-focus` (CI guard)
- custom matchers
- `cargo behave --tag slow` / `--exclude-tag flaky` for tag filtering
- `cargo behave --watch` for re-running on file changes
- `cargo behave checkout` style test-name filtering
- `cargo behave checkout -- --nocapture` style libtest flag forwarding
- `cargo behave --output json` and `cargo behave --output junit`
- workspace-aware package selection for flaky detection through `--package` and `--manifest-path`

Current limitations:

- only one `setup` block is allowed per group
- only one `teardown` block is allowed per group
- the DSL order within a group is: `tokio;` → `timeout` → `setup {}` → `teardown {}` → children
- `pending` blocks must be empty
- async teardown is error-safe (runs after `?` failures) but not panic-safe (no `catch_unwind` across `.await` points)
- libtest `--format` is reserved by `cargo-behave`
- flaky detection hashes selected package inputs, but it is not a full semantic dependency graph

## Which File Should I Read Next?

- [`examples/quickstart.rs`](../examples/quickstart.rs) for the fastest working example
- [`examples/parameterized.rs`](../examples/parameterized.rs) for `each` blocks with real scenarios
- [`examples/setup_inheritance.rs`](../examples/setup_inheritance.rs) for deeply nested setup
- [`examples/teardown.rs`](../examples/teardown.rs) for panic-safe cleanup patterns
- [`examples/custom_matcher.rs`](../examples/custom_matcher.rs) for reusable matchers
- [`docs/matchers/`](matchers/README.md) for every matcher and its intended use
- [`RELIABILITY.md`](RELIABILITY.md) for trust, support, and limitation signals
- [`tests/smoke.rs`](../tests/smoke.rs) for broad matcher and DSL coverage
- [`ARCHITECTURE.md`](ARCHITECTURE.md) for internals
- [`CONTRIBUTING.md`](../CONTRIBUTING.md) for contributor workflow

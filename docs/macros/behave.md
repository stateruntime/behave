# `behave!` Macro

The `behave!` proc macro is the core of the framework. It defines BDD-style
test suites using a human-readable DSL and compiles them to standard `#[test]`
functions at compile time. No custom test runner needed.

## Syntax

```rust,ignore
use behave::prelude::*;

behave! {
    "group label" {
        "test label" {
            // test body — must return Result<(), MatchError>
            expect!(1 + 1).to_equal(2)?;
        }
    }
}
```

Every string literal in braces is either a **group** (if it contains children)
or a **test** (if it contains a body expression). Groups generate Rust modules;
tests generate `#[test]` functions.

## What It Generates

```rust,ignore
behave! {
    "math" {
        "addition" {
            expect!(2 + 2).to_equal(4)?;
        }
    }
}
```

compiles to approximately:

```rust,ignore
mod math {
    use super::*;

    #[test]
    fn addition() -> Result<(), behave::MatchError> {
        expect!(2 + 2).to_equal(4)?;
        Ok(())
    }
}
```

Labels are slugified into valid Rust identifiers (`"hello world"` becomes
`hello_world`). Rust keywords are used as raw identifiers (`"type"` becomes
`r#type`).

## DSL Constructs

### Groups and Tests

Groups nest arbitrarily deep. Leaf blocks become test functions.

```rust,ignore
behave! {
    "checkout" {
        "pricing" {
            "free shipping over 100" {
                expect!(true).to_be_true()?;
            }
        }

        "inventory" {
            "decrements stock" {
                expect!(10 - 1).to_equal(9)?;
            }
        }
    }
}
```

### `setup`

Shared code that runs before each test in the group. Bindings flow into child
groups and tests automatically.

```rust
use behave::prelude::*;

behave! {
    "order" {
        setup {
            let items = vec![120, 80, 40];
            let total: i32 = items.iter().sum();
        }

        "sums line items" {
            expect!(total).to_equal(240)?;
        }

        "with discount" {
            setup {
                let discounted = total * 9 / 10;
            }

            "applies 10% off" {
                expect!(discounted).to_equal(216)?;
            }
        }
    }
}
```

Rules:
- One `setup` per group
- Child `setup` blocks can shadow parent bindings
- Scenario bodies can shadow setup bindings with `let`

### `teardown`

Cleanup code that runs after each test, even if the test panics (sync) or
returns an error (async).

```rust,ignore
behave! {
    "database" {
        setup {
            let conn = create_connection();
        }

        teardown {
            drop(conn);
        }

        "inserts a row" {
            expect!(conn.insert("data")).to_be_ok()?;
        }
    }
}
```

Rules:
- One `teardown` per group
- Inner teardowns run before outer teardowns (like Rust destructors)
- Sync teardown uses `catch_unwind` for panic safety
- Async teardown is error-safe but not panic-safe

### `each` (Parameterized Tests)

Generates one test per case. Each case becomes its own `#[test]` function.

**Multi-parameter tuples:**

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

Generates `addition::case_0`, `addition::case_1`, `addition::case_2`.

**Single parameter:**

```rust
use behave::prelude::*;

behave! {
    "positive" {
        each [1, 2, 3, 5, 8] |n| {
            expect!(n).to_be_greater_than(0)?;
        }
    }
}
```

**Named cases:**

Put a string label as the first tuple element to name the generated function:

```rust
use behave::prelude::*;

behave! {
    "http status" {
        each [
            ("ok", 200, true),
            ("not_found", 404, false),
        ] |code, success| {
            expect!(code).to_be_greater_than(0)?;
        }
    }
}
```

Generates `http_status::ok` and `http_status::not_found` instead of `case_0`
and `case_1`.

### `matrix` (Cartesian Product Tests)

Generates a test for every combination of values across dimensions:

```rust
use behave::prelude::*;

behave! {
    "formatting" {
        matrix [1, 2, 3] x ["a", "b"] |n, s| {
            let result = format!("{n}{s}");
            expect!(result.len()).to_be_greater_than(1)?;
        }
    }
}
```

Generates `formatting::case_0_0`, `case_0_1`, `case_1_0`, `case_1_1`,
`case_2_0`, `case_2_1`.

Supports 2+ dimensions:

```rust
use behave::prelude::*;

behave! {
    "3d" {
        matrix [1, 2] x [10, 20] x [true, false] |a, b, c| {
            expect!(a + b).to_be_greater_than(0)?;
        }
    }
}
```

### `pending`

Marks a test as ignored. The block body must be empty.

```rust
use behave::prelude::*;

behave! {
    "payments" {
        pending "supports refunds" {}
    }
}
```

Shows as `[pending]` with `○` in `cargo-behave` output.

### `focus`

Marks a test for attention. Encodes a `__FOCUS__` prefix in the generated name.

```rust
use behave::prelude::*;

behave! {
    "auth" {
        focus "login redirects" {
            expect!(true).to_be_true()?;
        }
    }
}
```

With `cargo behave --focus`, only focused tests run. With
`cargo behave --fail-on-focus`, the CLI rejects focused tests (useful for CI).

### `xfail` (Expected Failure)

Marks a test that is expected to fail. The test passes when the body returns
an error, and fails loudly if it unexpectedly succeeds.

```rust
use behave::prelude::*;

behave! {
    xfail "known broken" {
        expect!(1).to_equal(2)?;
    }
}
```

Works on individual tests, `each`, and `matrix` blocks. Cannot be combined
with `pending`.

### `tag` (Test Metadata)

Attaches metadata labels to groups, tests, `each`, or `matrix` blocks:

```rust
use behave::prelude::*;

behave! {
    "database" tag "slow", "integration" {
        "creates a user" {
            expect!(true).to_be_true()?;
        }
    }

    "parser" tag "unit" {
        "tokenizes" {
            expect!(true).to_be_true()?;
        }
    }
}
```

Tags encode as `__TAG_name__` prefixes in generated names. Tag inheritance is
automatic through module nesting.

Filter with the CLI:

```bash
cargo behave --tag slow               # run tests tagged "slow"
cargo behave --exclude-tag flaky       # skip tests tagged "flaky"
cargo behave --tag unit --exclude-tag slow  # combine both
```

### `timeout`

Sets a maximum execution time in milliseconds. Inherits through nesting; inner
values override outer.

```rust,ignore
behave! {
    "api tests" {
        timeout 5000;

        "responds quickly" {
            expect!(true).to_be_true()?;
        }

        "stricter inner deadline" {
            timeout 1000;

            "must be fast" {
                expect!(true).to_be_true()?;
            }
        }
    }
}
```

Sync tests use `recv_timeout`; async tests use `tokio::time::timeout`.

### `tokio;`

Generates `#[tokio::test]` instead of `#[test]` for the group and all children.
Requires the `tokio` feature.

```rust,ignore
behave! {
    "async api" {
        tokio;

        "fetches data" {
            let value = async { 42 }.await;
            expect!(value).to_equal(42)?;
        }
    }
}
```

## DSL Order

Within a group, constructs must appear in this order:

```text
tokio;           (optional)
timeout <ms>;    (optional)
setup { ... }    (optional)
teardown { ... } (optional)
// children: groups, tests, each, matrix, pending, focus, xfail
```

## Feature Requirements

| Construct | Feature |
|-----------|---------|
| Core DSL (groups, tests, setup, teardown) | None (always available) |
| `tokio;` | `tokio` |
| `timeout` (async) | `tokio` |
| `skip_when!` (inside tests) | `std` |

## See Also

- [User Guide](../USER_GUIDE.md) for onboarding and workflow
- [Matcher Reference](../matchers/README.md) for all matchers
- [`expect!`](expect.md) for assertion creation
- [`skip_when!`](skip_when.md) for conditional skipping

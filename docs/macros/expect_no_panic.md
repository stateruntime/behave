# `expect_no_panic!` Macro

Asserts that an expression does **not** panic. Returns `Result<(), MatchError>`,
so it integrates with `?` like all other `behave` assertions.

**Requires:** `std` feature (enabled by default).

## Syntax

```rust,ignore
expect_no_panic!({
    // code that should NOT panic
})?;
```

## When to Use

Use `expect_no_panic!` when you need to verify that code handles edge cases
gracefully without panicking:

- testing that error paths return errors instead of panicking
- verifying safe wrappers around unsafe or panic-prone operations
- asserting graceful degradation under unusual inputs
- regression tests for previously-panicking code paths

## Basic Usage

```rust
use behave::prelude::*;

fn demo() -> Result<(), MatchError> {
    expect_no_panic!({
        let _ = 1 + 1;
    })?;

    Ok(())
}

assert!(demo().is_ok());
```

## Inside `behave!`

```rust
use behave::prelude::*;

behave! {
    "safe operations" {
        "arithmetic does not panic" {
            expect_no_panic!({
                let _ = i32::MAX.wrapping_add(1);
            })?;
        }

        "empty slice access with get" {
            expect_no_panic!({
                let v: Vec<i32> = vec![];
                let _ = v.get(0); // returns None, no panic
            })?;
        }
    }
}
```

## What Happens When It Fails

If the expression **does** panic, the macro catches the panic and returns an
error:

```text
expect!({ risky_operation(); })
  actual: panicked
expected: to not panic
```

This error propagates via `?` like any other `MatchError`.

## Guarding Against Regressions

A common pattern is to use `expect_no_panic!` to prevent regressions in code
that previously panicked:

```rust,ignore
behave! {
    "issue #42 regression" {
        "empty input no longer panics" {
            expect_no_panic!({
                parse_config("");
            })?;
        }
    }
}
```

## How It Works

`expect_no_panic!` wraps the expression in `std::panic::catch_unwind`:

1. If the closure completes normally, the macro returns `Ok(())`
2. If the closure panics, the panic is caught and the macro returns
   `Err(MatchError { ... })`

The panic is fully caught and does not propagate.

## Comparison with `expect_panic!`

| Macro | Passes when | Fails when |
|-------|-------------|------------|
| `expect_no_panic!` | Expression completes normally | Expression panics |
| `expect_panic!` | Expression panics | Expression completes normally |

## See Also

- [`expect_panic!`](expect_panic.md) for the opposite assertion
- [`expect!`](expect.md) for value-based assertions
- [`behave!`](behave.md) for the test suite macro

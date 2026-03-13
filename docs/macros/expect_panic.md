# `expect_panic!` Macro

Asserts that an expression panics. Returns `Result<(), MatchError>`, so it
integrates with `?` like all other `behave` assertions.

**Requires:** `std` feature (enabled by default).

## Syntax

```rust,ignore
expect_panic!({
    // code that should panic
})?;
```

## When to Use

Use `expect_panic!` when you need to verify that code panics under specific
conditions:

- testing boundary checks (out-of-bounds indexing)
- verifying `unwrap()` on `None` or `Err`
- asserting `assert!` macro failures in library code
- validating precondition violations

## Basic Usage

```rust
use behave::prelude::*;

fn demo() -> Result<(), MatchError> {
    expect_panic!({
        let v: Vec<i32> = vec![];
        let _ = v[0]; // panics: index out of bounds
    })?;

    Ok(())
}

assert!(demo().is_ok());
```

## Inside `behave!`

```rust
use behave::prelude::*;

behave! {
    "boundary checks" {
        "empty vec panics on index" {
            expect_panic!({
                let v: Vec<i32> = vec![];
                let _ = v[0];
            })?;
        }

        "unwrap on None panics" {
            expect_panic!({
                let x: Option<i32> = None;
                let _ = x.unwrap();
            })?;
        }
    }
}
```

## What Happens When It Fails

If the expression does **not** panic, the macro returns an error:

```text
expect!({ let _ = v[0]; })
  actual: did not panic
expected: to panic
```

This error propagates via `?` like any other `MatchError`.

## Multi-Statement Blocks

The braces `{ }` can contain any number of statements. The macro catches the
panic from wherever it occurs:

```rust
use behave::prelude::*;

fn demo() -> Result<(), MatchError> {
    expect_panic!({
        let mut data = vec![1, 2, 3];
        data.clear();
        let _ = data[0]; // panics here
    })?;

    Ok(())
}

assert!(demo().is_ok());
```

## How It Works

`expect_panic!` wraps the expression in `std::panic::catch_unwind`:

1. If the closure panics, the panic is caught and the macro returns `Ok(())`
2. If the closure does not panic, the macro returns `Err(MatchError { ... })`

The panic is fully caught and does not propagate. This makes the macro safe to
use in any test context.

## Comparison with `expect_no_panic!`

| Macro | Passes when | Fails when |
|-------|-------------|------------|
| `expect_panic!` | Expression panics | Expression completes normally |
| `expect_no_panic!` | Expression completes normally | Expression panics |

## See Also

- [`expect_no_panic!`](expect_no_panic.md) for the opposite assertion
- [`expect!`](expect.md) for value-based assertions
- [`behave!`](behave.md) for the test suite macro

# `MatchError` in real tests

In practice you usually *don’t* “handle” `MatchError` — you either:

- propagate it with `?` so the test fails with a nice message, or
- capture it with `unwrap_err()` when you want to assert on the failure message/fields.

## The common case: just fail the test

Matchers return `Result<(), behave::MatchError>`, so the normal pattern is:

```rust
use behave::prelude::*;

behave! {
    "match_error" {
        "propagate with ?" {
            expect!(0.1_f64 + 0.2_f64).to_approximately_equal(0.3_f64)?;
        }
    }
}
```

If the matcher fails, the test fails and the framework prints `MatchError` using its `Display` format:

```text
expect!(...)
  actual: ...
expected: ...
```

## When you *do* want to inspect it

Sometimes you want to test the *failure message* (for example, when you’re writing your own matchers).
You can intentionally trigger a failure and inspect the returned `MatchError`.

```rust
use behave::prelude::*;

behave! {
    "match_error" {
        "inspect failure" {
            let err = expect!(1).to_equal(2).unwrap_err();

            // Structured fields:
            assert_eq!(err.expression, "1");
            assert!(err.expected.contains("to equal"));
            assert!(err.actual.contains("1"));

            // Human-friendly message:
            let msg = err.to_string();
            assert!(msg.contains("expect!(1)"));
            assert!(msg.contains("expected:"));
        }
    }
}
```

## See also

- [All matchers](README.md)

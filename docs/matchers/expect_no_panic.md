# `expect_no_panic!`

Asserts that an expression does **not** panic.

Requires feature `std` (enabled by default).

## Example

```rust
use behave::prelude::*;

behave! {
    "expect_no_panic!" {
        "example" {
            expect_no_panic!({
                let _ = 1 + 1;
            })?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "expect_no_panic!" {
        "edge cases" {
            // Fails when the expression panics.
            assert!(expect_no_panic!({ panic!("boom"); }).is_err());
        }
    }
}
```

## See also

- [`expect_panic!`](expect_panic.md)
- [All matchers](README.md)

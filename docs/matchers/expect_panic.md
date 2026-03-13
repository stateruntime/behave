# `expect_panic!`

Asserts that an expression panics.

Requires feature `std` (enabled by default).

## Example

```rust
use behave::prelude::*;

behave! {
    "expect_panic!" {
        "example" {
            expect_panic!({
                let v: Vec<i32> = vec![];
                let _ = v[0];
            })?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "expect_panic!" {
        "edge cases" {
            // Fails when the expression does not panic.
            assert!(expect_panic!({ let _ = 1 + 1; }).is_err());
        }
    }
}
```

## See also

- [`expect_no_panic!`](expect_no_panic.md)
- [All matchers](README.md)

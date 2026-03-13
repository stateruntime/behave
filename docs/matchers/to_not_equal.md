# `to_not_equal`

Asserts `actual != expected`. This is equivalent to `.not().to_equal(expected)`, but often reads better.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_not_equal" {
        "example" {
            expect!(2 + 2).to_not_equal(5)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_not_equal" {
        "edge cases" {
            assert!(expect!(4).to_not_equal(4).is_err());
            assert!(expect!(4).not().to_equal(4).is_err());
        }
    }
}
```

## See also

- [`to_equal`](to_equal.md)
- [Negation (`.not()` / `.negate()`)](not.md)
- [All matchers](README.md)

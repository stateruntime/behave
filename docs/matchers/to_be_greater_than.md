# `to_be_greater_than`

Asserts `actual > bound`.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_greater_than" {
        "example" {
            expect!(10).to_be_greater_than(5)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_greater_than" {
        "edge cases" {
            // Equality does not satisfy a strict comparison.
            assert!(expect!(5).to_be_greater_than(5).is_err());
            assert!(expect!(3).to_be_greater_than(5).is_err());
        }
    }
}
```

## See also

- [`to_be_at_least`](to_be_at_least.md)
- [`to_be_less_than`](to_be_less_than.md)
- [All matchers](README.md)

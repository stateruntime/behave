# `to_be_at_least`

Asserts `actual >= bound` (inclusive lower bound).

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_at_least" {
        "example" {
            expect!(5).to_be_at_least(5)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_at_least" {
        "edge cases" {
            assert!(expect!(4).to_be_at_least(5).is_err());
        }
    }
}
```

## See also

- [`to_be_greater_than`](to_be_greater_than.md)
- [`to_be_at_most`](to_be_at_most.md)
- [All matchers](README.md)

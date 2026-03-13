# `to_be_at_most`

Asserts `actual <= bound` (inclusive upper bound).

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_at_most" {
        "example" {
            expect!(5).to_be_at_most(5)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_at_most" {
        "edge cases" {
            assert!(expect!(6).to_be_at_most(5).is_err());
        }
    }
}
```

## See also

- [`to_be_less_than`](to_be_less_than.md)
- [`to_be_at_least`](to_be_at_least.md)
- [All matchers](README.md)

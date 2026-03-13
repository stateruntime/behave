# `to_be_less_than`

Asserts `actual < bound`.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_less_than" {
        "example" {
            expect!(3).to_be_less_than(10)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_less_than" {
        "edge cases" {
            assert!(expect!(5).to_be_less_than(5).is_err());
            assert!(expect!(10).to_be_less_than(5).is_err());
        }
    }
}
```

## See also

- [`to_be_at_most`](to_be_at_most.md)
- [`to_be_greater_than`](to_be_greater_than.md)
- [All matchers](README.md)

# `to_be_some`

Asserts the value is `Some(_)`. Use this when presence matters more than the exact inner value.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_some" {
        "example" {
            expect!(Some(42)).to_be_some()?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_some" {
        "edge cases" {
            assert!(expect!(None::<i32>).to_be_some().is_err());
            assert!(expect!(Some(1)).not().to_be_some().is_err());
        }
    }
}
```

## See also

- [`to_be_none`](to_be_none.md)
- [`to_be_some_with`](to_be_some_with.md)
- [All matchers](README.md)

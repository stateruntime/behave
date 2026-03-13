# `to_be_some_with`

Asserts the value is `Some(expected)` using `PartialEq` on the inner value.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_some_with" {
        "example" {
            expect!(Some(42)).to_be_some_with(42)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_some_with" {
        "edge cases" {
            assert!(expect!(Some(1)).to_be_some_with(2).is_err());
            assert!(expect!(None::<i32>).to_be_some_with(1).is_err());
        }
    }
}
```

## See also

- [`to_be_some`](to_be_some.md)
- [`to_equal`](to_equal.md)
- [All matchers](README.md)

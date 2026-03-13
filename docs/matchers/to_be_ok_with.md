# `to_be_ok_with`

Asserts the value is `Ok(expected)` using `PartialEq` on the `Ok` payload.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_ok_with" {
        "example" {
            expect!(Ok::<_, &str>(42)).to_be_ok_with(42)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_ok_with" {
        "edge cases" {
            assert!(expect!(Ok::<_, &str>(1)).to_be_ok_with(2).is_err());
            assert!(expect!(Err::<i32, &str>("boom")).to_be_ok_with(1).is_err());
        }
    }
}
```

## See also

- [`to_be_ok`](to_be_ok.md)
- [`to_equal`](to_equal.md)
- [All matchers](README.md)

# `to_be_err_with`

Asserts the value is `Err(expected)` using `PartialEq` on the `Err` payload.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_err_with" {
        "example" {
            expect!(Err::<i32, &str>("boom")).to_be_err_with("boom")?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_err_with" {
        "edge cases" {
            assert!(expect!(Err::<i32, &str>("boom"))
                .to_be_err_with("other")
                .is_err());
            assert!(expect!(Ok::<_, &str>(42)).to_be_err_with("boom").is_err());
        }
    }
}
```

## See also

- [`to_be_err`](to_be_err.md)
- [`to_equal`](to_equal.md)
- [All matchers](README.md)

# `to_be_err`

Asserts the value is `Err(_)`.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_err" {
        "example" {
            let response: Result<i32, &str> = Err("boom");
            expect!(response).to_be_err()?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_err" {
        "edge cases" {
            let response: Result<i32, &str> = Ok(42);
            assert!(expect!(response).to_be_err().is_err());
        }
    }
}
```

## See also

- [`to_be_ok`](to_be_ok.md)
- [`to_be_err_with`](to_be_err_with.md)
- [All matchers](README.md)

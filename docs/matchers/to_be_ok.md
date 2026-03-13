# `to_be_ok`

Asserts the value is `Ok(_)`.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_ok" {
        "example" {
            let response: Result<i32, &str> = Ok(42);
            expect!(response).to_be_ok()?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_ok" {
        "edge cases" {
            let response: Result<i32, &str> = Err("boom");
            assert!(expect!(response).to_be_ok().is_err());
        }
    }
}
```

## See also

- [`to_be_err`](to_be_err.md)
- [`to_be_ok_with`](to_be_ok_with.md)
- [All matchers](README.md)

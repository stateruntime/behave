# `to_end_with`

Asserts a string ends with the given suffix.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_end_with" {
        "example" {
            expect!("hello world").to_end_with("world")?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_end_with" {
        "edge cases" {
            // Empty suffix always matches.
            assert!(expect!("hello").to_end_with("").is_ok());

            assert!(expect!("").to_end_with("a").is_err());
        }
    }
}
```

## See also

- [`to_start_with`](to_start_with.md)
- [`to_contain_substr`](to_contain_substr.md)
- [All matchers](README.md)

# `to_start_with`

Asserts a string starts with the given prefix.

Works with `String`, `&str`, and other `AsRef<str>` string-like types.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_start_with" {
        "example" {
            expect!("hello world").to_start_with("hello")?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_start_with" {
        "edge cases" {
            // Empty prefix always matches.
            assert!(expect!("hello").to_start_with("").is_ok());

            // Non-empty prefix cannot match an empty string.
            assert!(expect!("").to_start_with("a").is_err());
        }
    }
}
```

## See also

- [`to_end_with`](to_end_with.md)
- [`to_contain_substr`](to_contain_substr.md)
- [All matchers](README.md)

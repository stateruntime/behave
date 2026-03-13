# `to_contain_substr`

Asserts a string contains the given substring.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_contain_substr" {
        "example" {
            expect!("hello world").to_contain_substr("lo wo")?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_contain_substr" {
        "edge cases" {
            // Empty substring is always contained.
            assert!(expect!("hello").to_contain_substr("").is_ok());

            assert!(expect!("hello").to_contain_substr("xyz").is_err());
        }
    }
}
```

## See also

- [`to_contain_regex`](to_contain_regex.md) *(feature: `regex`)*
- [All matchers](README.md)

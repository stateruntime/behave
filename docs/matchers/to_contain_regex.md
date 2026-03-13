# `to_contain_regex`

Asserts the string contains a substring that matches a regex.

This matcher requires the `regex` feature.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_contain_regex" {
        "example" {
            expect!("order #42 confirmed").to_contain_regex(r"#\d+")?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_contain_regex" {
        "edge cases" {
            assert!(expect!("hello").to_contain_regex(r"\d+").is_err());
            assert!(expect!("hello").to_contain_regex(r"[invalid").is_err());
        }
    }
}
```

## See also

- [`to_match_regex`](to_match_regex.md)
- [`to_contain_substr`](to_contain_substr.md)
- [All matchers](README.md)

# `to_match_regex`

Asserts the entire string matches a regex.

This matcher requires the `regex` feature.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_match_regex" {
        "example" {
            // Full-string match (auto-anchored).
            expect!("hello123").to_match_regex(r"hello\d+")?;
        }
    }
}
```

## Notes

- The pattern is auto-anchored (it must match the entire string). For substring matching, use
  [`to_contain_regex`](to_contain_regex.md).

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_match_regex" {
        "edge cases" {
            // Partial matches fail because the pattern is auto-anchored.
            assert!(expect!("abc123def").to_match_regex(r"\d+").is_err());

            // Invalid patterns return `MatchError` rather than panicking.
            assert!(expect!("hello").to_match_regex(r"[invalid").is_err());
        }
    }
}
```

## See also

- [`to_contain_regex`](to_contain_regex.md)
- [All matchers](README.md)

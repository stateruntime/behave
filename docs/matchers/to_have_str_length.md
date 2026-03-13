# `to_have_str_length`

Asserts a string has exactly the given **byte length** (`str::len`), not character count.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_have_str_length" {
        "example" {
            expect!("abc").to_have_str_length(3)?;

            // Each emoji is 4 bytes in UTF-8.
            expect!("\u{1F600}\u{1F601}").to_have_str_length(8)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_have_str_length" {
        "edge cases" {
            // Empty string has length 0.
            assert!(expect!("").to_have_str_length(0).is_ok());

            // Multi-byte characters: this is 2 bytes in UTF-8.
            assert!(expect!("é").to_have_str_length(2).is_ok());
        }
    }
}
```

## See also

- [`to_contain_substr`](to_contain_substr.md)
- [All matchers](README.md)

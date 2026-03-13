# `to_be_false`

Asserts the value is `false`.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_false" {
        "example" {
            let is_valid = "abc".ends_with('z');
            expect!(is_valid).to_be_false()?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_false" {
        "edge cases" {
            assert!(expect!(true).to_be_false().is_err());
            assert!(expect!(false).not().to_be_false().is_err());
        }
    }
}
```

## See also

- [`to_be_true`](to_be_true.md)
- [Negation (`.not()` / `.negate()`)](not.md)
- [All matchers](README.md)

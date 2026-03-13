# `to_be_true`

Asserts the value is `true`. Use this when you’re already dealing with a predicate-style boolean.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_true" {
        "example" {
            let is_valid = "abc".starts_with('a');
            expect!(is_valid).to_be_true()?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_true" {
        "edge cases" {
            assert!(expect!(false).to_be_true().is_err());
            assert!(expect!(true).not().to_be_true().is_err());
        }
    }
}
```

## See also

- [`to_be_false`](to_be_false.md)
- [Negation (`.not()` / `.negate()`)](not.md)
- [All matchers](README.md)

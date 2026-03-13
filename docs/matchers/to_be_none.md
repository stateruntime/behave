# `to_be_none`

Asserts the value is `None`.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_none" {
        "example" {
            expect!(None::<i32>).to_be_none()?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_none" {
        "edge cases" {
            assert!(expect!(Some(1)).to_be_none().is_err());
            assert!(expect!(None::<i32>).not().to_be_none().is_err());
        }
    }
}
```

## See also

- [`to_be_some`](to_be_some.md)
- [Negation (`.not()` / `.negate()`)](not.md)
- [All matchers](README.md)

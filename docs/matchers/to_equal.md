# `to_equal`

Asserts `actual == expected` using `PartialEq`.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_equal" {
        "example" {
            expect!(2 + 2).to_equal(4)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_equal" {
        "edge cases" {
            // When it fails, you can inspect the structured `MatchError`.
            let err = expect!(2 + 2).to_equal(5).unwrap_err();
            assert!(err.to_string().contains("expect!(2 + 2)"));
            assert!(err.expected.contains("to equal"));
        }
    }
}
```

## See also

- [`to_not_equal`](to_not_equal.md)
- [Negation (`.not()` / `.negate()`)](not.md)
- [All matchers](README.md)

# `to_satisfy`

Asserts a value satisfies an inline predicate.

Use this for one-off checks when no built-in matcher fits. For reusable domain rules, prefer
[`to_match`](to_match.md) with a `BehaveMatch<T>` implementation.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_satisfy" {
        "example" {
            expect!(42).to_satisfy(|x| x % 2 == 0, "to be even")?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_satisfy" {
        "edge cases" {
            assert!(expect!(3).to_satisfy(|x| x % 2 == 0, "to be even").is_err());

            // Negation flips the predicate result.
            assert!(expect!(3)
                .not()
                .to_satisfy(|x| x % 2 == 0, "to be even")
                .is_ok());
        }
    }
}
```

## See also

- [`to_match`](to_match.md)
- [All matchers](README.md)

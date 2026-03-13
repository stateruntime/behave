# `to_match`

Asserts a value satisfies a custom matcher implementing `BehaveMatch<T>`.

Use this when you want a named, reusable rule (especially for domain logic used across many tests).

## Example

```rust
use behave::prelude::*;

struct IsEven;
impl BehaveMatch<i32> for IsEven {
    fn matches(&self, actual: &i32) -> bool {
        actual % 2 == 0
    }

    fn description(&self) -> &str {
        "to be even"
    }
}

behave! {
    "to_match" {
        "example" {
            expect!(4).to_match(IsEven)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

struct IsEven;
impl BehaveMatch<i32> for IsEven {
    fn matches(&self, actual: &i32) -> bool {
        actual % 2 == 0
    }

    fn description(&self) -> &str {
        "to be even"
    }
}

behave! {
    "to_match" {
        "edge cases" {
            assert!(expect!(3).to_match(IsEven).is_err());
            assert!(expect!(3).not().to_match(IsEven).is_ok());
        }
    }
}
```

## See also

- [`to_satisfy`](to_satisfy.md)
- [`all_of`](all_of.md)
- [`any_of`](any_of.md)
- [All matchers](README.md)

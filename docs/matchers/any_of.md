# `any_of`

Builds a matcher that passes when **at least one** inner matcher passes (logical OR).

## Example

```rust
use behave::prelude::*;

struct IsPositive;
impl BehaveMatch<i32> for IsPositive {
    fn matches(&self, actual: &i32) -> bool { *actual > 0 }
    fn description(&self) -> &str { "to be positive" }
}

struct IsEven;
impl BehaveMatch<i32> for IsEven {
    fn matches(&self, actual: &i32) -> bool { actual % 2 == 0 }
    fn description(&self) -> &str { "to be even" }
}

behave! {
    "any_of" {
        "example" {
            let matcher = any_of(vec![
                Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
                Box::new(IsEven),
            ]);

            expect!(3).to_match(matcher)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "any_of" {
        "edge cases" {
            // Empty list is a fail (nothing can match).
            assert!(expect!(123).to_match(any_of::<i32>(vec![])).is_err());
        }
    }
}
```

## See also

- [`all_of`](all_of.md)
- [`not_matching`](not_matching.md)
- [All matchers](README.md)

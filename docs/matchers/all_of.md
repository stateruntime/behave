# `all_of`

Builds a matcher that passes when **all** inner matchers pass (logical AND).

This is a composition tool for custom `BehaveMatch<T>` matchers.

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
    "all_of" {
        "example" {
            let matcher = all_of(vec![
                Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
                Box::new(IsEven),
            ]);

            expect!(4).to_match(matcher)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "all_of" {
        "edge cases" {
            // Empty list is a pass (vacuous truth).
            expect!(123).to_match(all_of::<i32>(vec![]))?;
        }
    }
}
```

## See also

- [`any_of`](any_of.md)
- [`not_matching`](not_matching.md)
- [`to_match`](to_match.md)
- [All matchers](README.md)

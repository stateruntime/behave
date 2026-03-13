# `not_matching`

Builds a matcher that inverts a single inner matcher.

This is useful inside combinators when you want to negate one matcher without changing the outer expectation chain.

## Example

```rust
use behave::prelude::*;

struct IsEven;
impl BehaveMatch<i32> for IsEven {
    fn matches(&self, actual: &i32) -> bool { actual % 2 == 0 }
    fn description(&self) -> &str { "to be even" }
}

behave! {
    "not_matching" {
        "example" {
            let matcher = not_matching(Box::new(IsEven));
            expect!(3).to_match(matcher)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

struct IsEven;
impl BehaveMatch<i32> for IsEven {
    fn matches(&self, actual: &i32) -> bool { actual % 2 == 0 }
    fn description(&self) -> &str { "to be even" }
}

behave! {
    "not_matching" {
        "edge cases" {
            // Double inversion cancels out.
            let m = not_matching(Box::new(not_matching(Box::new(IsEven))));
            assert!(expect!(4).to_match(m).is_ok());
        }
    }
}
```

## See also

- [`all_of`](all_of.md)
- [`any_of`](any_of.md)
- [Negation (`.not()` / `.negate()`)](not.md)
- [All matchers](README.md)

# `expect!` Macro

Creates an [`Expectation`](https://docs.rs/behave/latest/behave/struct.Expectation.html)
that wraps a value for matcher assertions. This is the primary way to write
assertions in `behave`.

## Syntax

```rust,ignore
expect!(expression).matcher(expected)?;
```

The macro captures both the value and the source expression text, so failure
messages show what you wrote:

```text
expect!(user.age)
  actual: 17
expected: to be at least 18
```

## Basic Usage

```rust
use behave::prelude::*;

fn demo() -> Result<(), MatchError> {
    let count = 3;

    expect!(count).to_equal(3)?;
    expect!(count).to_be_greater_than(0)?;
    expect!(count).to_be_at_most(10)?;

    Ok(())
}

assert!(demo().is_ok());
```

## Inside `behave!`

Every test body returns `Result<(), MatchError>`, so `?` propagates failures
naturally:

```rust
use behave::prelude::*;

behave! {
    "shopping cart" {
        setup {
            let items = vec![10, 20, 30];
            let total: i32 = items.iter().sum();
        }

        "sums correctly" {
            expect!(total).to_equal(60)?;
        }

        "has items" {
            expect!(items).to_not_be_empty()?;
            expect!(items).to_have_length(3)?;
        }
    }
}
```

## Negation

Use `.not()` (or `.negate()`) to invert any matcher:

```rust
use behave::prelude::*;

fn demo() -> Result<(), MatchError> {
    expect!(42).not().to_equal(0)?;
    expect!(vec![1, 2]).not().to_contain(9)?;
    expect!("hello").not().to_start_with("xyz")?;

    Ok(())
}

assert!(demo().is_ok());
```

Failure messages for negated matchers read naturally:

```text
expect!(value)
  actual: 42
expected: not to equal 42
```

## Chaining Multiple Assertions

Use `?` after each assertion to fail fast on the first mismatch:

```rust
use behave::prelude::*;

fn demo() -> Result<(), MatchError> {
    let name = "Alice";

    expect!(name).to_start_with("A")?;
    expect!(name).to_end_with("e")?;
    expect!(name).to_have_str_length(5)?;

    Ok(())
}

assert!(demo().is_ok());
```

For collecting multiple failures without stopping, see
[`SoftErrors`](https://docs.rs/behave/latest/behave/struct.SoftErrors.html):

```rust
use behave::prelude::*;

fn demo() -> Result<(), SoftMatchError> {
    let mut errors = SoftErrors::new();

    errors.check(expect!(1).to_equal(1));
    errors.check(expect!(2).to_equal(2));
    errors.check(expect!(3).to_equal(3));

    errors.finish()?;
    Ok(())
}

assert!(demo().is_ok());
```

## With Custom Matchers

`expect!` works with any type implementing `BehaveMatch<T>`:

```rust
use behave::prelude::*;

struct IsEven;

#[allow(clippy::unnecessary_literal_bound)]
impl BehaveMatch<i32> for IsEven {
    fn matches(&self, actual: &i32) -> bool {
        actual % 2 == 0
    }

    fn description(&self) -> &str {
        "to be even"
    }
}

fn demo() -> Result<(), MatchError> {
    expect!(4).to_match(IsEven)?;
    expect!(7).not().to_match(IsEven)?;

    Ok(())
}

assert!(demo().is_ok());
```

## With Combinators

Compose matchers with `all_of`, `any_of`, and `not_matching`:

```rust
use behave::prelude::*;

struct IsPositive;

#[allow(clippy::unnecessary_literal_bound)]
impl BehaveMatch<i32> for IsPositive {
    fn matches(&self, actual: &i32) -> bool { *actual > 0 }
    fn description(&self) -> &str { "to be positive" }
}

struct IsEven;

#[allow(clippy::unnecessary_literal_bound)]
impl BehaveMatch<i32> for IsEven {
    fn matches(&self, actual: &i32) -> bool { actual % 2 == 0 }
    fn description(&self) -> &str { "to be even" }
}

fn demo() -> Result<(), MatchError> {
    let positive_and_even = all_of(vec![
        Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
        Box::new(IsEven),
    ]);

    expect!(4).to_match(positive_and_even)?;

    Ok(())
}

assert!(demo().is_ok());
```

## Available Matchers

| Category | Matchers |
|----------|----------|
| Equality | `to_equal`, `to_not_equal` |
| Boolean | `to_be_true`, `to_be_false` |
| Ordering | `to_be_greater_than`, `to_be_less_than`, `to_be_at_least`, `to_be_at_most` |
| Option | `to_be_some`, `to_be_none`, `to_be_some_with` |
| Result | `to_be_ok`, `to_be_err`, `to_be_ok_with`, `to_be_err_with` |
| Collections | `to_contain`, `to_be_empty`, `to_not_be_empty`, `to_have_length`, `to_contain_all_of` |
| Strings | `to_start_with`, `to_end_with`, `to_contain_substr`, `to_have_str_length` |
| Float | `to_approximately_equal`, `to_approximately_equal_within` |
| Regex *(feature)* | `to_match_regex`, `to_contain_regex` |
| Map | `to_contain_key`, `to_contain_value`, `to_contain_entry` |
| Predicate | `to_satisfy` |
| Custom | `to_match` |
| Composition | `all_of`, `any_of`, `not_matching` |

See [Matcher Reference](../matchers/README.md) for complete documentation of
every matcher.

## How It Works

`expect!(expr)` expands to:

```rust,ignore
Expectation::new(expr, stringify!(expr))
```

The stringified expression is stored for error messages. The `Expectation<T>`
wrapper provides all matcher methods. Each matcher returns
`Result<(), MatchError>`, so `?` propagates the structured error.

## See Also

- [Matcher Reference](../matchers/README.md) for all matchers and examples
- [`behave!`](behave.md) for the test suite macro
- [`expect_panic!`](expect_panic.md) for panic assertions
- [`expect_no_panic!`](expect_no_panic.md) for no-panic assertions

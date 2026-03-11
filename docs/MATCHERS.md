# Matcher Reference

This guide answers three questions for every matcher:

- what it checks
- why you would pick it over a lower-level assertion
- what a working example looks like

All matcher methods return `Result<(), MatchError>`, so they compose naturally
with `?` inside `behave!` scenarios.

## Equality

Use equality matchers when you want exact value comparison and failure messages
that show both the expression and the expected value.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_equal(expected)` | actual value equals `expected` | clearer scenario-style assertion than `assert_eq!` inside a BDD test |
| `to_not_equal(expected)` | actual value does not equal `expected` | useful when the negative case is the real requirement |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(2 + 2).to_equal(4)?;
    expect!(2 + 2).to_not_equal(5)?;
    Ok(())
}
```

## Boolean

Use boolean matchers when the behavior matters more than the literal expression.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_be_true()` | actual value is `true` | reads well for predicate-style results like `is_valid()` |
| `to_be_false()` | actual value is `false` | avoids noisy `to_equal(false)` style checks |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(true).to_be_true()?;
    expect!(false).to_be_false()?;
    Ok(())
}
```

## Ordering

Use ordering matchers when the exact value is less important than the bound.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_be_greater_than(bound)` | actual value is strictly greater than `bound` | for thresholds and lower limits |
| `to_be_less_than(bound)` | actual value is strictly less than `bound` | for caps and upper limits |
| `to_be_at_least(bound)` | actual value is greater than or equal to `bound` | for inclusive minimums |
| `to_be_at_most(bound)` | actual value is less than or equal to `bound` | for inclusive maximums |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(10).to_be_greater_than(5)?;
    expect!(3).to_be_less_than(10)?;
    expect!(5).to_be_at_least(5)?;
    expect!(5).to_be_at_most(5)?;
    Ok(())
}
```

## Option

Option matchers exist so you can assert structure and contents without manual
`unwrap()` calls.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_be_some()` | actual value is `Some(_)` | confirms presence without binding the inner value |
| `to_be_none()` | actual value is `None` | makes absence an explicit requirement |
| `to_be_some_with(expected)` | actual value is `Some(expected)` | avoids unwrap noise when both presence and value matter |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(Some(42)).to_be_some()?;
    expect!(None::<i32>).to_be_none()?;
    expect!(Some(42)).to_be_some_with(42)?;
    Ok(())
}
```

## Result

Result matchers are for APIs where success/failure shape matters just as much as
the returned data.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_be_ok()` | actual value is `Ok(_)` | verifies success without unpacking |
| `to_be_err()` | actual value is `Err(_)` | makes failure-path expectations explicit |
| `to_be_ok_with(expected)` | actual value is `Ok(expected)` | checks success and payload together |
| `to_be_err_with(expected)` | actual value is `Err(expected)` | checks failure variant and payload together |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    let ok_value: Result<i32, &str> = Ok(42);
    let err_value: Result<i32, &str> = Err("boom");

    expect!(ok_value).to_be_ok()?;
    expect!(err_value).to_be_err()?;
    expect!(Ok::<_, &str>(42)).to_be_ok_with(42)?;
    expect!(Err::<i32, &str>("boom")).to_be_err_with("boom")?;
    Ok(())
}
```

## Collections

Collection matchers let you talk about membership and size directly. They work
on both `Vec<T>` and `&[T]`.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_contain(item)` | collection contains `item` | clearer than hand-written `contains()` checks |
| `to_be_empty()` | collection has no items | makes empty-state behavior obvious |
| `to_not_be_empty()` | collection has at least one item | more direct than negating `to_be_empty()` |
| `to_have_length(len)` | collection length equals `len` | useful when exact size matters |
| `to_contain_all_of(&[...])` | collection contains every listed element (empty slice is vacuous truth) | concise subset-style assertion |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(vec![1, 2, 3]).to_contain(2)?;
    expect!(Vec::<i32>::new()).to_be_empty()?;
    expect!(vec![1]).to_not_be_empty()?;
    expect!(vec![1, 2, 3]).to_have_length(3)?;
    expect!(vec![1, 2, 3]).to_contain_all_of(&[1, 3])?;
    Ok(())
}
```

## Strings

String matchers are for text behavior, not byte-for-byte equality alone.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_start_with(prefix)` | string starts with `prefix` | good for labels, paths, or prefixes |
| `to_end_with(suffix)` | string ends with `suffix` | useful for extensions, IDs, or output tails |
| `to_contain_substr(substr)` | string contains `substr` | useful for logs and human-readable output |
| `to_have_str_length(len)` | byte length equals `len` | useful when exact serialized or protocol length matters |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!("hello world").to_start_with("hello")?;
    expect!("hello world").to_end_with("world")?;
    expect!("hello world").to_contain_substr("lo wo")?;
    expect!("abc").to_have_str_length(3)?;
    Ok(())
}
```

## Floating Point

Float matchers exist because exact equality is often the wrong tool for floating
point math.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_approximately_equal(expected)` | actual value is within the default epsilon of `expected` | useful for normal floating-point calculations |
| `to_approximately_equal_within(expected, epsilon)` | actual value is within a custom epsilon | use when domain tolerance matters |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(0.1_f64 + 0.2_f64).to_approximately_equal(0.3_f64)?;
    expect!(1.005_f64).to_approximately_equal_within(1.0_f64, 0.01_f64)?;
    Ok(())
}
```

## Panic Macros

Use the panic macros when the contract is about whether code panics at all.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `expect_panic!(...)` | enclosed code panics | useful for panic-based contracts or defensive assertions |
| `expect_no_panic!(...)` | enclosed code completes without panicking | useful when stability is the requirement |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect_panic!({
        let values: Vec<i32> = vec![];
        let _ = values[0];
    })?;

    expect_no_panic!({
        let _ = 1 + 1;
    })?;

    Ok(())
}
```

## Predicate

Use `to_satisfy` as a quick escape hatch when no built-in matcher fits and
a full custom matcher type would be overkill.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_satisfy(predicate, description)` | closure returns `true` for the value | inline one-off assertions without defining a custom type |

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(42).to_satisfy(|x| x % 2 == 0, "to be even")?;
    Ok(())
}
```

## Custom Matchers

Use `to_match` when the domain rule is too specific for a built-in matcher
and you want a reusable type.

| Matcher | What it checks | Why use it |
|---------|----------------|------------|
| `to_match(matcher)` | custom matcher returns `true` | lets teams encode domain-specific expectations once and reuse them everywhere |

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

fn main() -> Result<(), behave::MatchError> {
    expect!(4).to_match(IsEven)?;
    Ok(())
}
```

## Negation

Every matcher also supports `.negate()` (also available as `.not()`). Use that
when the negative form reads better than a dedicated inverse matcher.

```rust
use behave::prelude::*;

fn main() -> Result<(), behave::MatchError> {
    expect!(3).not().to_equal(4)?;
    expect!(false).not().to_be_true()?;
    expect!("hello").not().to_start_with("xyz")?;
    Ok(())
}
```

When a dedicated negative matcher exists, like `to_not_equal()` or
`to_not_be_empty()`, prefer the form that reads most clearly in the scenario.

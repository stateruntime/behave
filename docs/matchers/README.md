# Matchers

Matchers are methods on `Expectation<T>`. Use `expect!(expr)` to create an expectation and then chain matchers.

```rust
use behave::prelude::*;

behave! {
    "matchers" {
        "basic usage" {
            expect!(2 + 2).to_equal(4)?;
            expect!("hello").not().to_end_with("xyz")?;
        }
    }
}
```

All matchers return `Result<(), MatchError>`, so `?` composes naturally inside `behave!` scenarios.

## About the examples

The code snippets in these docs are written to be copy-pasteable and to work as standalone doctests:

- Matchers return `Result<(), MatchError>`, so examples often use `?`.
- Most examples use `behave! { ... }` because that’s the primary “real life” way you write matcher assertions.

In your real test suite, you typically write matchers in one of these styles:

- `behave! { ... }` scenarios (BDD-style, recommended if you want the DSL)
- regular Rust `#[test]` functions that return `Result<(), behave::MatchError>` (so you can keep using `?`)

## Index

- Errors: [`MatchError`](match_error.md)
- Negation: [`.not()` / `.negate()`](not.md)

### Equality

- [`to_equal`](to_equal.md)
- [`to_not_equal`](to_not_equal.md)

### Boolean

- [`to_be_true`](to_be_true.md)
- [`to_be_false`](to_be_false.md)

### Ordering

- [`to_be_greater_than`](to_be_greater_than.md)
- [`to_be_less_than`](to_be_less_than.md)
- [`to_be_at_least`](to_be_at_least.md)
- [`to_be_at_most`](to_be_at_most.md)

### Option

- [`to_be_some`](to_be_some.md)
- [`to_be_none`](to_be_none.md)
- [`to_be_some_with`](to_be_some_with.md)

### Result

- [`to_be_ok`](to_be_ok.md)
- [`to_be_err`](to_be_err.md)
- [`to_be_ok_with`](to_be_ok_with.md)
- [`to_be_err_with`](to_be_err_with.md)

### Collections

- [`to_contain`](to_contain.md)
- [`to_contain_all_of`](to_contain_all_of.md)
- [`to_be_empty`](to_be_empty.md)
- [`to_not_be_empty`](to_not_be_empty.md)
- [`to_have_length`](to_have_length.md)

### Strings

- [`to_start_with`](to_start_with.md)
- [`to_end_with`](to_end_with.md)
- [`to_contain_substr`](to_contain_substr.md)
- [`to_have_str_length`](to_have_str_length.md)

### Floating point

- [`to_approximately_equal`](to_approximately_equal.md)
- [`to_approximately_equal_within`](to_approximately_equal_within.md)

### Regex (feature: `regex`)

- [`to_match_regex`](to_match_regex.md)
- [`to_contain_regex`](to_contain_regex.md)

### Maps (feature: `std`, enabled by default)

- [`to_contain_key`](to_contain_key.md)
- [`to_contain_value`](to_contain_value.md)
- [`to_contain_entry`](to_contain_entry.md)
- [`to_be_empty`](to_be_empty.md)
- [`to_not_be_empty`](to_not_be_empty.md)
- [`to_have_length`](to_have_length.md)

### Panic macros (feature: `std`, enabled by default)

- [`expect_panic!`](expect_panic.md)
- [`expect_no_panic!`](expect_no_panic.md)

### Predicate and custom matchers

- [`to_satisfy`](to_satisfy.md)
- [`to_match`](to_match.md)

### Composition (custom matchers)

- [`all_of`](all_of.md)
- [`any_of`](any_of.md)
- [`not_matching`](not_matching.md)

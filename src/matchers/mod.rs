//! Built-in matchers for the [`expect!`](crate::expect) macro.
//!
//! Each submodule provides matcher methods on [`Expectation`](crate::Expectation)
//! via trait implementations. All matchers return `Result<(), MatchError>` so
//! the `?` operator propagates failures with clear diagnostics.
//!
//! Every matcher supports negation via
//! [`.not()`](crate::Expectation::not) /
//! [`.negate()`](crate::Expectation::negate).
//!
//! ## Matcher Categories
//!
//! | Module | Matchers | Use when |
//! |--------|----------|----------|
//! | `equality` | `to_equal`, `to_not_equal` | exact value equality matters |
//! | `boolean` | `to_be_true`, `to_be_false` | the expression is already a predicate |
//! | `ordering` | `to_be_greater_than`, `to_be_less_than`, `to_be_at_least`, `to_be_at_most` | bounds matter more than exact values |
//! | `option` | `to_be_some`, `to_be_none`, `to_be_some_with` | asserting `Option` shape without manual matching |
//! | `result` | `to_be_ok`, `to_be_err`, `to_be_ok_with`, `to_be_err_with` | asserting `Result` shape without manual matching |
//! | `collections` | `to_contain`, `to_contain_all_of`, `to_be_empty`, `to_not_be_empty`, `to_have_length` | size and membership |
//! | `strings` | `to_start_with`, `to_end_with`, `to_contain_substr`, `to_have_str_length` | text shape |
//! | `float` | `to_approximately_equal`, `to_approximately_equal_within` | float comparison with epsilon tolerance |
//! | `regex` | `to_match_regex`, `to_contain_regex` | pattern matching *(requires `regex` feature)* |
//!
//! Additional matchers defined outside this module:
//!
//! - [`to_satisfy(f, desc)`](crate::Expectation::to_satisfy) — custom predicate
//! - [`to_match(m)`](crate::Expectation::to_match) — custom [`BehaveMatch`](crate::BehaveMatch) impl
//! - [`expect_panic!`](crate::expect_panic) / [`expect_no_panic!`](crate::expect_no_panic) — panic assertions

mod boolean;
mod collections;
mod equality;
mod float;
mod option;
mod ordering;
#[cfg(feature = "regex")]
mod regex;
mod result;
mod strings;

//! Built-in matchers for the `expect!` macro.
//!
//! Each submodule provides matcher methods on [`Expectation`](crate::Expectation)
//! via trait implementations. All matchers return `Result<(), MatchError>` so
//! the `?` operator works inside test bodies.
//!
//! ## Matcher Categories
//!
//! | Module | Matchers | Use when |
//! |--------|----------|----------|
//! | `equality` | `to_equal`, `to_not_equal` | exact value equality matters |
//! | `boolean` | `to_be_true`, `to_be_false` | the expression is already a predicate |
//! | `ordering` | `to_be_greater_than`, `to_be_less_than`, etc. | bounds matter more than exact values |
//! | `option` | `to_be_some`, `to_be_none`, etc. | you want to avoid manual `unwrap()` logic |
//! | `result` | `to_be_ok`, `to_be_err`, etc. | success or failure shape is part of the contract |
//! | `collections` | `to_contain`, `to_be_empty`, `to_have_length`, etc. | size and membership are the behavior under test |
//! | `strings` | `to_start_with`, `to_end_with`, etc. | output text shape matters |
//! | `float` | `to_approximately_equal` | exact float equality is too strict |
//!
//! See the repository matcher guide for a fuller user-facing explanation of what
//! each matcher does and when to prefer it.

mod boolean;
mod collections;
mod equality;
mod float;
mod option;
mod ordering;
mod result;
mod strings;

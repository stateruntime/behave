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
//! | `strings` | `to_start_with`, `to_end_with`, `to_contain_substr`, `to_have_str_length`, `to_have_char_count`, `to_be_empty`, `to_not_be_empty` | text shape |
//! | `float` | `to_approximately_equal`, `to_approximately_equal_within`, `to_be_nan`, `to_be_finite`, `to_be_infinite`, `to_be_positive`, `to_be_negative` | float comparison and shape |
//! | `sequences` | `to_contain_exactly`, `to_contain_exactly_in_any_order`, `to_start_with_elements`, `to_end_with_elements`, `to_be_sorted` | ordered sequence assertions |
//! | `regex` | `to_match_regex`, `to_contain_regex` | pattern matching *(requires `regex` feature)* |
//! | `hashmap` | `to_contain_key`, `to_contain_value`, `to_contain_entry`, `to_be_empty`, `to_not_be_empty`, `to_have_length` | `HashMap` and `BTreeMap` assertions *(requires `std` feature)* |
//! | `sets` | `to_contain`, `to_be_empty`, `to_not_be_empty`, `to_have_length`, `to_be_subset_of`, `to_be_superset_of` | `HashSet` and `BTreeSet` assertions *(requires `std` feature)* |
//! | `paths` | `to_exist`, `to_be_a_file`, `to_be_a_directory`, `to_have_extension`, `to_have_file_name` | filesystem path assertions *(requires `std` feature)* |
//! | `json` | `to_have_field`, `to_have_field_value`, `to_be_json_superset_of` | JSON value assertions *(requires `json` feature)* |
//! | `http` | `to_be_success`, `to_be_redirect`, `to_be_client_error`, `to_be_server_error`, `to_have_status_code`, `to_have_header`, `to_have_header_value` | HTTP assertions *(requires `http` feature)* |
//! | `url_matchers` | `to_have_scheme`, `to_have_host`, `to_have_path`, `to_have_query_param`, `to_have_query_param_value`, `to_have_fragment` | URL assertions *(requires `url` feature)* |
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
#[cfg(feature = "std")]
mod hashmap;
#[cfg(feature = "http")]
mod http;
#[cfg(feature = "json")]
mod json;
mod option;
mod ordering;
#[cfg(feature = "std")]
mod paths;
#[cfg(feature = "regex")]
mod regex;
mod result;
mod sequences;
#[cfg(feature = "std")]
mod sets;
mod strings;
#[cfg(feature = "url")]
mod url_matchers;

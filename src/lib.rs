#![cfg_attr(docsrs, feature(doc_cfg))]
//! # behave
//!
//! A BDD testing framework for Rust with a zero-keyword DSL.
//!
//! `behave` provides a [`behave!`] macro for writing readable test suites and
//! an [`expect!`] macro for expressive assertions. Test suites compile to
//! standard `#[test]` functions — no custom test runner needed.
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use behave::prelude::*;
//!
//! behave! {
//!     "arithmetic" {
//!         "addition" {
//!             expect!(2 + 2).to_equal(4)?;
//!         }
//!
//!         "subtraction" {
//!             expect!(10 - 3).to_equal(7)?;
//!         }
//!     }
//! }
//! ```
//!
//! Every matcher returns `Result<(), MatchError>`, so use `?` to propagate
//! failures with clear diagnostics. When an assertion fails you see:
//!
//! ```text
//! expect!(2 + 2)
//!   actual: 4
//! expected: to equal 5
//! ```
//!
//! ## Matcher Reference
//!
//! All matchers are methods on [`Expectation`]. Use [`expect!`] to create one.
//! Every matcher supports negation via [`.not()`](Expectation::not).
//!
//! | Matcher | Description |
//! |---------|-------------|
//! | **Equality** | |
//! | `.to_equal(v)` | Exact equality (`==`) |
//! | `.to_not_equal(v)` | Exact inequality (`!=`) |
//! | **Boolean** | |
//! | `.to_be_true()` | Value is `true` |
//! | `.to_be_false()` | Value is `false` |
//! | **Ordering** | |
//! | `.to_be_greater_than(v)` | Strictly greater |
//! | `.to_be_less_than(v)` | Strictly less |
//! | `.to_be_at_least(v)` | Greater or equal (`>=`) |
//! | `.to_be_at_most(v)` | Less or equal (`<=`) |
//! | **Option** | |
//! | `.to_be_some()` | Value is `Some(_)` |
//! | `.to_be_none()` | Value is `None` |
//! | `.to_be_some_with(v)` | Value is `Some(v)` |
//! | **Result** | |
//! | `.to_be_ok()` | Value is `Ok(_)` |
//! | `.to_be_err()` | Value is `Err(_)` |
//! | `.to_be_ok_with(v)` | Value is `Ok(v)` |
//! | `.to_be_err_with(v)` | Value is `Err(v)` |
//! | **Collections** (`Vec<T>`, `&[T]`) | |
//! | `.to_contain(v)` | Contains element |
//! | `.to_contain_all_of(&[..])` | Contains every element |
//! | `.to_be_empty()` | Length is zero |
//! | `.to_not_be_empty()` | Length is non-zero |
//! | `.to_have_length(n)` | Exact length |
//! | **Strings** | |
//! | `.to_start_with(s)` | Has prefix |
//! | `.to_end_with(s)` | Has suffix |
//! | `.to_contain_substr(s)` | Contains substring |
//! | `.to_have_str_length(n)` | Byte length |
//! | `.to_have_char_count(n)` | Unicode character count |
//! | `.to_be_empty()` | String is empty |
//! | `.to_not_be_empty()` | String is non-empty |
//! | **Floating-Point** | |
//! | `.to_approximately_equal(v)` | Within default epsilon |
//! | `.to_approximately_equal_within(v, e)` | Within custom epsilon |
//! | `.to_be_nan()` | Value is NaN |
//! | `.to_be_finite()` | Not infinite, not NaN |
//! | `.to_be_infinite()` | Positive or negative infinity |
//! | `.to_be_positive()` | Strictly greater than zero |
//! | `.to_be_negative()` | Strictly less than zero |
//! | **Sequences** (`Vec<T>`, `&[T]`) | |
//! | `.to_contain_exactly(&[..])` | Exact ordered match |
//! | `.to_contain_exactly_in_any_order(&[..])` | Same elements, any order |
//! | `.to_start_with_elements(&[..])` | Prefix match |
//! | `.to_end_with_elements(&[..])` | Suffix match |
//! | `.to_be_sorted()` | Non-descending order |
//! | **Sets** (`HashSet`, `BTreeSet`) | |
//! | `.to_contain(&v)` | Set has element |
//! | `.to_be_subset_of(&set)` | All elements in other set |
//! | `.to_be_superset_of(&set)` | Contains all from other set |
//! | **Paths** (`PathBuf`, `&Path`) | |
//! | `.to_exist()` | Path exists on filesystem |
//! | `.to_be_a_file()` | Is a regular file |
//! | `.to_be_a_directory()` | Is a directory |
//! | `.to_have_extension(ext)` | Has file extension |
//! | `.to_have_file_name(name)` | Has file name |
//! | **Regex** *(requires `regex` feature)* | |
//! | `.to_match_regex(pat)` | Full-string regex match |
//! | `.to_contain_regex(pat)` | Substring regex match |
//! | **JSON** *(requires `json` feature)* | |
//! | `.to_have_field(f)` | Object has key |
//! | `.to_have_field_value(f, v)` | Key has specific value |
//! | `.to_be_json_superset_of(v)` | Recursive partial match |
//! | **HTTP** *(requires `http` feature)* | |
//! | `.to_be_success()` | Status 2xx |
//! | `.to_be_redirect()` | Status 3xx |
//! | `.to_be_client_error()` | Status 4xx |
//! | `.to_be_server_error()` | Status 5xx |
//! | `.to_have_status_code(n)` | Exact status code |
//! | `.to_have_header(name)` | Header present |
//! | `.to_have_header_value(name, val)` | Header has value |
//! | **URL** *(requires `url` feature)* | |
//! | `.to_have_scheme(s)` | URL scheme |
//! | `.to_have_host(h)` | URL host |
//! | `.to_have_path(p)` | URL path |
//! | `.to_have_query_param(k)` | Query param exists |
//! | `.to_have_query_param_value(k, v)` | Query param has value |
//! | `.to_have_fragment(f)` | URL fragment |
//! | **General** | |
//! | `.to_satisfy(f, desc)` | Custom predicate function |
//! | `.to_match(m)` | Custom [`BehaveMatch`] impl |
//! | [`expect_panic!`] | Expression panics |
//! | [`expect_no_panic!`] | Expression does not panic |
//! | [`expect_match!`] | Matches a pattern |
//! | **Composition** ([`combinators`]) | |
//! | [`all_of`](combinators::all_of) | All matchers must pass |
//! | [`any_of`](combinators::any_of) | At least one must pass |
//! | [`not_matching`](combinators::not_matching) | Inverts one matcher |
//! | **Map** (`HashMap`, `BTreeMap`) | |
//! | `.to_contain_key(k)` | Map has key |
//! | `.to_contain_value(v)` | Map has value |
//! | `.to_contain_entry(k, v)` | Map has key-value pair |
//! | **Soft Assertions** ([`SoftErrors`]) | |
//! | [`SoftErrors::check`] | Collect result without stopping |
//! | [`SoftErrors::finish`] | Report all collected failures |
//!
//! ## Negation
//!
//! Any matcher can be negated with [`.not()`](Expectation::not) or
//! [`.negate()`](Expectation::negate):
//!
//! ```
//! use behave::prelude::*;
//!
//! # fn demo() -> Result<(), behave::MatchError> {
//! expect!(42).not().to_equal(99)?;
//! expect!(vec![1, 2]).not().to_contain(9)?;
//! # Ok(())
//! # }
//! # assert!(demo().is_ok());
//! ```
//!
//! Negated failures read naturally:
//!
//! ```text
//! expect!(value)
//!   actual: 42
//! expected: not to equal 42
//! ```
//!
//! ## DSL Features
//!
//! The [`behave!`] macro supports these constructs:
//!
//! - **`setup { ... }`** — shared setup code inherited by nested tests
//! - **`teardown { ... }`** — cleanup code that runs after each test
//! - **`each [...] |args| { ... }`** — parameterized test generation
//! - **`pending "name" { ... }`** — mark tests as ignored
//! - **`focus "name" { ... }`** — mark tests with a `__FOCUS__` prefix in generated names
//! - **`tag "name1", "name2"`** — attach metadata tags for CLI filtering
//! - **`xfail "name" { ... }`** — mark a test as expected-to-fail
//! - **`matrix [...] x [...] |a, b| { ... }`** — Cartesian product test generation
//! - **`tokio;`** — generate async tests *(requires `tokio` feature)*
//! - **`timeout <ms>;`** — fail tests that exceed a deadline (inherits through nesting)
//! - **`skip_when!(condition, "reason")`** — skip a test conditionally at runtime
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std`   | Yes     | Standard library support |
//! | `cli`   | No      | Enables `cargo-behave` binary |
//! | `color` | No      | ANSI-colored diff output for assertion failures |
//! | `regex` | No      | `to_match_regex` and `to_contain_regex` matchers |
//! | `tokio` | No      | Re-exports `tokio` for `tokio;` async test generation |
//! | `http`  | No      | HTTP status code and header matchers |
//! | `url`   | No      | URL component matchers |
//! | `json`  | No      | JSON value matchers |

pub mod combinators;
mod custom;
mod error;
mod expectation;
mod matchers;
#[cfg(feature = "std")]
mod soft;

#[cfg(feature = "cli")]
pub mod cli;

pub use behave_macros::behave;
pub use custom::BehaveMatch;
pub use error::MatchError;
pub use expectation::Expectation;
#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub use soft::{SoftErrors, SoftMatchError};

/// Creates an [`Expectation`] capturing the expression and its value.
///
/// The macro stringifies the expression for use in error messages, so
/// `expect!(x + 1)` produces output like `expect!(x + 1)` on failure.
///
/// Returns `Result<(), MatchError>` — use `?` to propagate failures.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
///
/// fn demo() -> Result<(), behave::MatchError> {
///     let value = 42;
///     expect!(value).to_equal(42)?;
///     expect!(value).not().to_equal(0)?;
///     Ok(())
/// }
///
/// assert!(demo().is_ok());
/// ```
#[macro_export]
macro_rules! expect {
    ($expr:expr) => {
        $crate::Expectation::new($expr, stringify!($expr))
    };
}

/// Asserts that the given expression panics.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
///
/// fn demo() -> Result<(), behave::MatchError> {
///     expect_panic!({
///         let v: Vec<i32> = vec![];
///         let _ = v[0];
///     })?;
///     Ok(())
/// }
///
/// assert!(demo().is_ok());
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! expect_panic {
    ($expr:expr) => {{
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            $expr;
        }));
        if result.is_ok() {
            Err($crate::MatchError::new(
                stringify!($expr).to_string(),
                "to panic".to_string(),
                "did not panic".to_string(),
                false,
            ))
        } else {
            Ok(())
        }
    }};
}

/// Asserts that the given expression does not panic.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
///
/// fn demo() -> Result<(), behave::MatchError> {
///     expect_no_panic!({
///         let _ = 1 + 1;
///     })?;
///     Ok(())
/// }
///
/// assert!(demo().is_ok());
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! expect_no_panic {
    ($expr:expr) => {{
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            $expr;
        }));
        if result.is_err() {
            Err($crate::MatchError::new(
                stringify!($expr).to_string(),
                "to not panic".to_string(),
                "panicked".to_string(),
                false,
            ))
        } else {
            Ok(())
        }
    }};
}

/// Asserts that an expression matches a pattern, with optional guard.
///
/// Use this for `enum` variant checks and destructuring where regular
/// matchers cannot express the shape. Named `expect_match!` to follow
/// the existing `expect_panic!` / `expect_no_panic!` convention.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
///
/// #[derive(Debug)]
/// enum Status { Active, Inactive }
///
/// fn demo() -> Result<(), behave::MatchError> {
///     expect_match!(Status::Active, Status::Active)?;
///     expect_match!(Some(42), Some(v) if *v > 0)?;
///     Ok(())
/// }
///
/// assert!(demo().is_ok());
/// ```
#[macro_export]
macro_rules! expect_match {
    ($expr:expr, $($pattern:pat_param)|+ $(if $guard:expr)?) => {{
        let __behave_val = &$expr;
        if matches!(__behave_val, $($pattern)|+ $(if $guard)?) {
            Ok(())
        } else {
            Err($crate::MatchError::new(
                stringify!($expr).to_string(),
                concat!("to match pattern ", stringify!($($pattern)|+ $(if $guard)?)).to_string(),
                format!("{:?}", __behave_val),
                false,
            ))
        }
    }};
}

/// Conditionally skips a test at runtime with a reason.
///
/// When the condition is `true`, prints a sentinel line and returns early.
/// The `cargo-behave` CLI detects the sentinel and reclassifies the test
/// as `Skipped`.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
///
/// fn demo() -> Result<(), behave::MatchError> {
///     skip_when!(cfg!(windows), "only runs on unix");
///     expect!(1 + 1).to_equal(2)?;
///     Ok(())
/// }
///
/// assert!(demo().is_ok());
/// ```
#[cfg(feature = "std")]
#[macro_export]
macro_rules! skip_when {
    ($cond:expr, $reason:expr) => {
        if $cond {
            #[allow(clippy::print_stdout)]
            {
                println!("BEHAVE_SKIP: {}", $reason);
            }
            return Ok(());
        }
    };
}

/// Prelude module that re-exports everything needed for writing tests.
///
/// # Examples
///
/// ```
/// use behave::prelude::*;
///
/// let _ = expect!(1 + 1);
/// ```
pub mod prelude {
    pub use crate::combinators::{all_of, any_of, not_matching};
    pub use crate::custom::BehaveMatch;
    pub use crate::error::MatchError;
    pub use crate::expectation::Expectation;
    pub use crate::{behave, expect, expect_match};

    #[cfg(feature = "std")]
    pub use crate::soft::{SoftErrors, SoftMatchError};

    #[cfg(feature = "std")]
    pub use crate::{expect_no_panic, expect_panic, skip_when};
}

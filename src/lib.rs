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
//! | **Floating-Point** | |
//! | `.to_approximately_equal(v)` | Within default epsilon |
//! | `.to_approximately_equal_within(v, e)` | Within custom epsilon |
//! | **Regex** *(requires `regex` feature)* | |
//! | `.to_match_regex(pat)` | Full-string regex match |
//! | `.to_contain_regex(pat)` | Substring regex match |
//! | **General** | |
//! | `.to_satisfy(f, desc)` | Custom predicate function |
//! | `.to_match(m)` | Custom [`BehaveMatch`] impl |
//! | [`expect_panic!`] | Expression panics |
//! | [`expect_no_panic!`] | Expression does not panic |
//! | **Composition** ([`combinators`]) | |
//! | [`all_of`](combinators::all_of) | All matchers must pass |
//! | [`any_of`](combinators::any_of) | At least one must pass |
//! | [`not_matching`](combinators::not_matching) | Inverts one matcher |
//! | **Map** (`HashMap`, `BTreeMap`) | |
//! | `.to_contain_key(k)` | Map has key |
//! | `.to_contain_value(v)` | Map has value |
//! | `.to_contain_entry(k, v)` | Map has key-value pair |
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
//! - **`focus "name" { ... }`** — mark tests with a __FOCUS__ prefix in generated names
//! - **`tokio;`** — generate async tests *(requires `tokio` feature)*
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

pub mod combinators;
mod custom;
mod error;
mod expectation;
mod matchers;

#[cfg(feature = "cli")]
pub mod cli;

pub use behave_macros::behave;
pub use custom::BehaveMatch;
pub use error::MatchError;
pub use expectation::Expectation;

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
    pub use crate::{behave, expect};

    #[cfg(feature = "std")]
    pub use crate::{expect_no_panic, expect_panic};
}

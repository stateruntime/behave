//! # behave
//!
//! A BDD testing framework for Rust with a zero-keyword DSL.
//!
//! `behave` provides a `behave!` macro for writing readable test suites and
//! an `expect!` macro for expressive assertions. Test suites compile to
//! standard `#[test]` functions with no custom test runtime.
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
//!     }
//! }
//! ```
//!
//! ## Feature Flags
//!
//! | Feature | Default | Description |
//! |---------|---------|-------------|
//! | `std`   | Yes     | Standard library support |
//! | `cli`   | No      | Enables `cargo-behave` binary |
//! | `tokio` | No      | Re-exports `tokio` for `tokio;` async test generation |

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
/// # Examples
///
/// ```
/// use behave::prelude::*;
///
/// fn demo() -> Result<(), behave::MatchError> {
///     let value = 42;
///     expect!(value).to_equal(42)?;
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
    pub use crate::custom::BehaveMatch;
    pub use crate::error::MatchError;
    pub use crate::expectation::Expectation;
    pub use crate::{behave, expect};

    #[cfg(feature = "std")]
    pub use crate::{expect_no_panic, expect_panic};
}

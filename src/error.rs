//! Crate-level error types.

use std::fmt;

/// The main error type for this crate.
///
/// All public functions return `Result<T, Error>` using this type.
/// Uses `#[non_exhaustive]` so new variants can be added without breaking
/// downstream code.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Placeholder error variant. Replace with actual error cases.
    OperationFailed {
        /// Human-readable explanation of what went wrong.
        reason: String,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OperationFailed { reason } => write!(f, "operation failed: {reason}"),
        }
    }
}

impl std::error::Error for Error {}

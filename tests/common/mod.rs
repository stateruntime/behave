//! Shared helpers for integration tests.
//!
//! Import from any test file with `mod common;`.

use behave::prelude::*;

/// A 2D point for testing custom types and matchers.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Point {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Point {
    /// Creates a new point.
    pub(crate) const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Returns the origin `(0, 0)`.
    pub(crate) const fn origin() -> Self {
        Self::new(0, 0)
    }
}

/// Matcher that checks if a point is at the origin.
pub(crate) struct IsOrigin;

impl BehaveMatch<Point> for IsOrigin {
    fn matches(&self, actual: &Point) -> bool {
        actual.x == 0 && actual.y == 0
    }

    fn description(&self) -> &str {
        "to be the origin (0, 0)"
    }
}

/// Doubles a number. Example shared helper.
pub(crate) fn double(n: i32) -> i32 {
    n * 2
}

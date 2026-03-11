//! Parameterized tests using `each` blocks.
//!
//! `each` generates one `#[test]` function per case inside a module named after
//! the label. Failures are isolated so `cargo test` shows exactly which case
//! broke.

use behave::prelude::*;

/// Tiny helper used by the examples below.
#[allow(dead_code)]
const fn http_status_class(code: u16) -> &'static str {
    match code / 100 {
        2 => "success",
        3 => "redirect",
        4 => "client error",
        5 => "server error",
        _ => "unknown",
    }
}

behave! {
    "parameterized tests" {
        "addition table" {
            each [
                (2, 2, 4),
                (0, 0, 0),
                (-1, 1, 0),
                (100, -50, 50),
            ] |a, b, expected| {
                expect!(a + b).to_equal(expected)?;
            }
        }

        "HTTP status codes" {
            each [
                (200_u16, "success"),
                (301_u16, "redirect"),
                (404_u16, "client error"),
                (500_u16, "server error"),
            ] |code, expected_class| {
                expect!(http_status_class(code)).to_equal(expected_class)?;
            }
        }

        "single-param: Fibonacci numbers are positive" {
            each [1, 1, 2, 3, 5, 8, 13, 21] |n| {
                expect!(n).to_be_greater_than(0)?;
            }
        }

        "each with inherited setup" {
            setup {
                let tax_rate = 0.08_f64;
            }

            "tax calculation" {
                each [
                    (100.0_f64, 108.0_f64),
                    (50.0_f64, 54.0_f64),
                    (0.0_f64, 0.0_f64),
                ] |price, expected_total| {
                    let total = price.mul_add(tax_rate, price);
                    expect!(total).to_approximately_equal(expected_total)?;
                }
            }
        }
    }
}

#[allow(clippy::missing_const_for_fn, dead_code)]
fn main() {}

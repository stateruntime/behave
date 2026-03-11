//! Quick start example for the `behave` crate.

use behave::prelude::*;

behave! {
    "cart totals" {
        setup {
            let prices = [120, 80, 40];
            let subtotal: i32 = prices.iter().sum();
        }

        "adds line items" {
            expect!(subtotal).to_equal(240)?;
        }

        "supports matcher negation" {
            expect!(subtotal).not().to_equal(0)?;
        }

        "checks text output" {
            let receipt = format!("subtotal={subtotal}");
            expect!(receipt).to_contain_substr("240")?;
        }
    }

    pending "applies coupon codes" {}
}

#[allow(clippy::missing_const_for_fn, dead_code)]
fn main() {}

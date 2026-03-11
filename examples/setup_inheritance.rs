//! Nested setup inheritance across multiple levels.
//!
//! Each `setup` block adds bindings that are visible to all descendant
//! scenarios. `behave!` pastes parent setup code before child setup code
//! in the generated test function, so later `let` bindings can shadow
//! earlier ones using normal Rust rules.

use behave::prelude::*;

/// A small pricing model used by the examples below.
#[allow(dead_code)]
#[derive(Debug, Clone)]
struct LineItem {
    name: &'static str,
    cents: i64,
}

#[allow(dead_code)]
impl LineItem {
    const fn new(name: &'static str, cents: i64) -> Self {
        Self { name, cents }
    }
}

#[allow(dead_code)]
fn subtotal(items: &[LineItem]) -> i64 {
    items.iter().map(|i| i.cents).sum()
}

#[allow(dead_code)]
const fn apply_discount(total: i64, percent: i64) -> i64 {
    total - (total * percent / 100)
}

behave! {
    "order pricing" {
        setup {
            let items = vec![
                LineItem::new("Widget", 1200),
                LineItem::new("Gadget", 800),
                LineItem::new("Doohickey", 350),
            ];
        }

        "subtotal sums line items" {
            expect!(subtotal(&items)).to_equal(2350)?;
        }

        "line item count" {
            expect!(items).to_have_length(3)?;
        }

        "with 10% discount" {
            setup {
                let discounted = apply_discount(subtotal(&items), 10);
            }

            "applies percentage" {
                expect!(discounted).to_equal(2115)?;
            }

            "discount is less than original" {
                expect!(discounted).to_be_less_than(subtotal(&items))?;
            }

            "with additional shipping" {
                setup {
                    let shipping = 500;
                }

                "adds flat shipping fee" {
                    expect!(discounted + shipping).to_equal(2615)?;
                }

                "shipping does not exceed order value" {
                    expect!(shipping).to_be_less_than(discounted)?;
                }

                "receipt line" {
                    let final_total = discounted + shipping;
                    let receipt = format!("total={final_total}");
                    expect!(receipt).to_contain_substr("2615")?;
                }
            }
        }

        "scenario can shadow setup bindings" {
            let total = subtotal(&items) + 1;
            expect!(total).to_equal(2351)?;
        }
    }
}

#[allow(clippy::missing_const_for_fn, dead_code)]
fn main() {}

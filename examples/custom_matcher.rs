//! Example showing how to define a reusable custom matcher.

use behave::prelude::*;

#[allow(dead_code)]
struct IsSortedAscending;

#[allow(clippy::unnecessary_literal_bound)]
impl BehaveMatch<Vec<i32>> for IsSortedAscending {
    fn matches(&self, actual: &Vec<i32>) -> bool {
        actual.windows(2).all(|window| window[0] <= window[1])
    }

    fn description(&self) -> &str {
        "to be sorted in ascending order"
    }
}

behave! {
    "custom matcher example" {
        "accepts reusable matcher types" {
            expect!(vec![1, 2, 3]).to_match(IsSortedAscending)?;
        }

        "works with negation too" {
            expect!(vec![3, 1, 2]).negate().to_match(IsSortedAscending)?;
        }
    }
}

#[allow(clippy::missing_const_for_fn, dead_code)]
fn main() {}

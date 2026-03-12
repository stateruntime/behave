#![allow(missing_docs)]

use behave::prelude::*;

behave! {
    "arithmetic" {
        "addition" {
            expect!(2 + 2).to_equal(4)?;
        }

        "subtraction" {
            expect!(10 - 3).to_equal(7)?;
        }
    }

    "booleans" {
        "true is true" {
            expect!(true).to_be_true()?;
        }

        "false is false" {
            expect!(false).to_be_false()?;
        }
    }

    "comparisons" {
        "greater than" {
            expect!(10).to_be_greater_than(5)?;
        }

        "less than" {
            expect!(3).to_be_less_than(10)?;
        }

        "at least equal" {
            expect!(5).to_be_at_least(5)?;
        }

        "at most equal" {
            expect!(5).to_be_at_most(5)?;
        }
    }

    "options" {
        "some value" {
            expect!(Some(42)).to_be_some()?;
        }

        "none value" {
            expect!(None::<i32>).to_be_none()?;
        }

        "some with value" {
            expect!(Some(42)).to_be_some_with(42)?;
        }
    }

    "results" {
        "ok value" {
            let val: Result<i32, &str> = Ok(1);
            expect!(val).to_be_ok()?;
        }

        "err value" {
            let val: Result<i32, &str> = Err("oops");
            expect!(val).to_be_err()?;
        }

        "ok with value" {
            let val: Result<i32, &str> = Ok(42);
            expect!(val).to_be_ok_with(42)?;
        }

        "err with value" {
            let val: Result<i32, &str> = Err("oops");
            expect!(val).to_be_err_with("oops")?;
        }
    }

    "collections" {
        "contains element" {
            expect!(vec![1, 2, 3]).to_contain(2)?;
        }

        "is empty" {
            let v: Vec<i32> = vec![];
            expect!(v).to_be_empty()?;
        }

        "has length" {
            expect!(vec![1, 2, 3]).to_have_length(3)?;
        }

        "not empty" {
            expect!(vec![1]).to_not_be_empty()?;
        }

        "contains all of" {
            expect!(vec![1, 2, 3]).to_contain_all_of(&[1, 3])?;
        }
    }

    "strings" {
        "starts with" {
            expect!("hello world").to_start_with("hello")?;
        }

        "ends with" {
            expect!("hello world").to_end_with("world")?;
        }

        "contains substring" {
            expect!("hello world").to_contain_substr("lo wo")?;
        }

        "has string length" {
            expect!("abc").to_have_str_length(3)?;
        }
    }

    "floats" {
        "approximately equal" {
            expect!(0.1_f64 + 0.2_f64).to_approximately_equal(0.3)?;
        }

        "approximately equal within" {
            expect!(1.005_f64).to_approximately_equal_within(1.0, 0.01)?;
        }
    }

    "negation" {
        "not equal" {
            expect!(1).negate().to_equal(2)?;
        }

        "not equal via not" {
            expect!(1).not().to_equal(2)?;
        }

        "not true" {
            expect!(false).negate().to_be_true()?;
        }

        "not greater than" {
            expect!(3).negate().to_be_greater_than(5)?;
        }

        "not some" {
            expect!(None::<i32>).negate().to_be_some()?;
        }

        "not ok" {
            let val: Result<i32, &str> = Err("e");
            expect!(val).negate().to_be_ok()?;
        }

        "not contain" {
            expect!(vec![1, 2, 3]).negate().to_contain(9)?;
        }

        "not start with" {
            expect!("hello").negate().to_start_with("xyz")?;
        }

        "not approximately equal" {
            expect!(1.0_f64).negate().to_approximately_equal(2.0)?;
        }

        "not equal direct" {
            expect!(1).to_not_equal(2)?;
        }

        "not have length" {
            expect!(vec![1, 2]).negate().to_have_length(5)?;
        }

        "not contain all of" {
            expect!(vec![1, 2]).negate().to_contain_all_of(&[8, 9])?;
        }
    }

    "predicate matcher" {
        "to satisfy passes" {
            expect!(42).to_satisfy(|x| x % 2 == 0, "to be even")?;
        }

        "to satisfy with negation" {
            expect!(7).negate().to_satisfy(|x| x % 2 == 0, "to be even")?;
        }
    }

    "slice matchers" {
        "slice contains" {
            let s: &[i32] = &[1, 2, 3];
            expect!(s).to_contain(2)?;
        }

        "slice is empty" {
            let s: &[i32] = &[];
            expect!(s).to_be_empty()?;
        }

        "slice has length" {
            let s: &[i32] = &[10, 20];
            expect!(s).to_have_length(2)?;
        }
    }

    "setup blocks" {
        setup {
            let base = 10;
        }

        "uses setup value" {
            expect!(base + 5).to_equal(15)?;
        }

        "also uses setup value" {
            expect!(base * 2).to_equal(20)?;
        }

        "nested setup" {
            setup {
                let extra = 5;
            }

            "inherits both setups" {
                expect!(base + extra).to_equal(15)?;
            }

            "deeply nested" {
                setup {
                    let deep = 1;
                }

                "inherits all three setups" {
                    expect!(base + extra + deep).to_equal(16)?;
                }
            }
        }

        "scenario body can shadow setup values" {
            let base = base + 1;
            expect!(base).to_equal(11)?;
        }

        "child setup can shadow parent setup" {
            setup {
                let _ = base;
                let base = 25;
            }

            "uses shadowed value" {
                expect!(base).to_equal(25)?;
            }
        }
    }

    "custom matchers" {
        "custom matcher passes" {
            struct IsEven;
            #[allow(clippy::unnecessary_literal_bound)]
            impl BehaveMatch<i32> for IsEven {
                fn matches(&self, actual: &i32) -> bool { actual % 2 == 0 }
                fn description(&self) -> &str { "to be even" }
            }
            expect!(4).to_match(IsEven)?;
        }
    }

    "teardown blocks" {
        "basic teardown runs after test" {
            teardown {
                // teardown code compiles and runs
                let _ = 1;
            }

            "test body executes" {
                expect!(1 + 1).to_equal(2)?;
            }
        }

        "teardown accesses setup variables" {
            setup {
                let resource = 42;
            }

            teardown {
                let _ = resource + 1;
            }

            "uses the resource" {
                expect!(resource).to_equal(42)?;
            }
        }

        "nested teardown inheritance" {
            setup {
                let outer = 10;
            }

            teardown {
                let _ = outer;
            }

            "inner group" {
                teardown {
                    let _ = outer + 1;
                }

                "sees both teardowns" {
                    expect!(outer).to_equal(10)?;
                }
            }
        }
    }

    "parameterized" {
        "addition" {
            each [
                (2, 2, 4),
                (0, 0, 0),
                (-1, 1, 0),
            ] |a, b, expected| {
                expect!(a + b).to_equal(expected)?;
            }
        }

        "single param" {
            each [1, 2, 3, 5, 8] |n| {
                expect!(n).to_be_greater_than(0)?;
            }
        }
    }

    "parameterized with setup" {
        setup {
            let base = 10;
        }

        "offset" {
            each [
                (1, 11),
                (5, 15),
            ] |n, expected| {
                expect!(base + n).to_equal(expected)?;
            }
        }
    }

    pending "todo test" {}

    "focus marker" {
        focus "focused test" {
            expect!(true).to_be_true()?;
        }
    }
}

#[cfg(feature = "color")]
mod color_tests {
    use behave::MatchError;

    #[test]
    fn color_single_line_contains_structural_content() {
        let err = MatchError::new("val".to_string(), "42".to_string(), "99".to_string(), false);
        let msg = err.to_string();
        assert!(msg.contains("expect!(val)"));
        assert!(msg.contains("99"));
        assert!(msg.contains("42"));
    }

    #[test]
    fn color_multiline_shows_diff_markers() {
        let err = MatchError::new(
            "text".to_string(),
            "alpha\nbeta\n".to_string(),
            "alpha\ngamma\n".to_string(),
            false,
        );
        let msg = err.to_string();
        assert!(msg.contains("--- actual"));
        assert!(msg.contains("+++ expected"));
        assert!(msg.contains("-gamma"));
        assert!(msg.contains("+beta"));
        assert!(msg.contains(" alpha"));
    }

    #[test]
    fn color_multiline_negated_uses_single_line_format() {
        let err = MatchError::new(
            "text".to_string(),
            "a\nb\n".to_string(),
            "a\nb\n".to_string(),
            true,
        );
        let msg = err.to_string();
        // Negated always uses single-line format, even for multiline values
        assert!(msg.contains("expected: not"));
        assert!(!msg.contains("---"));
    }
}

#[cfg(feature = "regex")]
mod regex_tests {
    use behave::prelude::*;

    behave! {
        "regex matchers" {
            "to_match_regex full match" {
                expect!("hello123").to_match_regex(r"hello\d+")?;
            }

            "to_match_regex rejects partial" {
                expect!("abc123def").negate().to_match_regex(r"\d+")?;
            }

            "to_contain_regex finds substring" {
                expect!("abc123def").to_contain_regex(r"\d+")?;
            }

            "to_contain_regex no match" {
                expect!("hello").negate().to_contain_regex(r"\d+")?;
            }
        }
    }
}

#[cfg(feature = "tokio")]
mod async_tests {
    use behave::prelude::*;

    behave! {
        "async support" {
            tokio;

            "basic async test" {
                let value = async { 42 }.await;
                expect!(value).to_equal(42)?;
            }

            "nested group inherits async" {
                "inner async test" {
                    let msg = async { "hello" }.await;
                    expect!(msg).to_equal("hello")?;
                }
            }

            "async with setup" {
                setup {
                    let base = 10;
                }

                "uses setup in async" {
                    let result = async { base + 5 }.await;
                    expect!(result).to_equal(15)?;
                }
            }

            "async with teardown" {
                setup {
                    let val = 99;
                }

                teardown {
                    let _ = val;
                }

                "teardown runs after async body" {
                    let result = async { val + 1 }.await;
                    expect!(result).to_equal(100)?;
                }
            }
        }
    }
}

#[cfg(feature = "std")]
mod panic_macros {
    use behave::prelude::*;

    behave! {
        "panic macros" {
            "expect panic catches panic" {
                expect_panic!({
                    let v: Vec<i32> = vec![];
                    let _ = v[0];
                })?;
            }

            "expect no panic succeeds normally" {
                expect_no_panic!({
                    let _ = 1 + 1;
                })?;
            }
        }
    }
}

#[cfg(feature = "std")]
mod combinator_tests {
    use behave::prelude::*;

    struct IsPositive;
    #[allow(clippy::unnecessary_literal_bound)]
    impl BehaveMatch<i32> for IsPositive {
        fn matches(&self, actual: &i32) -> bool {
            *actual > 0
        }
        fn description(&self) -> &str {
            "to be positive"
        }
    }

    struct IsEven;
    #[allow(clippy::unnecessary_literal_bound)]
    impl BehaveMatch<i32> for IsEven {
        fn matches(&self, actual: &i32) -> bool {
            actual % 2 == 0
        }
        fn description(&self) -> &str {
            "to be even"
        }
    }

    struct IsZero;
    #[allow(clippy::unnecessary_literal_bound)]
    impl BehaveMatch<i32> for IsZero {
        fn matches(&self, actual: &i32) -> bool {
            *actual == 0
        }
        fn description(&self) -> &str {
            "to be zero"
        }
    }

    behave! {
        "combinators" {
            "all_of passes when all match" {
                let m = all_of(vec![
                    Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
                    Box::new(IsEven),
                ]);
                expect!(4).to_match(m)?;
            }

            "all_of fails when one does not match" {
                let m = all_of(vec![
                    Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
                    Box::new(IsEven),
                ]);
                expect!(3).negate().to_match(m)?;
            }

            "any_of passes when one matches" {
                let m = any_of(vec![
                    Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
                    Box::new(IsPositive),
                ]);
                expect!(5).to_match(m)?;
            }

            "any_of fails when none match" {
                let m = any_of(vec![
                    Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
                    Box::new(IsEven),
                ]);
                expect!(3).negate().to_match(m)?;
            }

            "not_matching inverts" {
                let m = not_matching(Box::new(IsEven));
                expect!(7).to_match(m)?;
            }

            "nested composition" {
                let inner = all_of(vec![
                    Box::new(IsPositive) as Box<dyn BehaveMatch<i32>>,
                    Box::new(IsEven),
                ]);
                let m = any_of(vec![
                    Box::new(IsZero) as Box<dyn BehaveMatch<i32>>,
                    Box::new(inner),
                ]);
                expect!(4).to_match(m)?;
            }
        }
    }
}

#[cfg(feature = "std")]
mod hashmap_tests {
    use behave::prelude::*;
    use std::collections::{BTreeMap, HashMap};

    behave! {
        "hashmap matchers" {
            "contains key" {
                let mut m = HashMap::new();
                m.insert("x", 1);
                expect!(m).to_contain_key(&"x")?;
            }

            "contains value" {
                let mut m = HashMap::new();
                m.insert("x", 42);
                expect!(m).to_contain_value(&42)?;
            }

            "contains entry" {
                let mut m = HashMap::new();
                m.insert("x", 42);
                expect!(m).to_contain_entry(&"x", &42)?;
            }

            "is empty" {
                let m: HashMap<&str, i32> = HashMap::new();
                expect!(m).to_be_empty()?;
            }

            "not empty" {
                let mut m = HashMap::new();
                m.insert("a", 1);
                expect!(m).to_not_be_empty()?;
            }

            "has length" {
                let mut m = HashMap::new();
                m.insert("a", 1);
                m.insert("b", 2);
                expect!(m).to_have_length(2)?;
            }
        }

        "btreemap matchers" {
            "contains key" {
                let mut m = BTreeMap::new();
                m.insert("x", 1);
                expect!(m).to_contain_key(&"x")?;
            }

            "contains value" {
                let mut m = BTreeMap::new();
                m.insert("x", 42);
                expect!(m).to_contain_value(&42)?;
            }

            "contains entry" {
                let mut m = BTreeMap::new();
                m.insert("x", 42);
                expect!(m).to_contain_entry(&"x", &42)?;
            }

            "is empty" {
                let m: BTreeMap<&str, i32> = BTreeMap::new();
                expect!(m).to_be_empty()?;
            }

            "has length" {
                let mut m = BTreeMap::new();
                m.insert("a", 1);
                m.insert("b", 2);
                expect!(m).to_have_length(2)?;
            }
        }
    }
}

#[cfg(feature = "std")]
mod soft_assertion_tests {
    use behave::prelude::*;

    behave! {
        "soft assertions" {
            "reports success when all checks pass" {
                let mut errors = SoftErrors::new();
                errors.check(expect!(1).to_equal(1));
                errors.check(expect!(true).to_be_true());
                errors.check(expect!("hello").to_start_with("he"));
                errors.finish()?;
            }

            "collects only the failures, ignores successes" {
                let mut errors = SoftErrors::new();
                errors.check(expect!(1).to_equal(1));   // pass
                errors.check(expect!(2).to_equal(99));  // fail
                errors.check(expect!(3).to_equal(3));   // pass
                errors.check(expect!(4).to_equal(88));  // fail
                expect!(errors.len()).to_equal(2)?;
            }

            "can mix hard and soft assertions in same test" {
                // Hard assertion first — fails fast if wrong
                expect!(true).to_be_true()?;

                // Soft assertions in the middle
                let mut errors = SoftErrors::new();
                errors.check(expect!(1).to_equal(1));
                errors.finish()?;

                // Hard assertion after — verifies final state
                expect!(42).to_equal(42)?;
            }

            "finish succeeds on empty collector" {
                let errors = SoftErrors::new();
                errors.finish()?;
            }
        }
    }
}

#[cfg(feature = "std")]
mod timeout_tests {
    use behave::prelude::*;

    behave! {
        "timeout" {
            "basic sync timeout" {
                timeout 5000;

                "passes when test completes within deadline" {
                    expect!(1 + 1).to_equal(2)?;
                }
            }

            "timeout works with setup" {
                timeout 5000;

                setup {
                    let base = 10;
                }

                "setup variables are accessible" {
                    expect!(base).to_equal(10)?;
                }
            }

            "timeout works with setup and teardown" {
                timeout 5000;

                setup {
                    let val = 42;
                }

                teardown {
                    let _ = val;
                }

                "teardown can access setup variables" {
                    expect!(val).to_equal(42)?;
                }
            }

            "timeout inherits to child groups" {
                timeout 5000;

                "inner group without timeout declaration" {
                    "still enforces parent timeout" {
                        expect!(true).to_be_true()?;
                    }
                }
            }

            "inner timeout overrides outer timeout" {
                timeout 10000;

                "stricter inner group" {
                    timeout 5000;

                    "uses the inner 5s timeout" {
                        expect!(true).to_be_true()?;
                    }
                }
            }
        }
    }
}

#[cfg(feature = "tokio")]
mod async_timeout_tests {
    use behave::prelude::*;

    behave! {
        "async timeout" {
            tokio;
            timeout 5000;

            "passes when async test completes within deadline" {
                let val = async { 42 }.await;
                expect!(val).to_equal(42)?;
            }

            "async timeout with setup and teardown" {
                setup {
                    let val = 99;
                }

                teardown {
                    let _ = val;
                }

                "teardown runs after async body" {
                    let result = async { val + 1 }.await;
                    expect!(result).to_equal(100)?;
                }
            }
        }
    }
}

mod common;

/// Demonstrates importing shared helpers from `tests/common/mod.rs`.
///
/// The `mod common;` import brings in shared types, matchers, and helpers.
/// Inside the `behave!` block, each group generates `use super::*;` so
/// the common items propagate through all nesting levels.
mod shared_imports {
    use crate::common::{double, IsOrigin, Point};
    use behave::prelude::*;

    behave! {
        "shared imports" {
            "uses helper from common" {
                expect!(double(21)).to_equal(42)?;
            }

            "uses shared type from common" {
                let p = Point::new(3, 4);
                expect!(p).to_equal(Point::new(3, 4))?;
            }

            "uses custom matcher from common" {
                let origin = Point::origin();
                expect!(origin).to_match(IsOrigin)?;
            }

            "nested group inherits imports" {
                "still has access to shared helpers" {
                    expect!(double(5)).to_equal(10)?;
                }
            }
        }
    }
}

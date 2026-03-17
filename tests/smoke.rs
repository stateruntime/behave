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

    "named parameterized" {
        "named tuples" {
            each [
                ("two plus two", 2, 2, 4),
                ("zero plus zero", 0, 0, 0),
                ("negative plus positive", -1, 1, 0),
            ] |a, b, expected| {
                expect!(a + b).to_equal(expected)?;
            }
        }

        "named single values" {
            each [
                ("the answer", 42),
                ("lucky number", 7),
            ] |n| {
                expect!(n).to_be_greater_than(0)?;
            }
        }

        "keyword labels become raw idents" {
            each [
                ("type", 1),
                ("match", 2),
                ("fn", 3),
            ] |n| {
                expect!(n).to_be_greater_than(0)?;
            }
        }

        "special character labels are slugified" {
            each [
                ("hello world!", 1),
                ("test-case-123", 2),
                ("100% coverage", 3),
            ] |n| {
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

    "xfail tests" {
        xfail "expected failure passes" {
            expect!(1).to_equal(2)?;
        }

        xfail "assertion error caught" {
            expect!("hello").to_start_with("xyz")?;
        }
    }

    "xfail with setup" {
        setup {
            let val = 42;
        }

        xfail "setup variable accessible" {
            expect!(val).to_equal(999)?;
        }
    }

    "matrix tests" {
        "two dimensions" {
            matrix [1, 2, 3] x ["a", "b"] |n, s| {
                let result = format!("{n}{s}");
                expect!(result.len()).to_be_greater_than(1)?;
            }
        }

        "matrix with setup" {
            setup {
                let prefix = "item";
            }

            "formatted" {
                matrix [1, 2] x [10, 20] |a, b| {
                    let label = format!("{prefix}_{a}_{b}");
                    expect!(label).to_start_with("item")?;
                }
            }
        }

        "matrix with teardown" {
            setup {
                let tracker = std::cell::Cell::new(0);
            }

            teardown {
                let _ = tracker.get();
            }

            "runs teardown after each combo" {
                matrix [1, 2] x [10, 20] |a, b| {
                    tracker.set(a + b);
                    expect!(tracker.get()).to_be_greater_than(0)?;
                }
            }
        }

        "matrix with timeout" {
            timeout 5000;

            "completes within deadline" {
                matrix [1, 2] x [10, 20] |a, b| {
                    expect!(a + b).to_be_greater_than(0)?;
                }
            }
        }
    }

    "xfail with teardown" {
        setup {
            let resource = 42;
        }

        teardown {
            let _ = resource;
        }

        xfail "teardown runs after xfail catches error" {
            expect!(resource).to_equal(999)?;
        }
    }

    "xfail with timeout" {
        timeout 5000;

        xfail "fails within timeout" {
            expect!(1).to_equal(2)?;
        }
    }

    xfail "matrix with xfail" {
        matrix [1, 2] x [10, 20] |a, b| {
            expect!(a + b).to_equal(0)?;
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

            xfail "async xfail catches error" {
                let val = async { 1 }.await;
                expect!(val).to_equal(999)?;
            }

            "async matrix" {
                matrix [1, 2] x [10, 20] |a, b| {
                    let sum = async { a + b }.await;
                    expect!(sum).to_be_greater_than(0)?;
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

mod tag_tests {
    use behave::prelude::*;

    behave! {
        "tagged suite" tag "integration" {
            "test inside tagged group" {
                expect!(true).to_be_true()?;
            }
        }

        "tagged test" tag "slow", "nightly" {
            expect!(1 + 1).to_equal(2)?;
        }

        "tagged each" tag "unit" {
            each [1, 2] |n| {
                expect!(n).to_be_greater_than(0)?;
            }
        }

        "tagged matrix" tag "slow" {
            matrix [1, 2] x [10, 20] |a, b| {
                expect!(a + b).to_be_greater_than(0)?;
            }
        }

        "focus with tag" {
            focus "focused and tagged" tag "critical" {
                expect!(42).to_equal(42)?;
            }
        }
    }
}

#[cfg(feature = "std")]
mod skip_when_tests {
    use behave::prelude::*;

    behave! {
        "skip when" {
            "skips when condition is true" {
                skip_when!(true, "always skip");
                expect!(false).to_be_true()?;
            }

            "continues when condition is false" {
                skip_when!(false, "never skip");
                expect!(true).to_be_true()?;
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

// --- v0.9.0 matchers ---

mod v090_matchers {
    use behave::prelude::*;

    behave! {
        "v0.9.0 matchers" {
            "range matcher" {
                expect!(5).to_be_between(1, 10)?;
                expect!(1).to_be_between(1, 10)?;
                expect!(10).to_be_between(1, 10)?;
            }

            "case insensitive string" {
                expect!("Hello World").to_equal_ignoring_case("hello world")?;
            }

            "option predicate" {
                expect!(Some(42)).to_be_some_and(|v| *v > 0, "to be positive")?;
            }

            "result predicates" {
                let ok_val: Result<i32, &str> = Ok(42);
                expect!(ok_val).to_be_ok_and(|v| *v > 0, "to be positive")?;

                let err_val: Result<i32, String> = Err("timeout".to_string());
                expect!(err_val)
                    .to_be_err_and(|e| e.contains("timeout"), "to contain timeout")?;
            }

            "collection predicates" {
                expect!(vec![2, 4, 6]).to_all_satisfy(|x| x % 2 == 0, "to be even")?;
                expect!(vec![1, 2, 3]).to_any_satisfy(|x| x % 2 == 0, "to be even")?;
                expect!(vec![1, 3, 5]).to_none_satisfy(|x| x % 2 == 0, "to be even")?;
                expect!(vec![1, 2, 3]).to_contain_any_of(&[9, 2])?;
            }

            "sorted by key" {
                expect!(vec!["a", "bb", "ccc"])
                    .to_be_sorted_by_key(|s| s.len(), "by length")?;
            }

            "display matchers" {
                expect!(42).to_display_as("42")?;
                expect!(42).to_display_containing("4")?;
                expect!(vec![1, 2]).to_debug_containing("[1, 2]")?;
            }
        }
    }
}

#[cfg(feature = "std")]
mod v090_std_matchers {
    use behave::prelude::*;
    use std::time::Duration;

    behave! {
        "v0.9.0 std matchers" {
            "duration matchers" {
                expect!(Duration::from_millis(500))
                    .to_be_shorter_than(Duration::from_secs(1))?;
                expect!(Duration::from_secs(2))
                    .to_be_longer_than(Duration::from_secs(1))?;
                expect!(Duration::from_millis(1050))
                    .to_be_close_to_duration(
                        Duration::from_secs(1),
                        Duration::from_millis(100),
                    )?;
            }

            "error chain matchers" {
                use std::io;
                use std::fmt;

                #[derive(Debug)]
                struct Wrapper(io::Error);

                impl fmt::Display for Wrapper {
                    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                        write!(f, "wrapper: {}", self.0)
                    }
                }

                impl std::error::Error for Wrapper {
                    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                        Some(&self.0)
                    }
                }

                let err = Wrapper(io::Error::other("connection timeout"));
                expect!(err).to_have_source()?;
            }

            "path matchers" {
                use std::path::PathBuf;

                let cargo = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
                expect!(cargo.clone()).to_exist()?;
                expect!(cargo.clone()).to_be_a_file()?;
                expect!(cargo).to_have_extension("toml")?;

                let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
                expect!(src).to_be_a_directory()?;
            }
        }
    }
}

#[cfg(feature = "std")]
mod v090_expect_match {
    use behave::prelude::*;

    #[derive(Debug)]
    enum Status {
        Active,
        #[allow(dead_code)]
        Inactive,
    }

    behave! {
        "expect_match macro" {
            "matches enum variant" {
                expect_match!(Status::Active, Status::Active)?;
            }

            "matches with guard" {
                expect_match!(Some(42), Some(v) if *v > 0)?;
            }
        }
    }
}

#[cfg(feature = "json")]
mod v090_json_matchers {
    use behave::prelude::*;

    behave! {
        "json matchers" {
            "has field" {
                let val: serde_json::Value = serde_json::json!({"name": "Alice", "age": 30});
                expect!(val.clone()).to_have_field("name")?;
                expect!(val).to_have_field_value("age", &serde_json::json!(30))?;
            }

            "json superset" {
                let actual: serde_json::Value =
                    serde_json::json!({"a": 1, "b": 2, "c": 3});
                let expected: serde_json::Value = serde_json::json!({"a": 1, "b": 2});
                expect!(actual).to_be_json_superset_of(&expected)?;
            }
        }
    }
}

#[cfg(feature = "http")]
mod v090_http_matchers {
    use behave::prelude::*;

    behave! {
        "http matchers" {
            "status codes" {
                expect!(http::StatusCode::OK).to_be_success()?;
                expect!(http::StatusCode::MOVED_PERMANENTLY).to_be_redirect()?;
                expect!(http::StatusCode::NOT_FOUND).to_be_client_error()?;
                expect!(http::StatusCode::INTERNAL_SERVER_ERROR).to_be_server_error()?;
                expect!(http::StatusCode::OK).to_have_status_code(200)?;
            }

            "headers" {
                let mut headers = http::HeaderMap::new();
                headers.insert(
                    http::header::CONTENT_TYPE,
                    http::HeaderValue::from_static("application/json"),
                );
                expect!(headers.clone()).to_have_header("content-type")?;
                expect!(headers).to_have_header_value("content-type", "application/json")?;
            }
        }
    }
}

// --- Negative test cases (Fix 31) ---
// These tests intentionally trigger failures and verify the error messages.

mod negative_tests {
    use behave::prelude::*;

    #[test]
    fn to_equal_failure_shows_actual_and_expected() {
        let result = expect!(42).to_equal(99);
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("42"), "error should contain actual value");
            assert!(msg.contains("99"), "error should contain expected value");
        }
    }

    #[test]
    fn to_be_true_failure_shows_false() {
        let result = expect!(false).to_be_true();
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("false"), "error should show actual: false");
        }
    }

    #[test]
    fn to_be_false_failure_shows_true() {
        let result = expect!(true).to_be_false();
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("true"), "error should show actual: true");
        }
    }

    #[test]
    fn to_contain_failure_shows_element() {
        let result = expect!(vec![1, 2, 3]).to_contain(9);
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains('9'), "error should show missing element");
        }
    }

    #[test]
    fn to_be_some_failure_on_none() {
        let result = expect!(None::<i32>).to_be_some();
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("None"), "error should show actual: None");
        }
    }

    #[test]
    fn to_be_none_failure_on_some() {
        let result = expect!(Some(42)).to_be_none();
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("42"), "error should show the Some value");
        }
    }

    #[test]
    fn negated_to_equal_failure_shows_not() {
        let result = expect!(42).not().to_equal(42);
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("not"), "negated error should contain 'not'");
        }
    }

    #[test]
    fn to_start_with_failure_shows_strings() {
        let result = expect!("hello").to_start_with("xyz");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("hello"), "error should show actual string");
            assert!(msg.contains("xyz"), "error should show expected prefix");
        }
    }

    #[test]
    fn to_be_greater_than_failure() {
        let result = expect!(3).to_be_greater_than(10);
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains('3'), "error should show actual value");
            assert!(msg.contains("10"), "error should show threshold");
        }
    }

    #[test]
    fn to_have_length_failure_shows_lengths() {
        let result = expect!(vec![1, 2]).to_have_length(5);
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains('5'), "error should show expected length");
        }
    }

    #[test]
    fn to_be_empty_failure_on_nonempty() {
        let result = expect!(vec![1]).to_be_empty();
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("[1]"), "error should show actual collection");
        }
    }

    #[test]
    fn to_be_ok_failure_on_err() {
        let val: Result<i32, &str> = Err("oops");
        let result = expect!(val).to_be_ok();
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("oops"), "error should show the Err value");
        }
    }
}

#[cfg(feature = "url")]
mod v090_url_matchers {
    use behave::prelude::*;

    /// Parses a URL from a string literal, aborting on invalid input.
    ///
    /// All URLs in these tests are known-valid literals, so the error branch
    /// is unreachable in practice.
    fn parse_url(s: &str) -> url::Url {
        url::Url::parse(s).unwrap_or_else(|_| std::process::abort())
    }

    behave! {
        "url matchers" {
            "url components" {
                let url = parse_url("https://example.com/path?key=val#frag");
                expect!(url.clone()).to_have_scheme("https")?;
                expect!(url.clone()).to_have_host("example.com")?;
                expect!(url.clone()).to_have_path("/path")?;
                expect!(url.clone()).to_have_query_param("key")?;
                expect!(url.clone()).to_have_query_param_value("key", "val")?;
                expect!(url).to_have_fragment("frag")?;
            }
        }
    }

    // --- URL negative tests (Fix 33) ---

    #[test]
    fn url_wrong_scheme_error() {
        let url = parse_url("https://example.com");
        let result = expect!(url).to_have_scheme("ftp");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("ftp"), "error should show expected scheme");
        }
    }

    #[test]
    fn url_wrong_host_error() {
        let url = parse_url("https://example.com");
        let result = expect!(url).to_have_host("other.com");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("other.com"), "error should show expected host");
        }
    }

    #[test]
    fn url_wrong_path_error() {
        let url = parse_url("https://example.com/actual");
        let result = expect!(url).to_have_path("/expected");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("/expected"), "error should show expected path");
        }
    }

    #[test]
    fn url_missing_query_param_error() {
        let url = parse_url("https://example.com");
        let result = expect!(url).to_have_query_param("missing");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(msg.contains("missing"), "error should show expected param");
        }
    }

    #[test]
    fn url_wrong_fragment_error() {
        let url = parse_url("https://example.com#actual");
        let result = expect!(url).to_have_fragment("expected");
        assert!(result.is_err());
        if let Err(err) = result {
            let msg = err.to_string();
            assert!(
                msg.contains("expected"),
                "error should show expected fragment"
            );
        }
    }
}

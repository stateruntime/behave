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

    pending "todo test" {}

    "focus marker" {
        focus "focused test" {
            expect!(true).to_be_true()?;
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

# `to_approximately_equal`

Asserts two floats are approximately equal within a default epsilon.

- For `f64`, the default epsilon is `1e-10`.
- For `f32`, the default epsilon is `1e-6`.

If you need a custom tolerance, use [`to_approximately_equal_within`](to_approximately_equal_within.md).

## Example

```rust
use behave::prelude::*;

behave! {
    "to_approximately_equal" {
        "example" {
            expect!(0.1_f64 + 0.2_f64).to_approximately_equal(0.3_f64)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_approximately_equal" {
        "edge cases" {
            // `NaN` never compares as approximately equal (even to itself).
            assert!(expect!(f64::NAN).to_approximately_equal(f64::NAN).is_err());

            // `INFINITY - INFINITY` is `NaN`, so this also fails.
            assert!(expect!(f64::INFINITY).to_approximately_equal(f64::INFINITY).is_err());

            // `-0.0` and `0.0` compare equal (diff is 0).
            assert!(expect!(-0.0_f64).to_approximately_equal(0.0_f64).is_ok());
        }
    }
}
```

## See also

- [`to_approximately_equal_within`](to_approximately_equal_within.md)
- [All matchers](README.md)

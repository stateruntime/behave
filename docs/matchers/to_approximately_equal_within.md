# `to_approximately_equal_within`

Asserts two floats are approximately equal within a custom epsilon.

## Example

```rust
use behave::prelude::*;

behave! {
    "to_approximately_equal_within" {
        "example" {
            expect!(1.005_f64).to_approximately_equal_within(1.0_f64, 0.01_f64)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_approximately_equal_within" {
        "edge cases" {
            // With epsilon = 0, this becomes an exact comparison (diff must be 0).
            assert!(expect!(1.0_f64)
                .to_approximately_equal_within(1.0_f64, 0.0_f64)
                .is_ok());
            assert!(expect!(1.0_f64)
                .to_approximately_equal_within(1.000_000_1_f64, 0.0_f64)
                .is_err());

            // Negative epsilons make the check impossible (diff is always >= 0).
            assert!(expect!(1.0_f64)
                .to_approximately_equal_within(1.0_f64, -1.0_f64)
                .is_err());
        }
    }
}
```

## See also

- [`to_approximately_equal`](to_approximately_equal.md)
- [All matchers](README.md)

# `to_contain_all_of`

Asserts a collection contains *every* element in the provided slice.

If the provided slice is empty, this matcher returns `Ok(())` (vacuous truth).

## Example

```rust
use behave::prelude::*;

behave! {
    "to_contain_all_of" {
        "example" {
            expect!(vec![1, 2, 3]).to_contain_all_of(&[1, 3])?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_contain_all_of" {
        "edge cases" {
            // Vacuous truth: empty list always passes.
            assert!(expect!(vec![1, 2, 3])
                .to_contain_all_of(&[] as &[i32])
                .is_ok());

            // Missing any element fails.
            assert!(expect!(vec![1, 2, 3]).to_contain_all_of(&[1, 9]).is_err());
        }
    }
}
```

## See also

- [`to_contain`](to_contain.md)
- [`to_have_length`](to_have_length.md)
- [All matchers](README.md)

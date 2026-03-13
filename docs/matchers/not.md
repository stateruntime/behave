# `.not()` / `.negate()`

Negation flips the pass/fail logic for the next matcher in the chain.

- Use `.not()` when it reads better in a sentence.
- Use `.negate()` if you prefer explicitness (it’s the same thing).

```rust
use behave::prelude::*;

behave! {
    "not" {
        "example" {
            expect!(42).not().to_equal(0)?;
            expect!(vec![1, 2, 3]).negate().to_contain(9)?;
        }
    }
}
```

If a dedicated negative matcher exists (like [`to_not_equal`](to_not_equal.md) or
[`to_not_be_empty`](to_not_be_empty.md)), prefer whichever reads most clearly.

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "not" {
        "edge cases" {
            // Negation is a toggle: calling it twice returns to the original state.
            assert!(expect!(1).not().not().to_equal(1).is_ok());
        }
    }
}
```

## See also

- [`to_not_equal`](to_not_equal.md)
- [`to_not_be_empty`](to_not_be_empty.md)
- [All matchers](README.md)

# `to_contain`

Asserts a collection contains an element (uses `PartialEq`).

Supported by built-in collection matchers (like `Vec<T>` and `&[T]`).

## Example

```rust
use behave::prelude::*;

behave! {
    "to_contain" {
        "example" {
            expect!(vec![1, 2, 3]).to_contain(2)?;
            expect!(&[1, 2, 3][..]).not().to_contain(9)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_contain" {
        "edge cases" {
            assert!(expect!(Vec::<i32>::new()).to_contain(1).is_err());
            assert!(expect!(&[1, 2, 3][..]).to_contain(4).is_err());
        }
    }
}
```

## See also

- [`to_contain_all_of`](to_contain_all_of.md)
- [Negation (`.not()` / `.negate()`)](not.md)
- [All matchers](README.md)

# `to_not_be_empty`

Asserts a collection has at least one element.

This matcher is implemented for:

- `Vec<T>` and `&[T]`
- `HashMap<K, V>` and `BTreeMap<K, V>` *(requires feature `std`, enabled by default)*

## Example

```rust
use behave::prelude::*;

behave! {
    "to_not_be_empty" {
        "example" {
            expect!(vec![1]).to_not_be_empty()?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_not_be_empty" {
        "edge cases" {
            assert!(expect!(Vec::<i32>::new()).to_not_be_empty().is_err());
            assert!(expect!(&[] as &[i32]).to_not_be_empty().is_err());
        }
    }
}
```

## See also

- [`to_be_empty`](to_be_empty.md)
- [Negation (`.not()` / `.negate()`)](not.md)
- [All matchers](README.md)

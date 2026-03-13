# `to_be_empty`

Asserts a collection has no elements.

This matcher is implemented for:

- `Vec<T>` and `&[T]`
- `HashMap<K, V>` and `BTreeMap<K, V>` *(requires feature `std`, enabled by default)*

## Example

```rust
use behave::prelude::*;

behave! {
    "to_be_empty" {
        "example" {
            expect!(Vec::<i32>::new()).to_be_empty()?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_be_empty" {
        "edge cases" {
            assert!(expect!(vec![1]).to_be_empty().is_err());
            assert!(expect!(&[1][..]).to_be_empty().is_err());
        }
    }
}
```

## See also

- [`to_not_be_empty`](to_not_be_empty.md)
- [`to_have_length`](to_have_length.md)
- [All matchers](README.md)

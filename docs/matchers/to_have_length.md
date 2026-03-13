# `to_have_length`

Asserts a collection has exactly `n` elements.

This matcher is implemented for:

- `Vec<T>` and `&[T]`
- `HashMap<K, V>` and `BTreeMap<K, V>` *(requires feature `std`, enabled by default)*

## Example

```rust
use behave::prelude::*;

behave! {
    "to_have_length" {
        "example" {
            expect!(vec![1, 2, 3]).to_have_length(3)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;

behave! {
    "to_have_length" {
        "edge cases" {
            assert!(expect!(vec![1, 2, 3]).to_have_length(2).is_err());

            // Works on slices too.
            assert!(expect!(&[1, 2, 3][..]).to_have_length(3).is_ok());
        }
    }
}
```

## See also

- [`to_be_empty`](to_be_empty.md)
- [`to_contain`](to_contain.md)
- [All matchers](README.md)

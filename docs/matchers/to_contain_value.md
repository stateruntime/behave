# `to_contain_value`

Asserts a `HashMap`/`BTreeMap` contains a value.

Requires feature `std` (enabled by default).

## Example

```rust
use behave::prelude::*;
use std::collections::HashMap;

behave! {
    "to_contain_value" {
        "example" {
            let mut m = HashMap::new();
            m.insert("a", 1);
            expect!(m).to_contain_value(&1)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;
use std::collections::HashMap;

behave! {
    "to_contain_value" {
        "edge cases" {
            let mut m = HashMap::new();
            m.insert("a", 1);
            assert!(expect!(m).to_contain_value(&2).is_err());
        }
    }
}
```

## See also

- [`to_contain_key`](to_contain_key.md)
- [`to_contain_entry`](to_contain_entry.md)
- [All matchers](README.md)

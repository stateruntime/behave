# `to_contain_entry`

Asserts a `HashMap`/`BTreeMap` contains a specific key-value pair.

Requires feature `std` (enabled by default).

## Example

```rust
use behave::prelude::*;
use std::collections::HashMap;

behave! {
    "to_contain_entry" {
        "example" {
            let mut m = HashMap::new();
            m.insert("a", 1);
            expect!(m).to_contain_entry(&"a", &1)?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;
use std::collections::HashMap;

behave! {
    "to_contain_entry" {
        "edge cases" {
            let mut m = HashMap::new();
            m.insert("a", 1);

            assert!(expect!(m.clone()).to_contain_entry(&"a", &2).is_err());
            assert!(expect!(m).to_contain_entry(&"b", &1).is_err());
        }
    }
}
```

## See also

- [`to_contain_key`](to_contain_key.md)
- [`to_contain_value`](to_contain_value.md)
- [All matchers](README.md)

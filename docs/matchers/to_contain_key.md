# `to_contain_key`

Asserts a `HashMap`/`BTreeMap` contains a key.

Requires feature `std` (enabled by default).

## Example

```rust
use behave::prelude::*;
use std::collections::HashMap;

behave! {
    "to_contain_key" {
        "example" {
            let mut m = HashMap::new();
            m.insert("a", 1);
            expect!(m).to_contain_key(&"a")?;
        }
    }
}
```

## Edge cases

```rust
use behave::prelude::*;
use std::collections::HashMap;

behave! {
    "to_contain_key" {
        "edge cases" {
            let mut m = HashMap::new();
            m.insert("a", 1);
            assert!(expect!(m).to_contain_key(&"b").is_err());
        }
    }
}
```

## See also

- [`to_contain_value`](to_contain_value.md)
- [`to_contain_entry`](to_contain_entry.md)
- [All matchers](README.md)

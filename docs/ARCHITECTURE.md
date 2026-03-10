# Architecture

High-level overview of the crate's design and structure.

## Overview

<!-- Describe the purpose of the crate and the problem it solves. -->

`crate-name` is a Rust library that ...

## Design Principles

1. **Dependencies point inward.** Core types have zero external deps. Adapters depend on core.
2. **Parse, don't validate.** Use newtypes for validated data. Validate at boundaries, use types internally.
3. **Modular by default.** Optional functionality behind feature flags. Core is minimal.
4. **No hidden state.** All inputs are parameters, all outputs are return values.

## Module Map

```
src/
  lib.rs      # Public API surface. Re-exports key types and functions.
  error.rs    # Crate error types.
```

<!-- As the crate grows, document modules here:
  config.rs   # Configuration types and parsing
  parser.rs   # Input parsing logic
  types.rs    # Domain newtypes and value objects
-->

## Public API Surface

All public types are re-exported from `lib.rs`. The public API is intentionally small — expand it deliberately.

## Key Design Decisions

<!-- Document important architectural choices and their rationale. -->

1. **`#[non_exhaustive]` on all public enums/structs** — allows adding fields/variants in minor versions.
2. **Manual error impls** — typed errors with `Display` + `std::error::Error`, no macro dependencies.
3. **`unsafe` forbidden** — correctness over performance. Can be revisited per-module if profiling demands it.

## Dependencies

| Crate | Purpose |
|-------|---------|
| *(none)* | Zero dependencies by default |

## Feature Flags

| Feature | Default | Purpose |
|---------|---------|---------|
| `std`   | Yes     | Standard library support |

## Future Considerations

<!-- Note anticipated changes or known limitations. -->

# `skip_when!` Macro

Conditionally skips a test at runtime. When the condition is true, the test
returns early and is reported as skipped. When false, the test continues
normally.

**Requires:** `std` feature (enabled by default).

## Syntax

```rust,ignore
skip_when!(condition, "reason");
```

- **condition** — any expression that evaluates to `bool`
- **reason** — a string describing why the test was skipped

## When to Use

Use `skip_when!` for tests that should only run under certain conditions:

- **Environment requirements** — database URLs, API keys, CI variables
- **Platform checks** — OS-specific tests, architecture requirements
- **External dependencies** — services that may not be running locally
- **Feature toggles** — tests for optional runtime capabilities

Unlike `pending`, which permanently ignores a test, `skip_when!` makes a
runtime decision so the test runs when conditions are met.

## Basic Usage

```rust
use behave::prelude::*;

fn demo() -> Result<(), MatchError> {
    skip_when!(cfg!(windows), "only runs on unix");
    expect!(1 + 1).to_equal(2)?;
    Ok(())
}

assert!(demo().is_ok());
```

## Inside `behave!`

```rust,ignore
use behave::prelude::*;

behave! {
    "integration tests" {
        "requires redis" {
            skip_when!(
                std::env::var("REDIS_URL").is_err(),
                "REDIS_URL not set"
            );
            // ... test code that needs Redis ...
            expect!(true).to_be_true()?;
        }

        "requires CI" {
            skip_when!(
                std::env::var("CI").is_err(),
                "only runs in CI environment"
            );
            expect!(true).to_be_true()?;
        }
    }
}
```

## With Parameterized Tests

`skip_when!` works inside `each` and `matrix` blocks:

```rust,ignore
behave! {
    "platform matrix" {
        each [
            ("posix feature", cfg!(unix)),
            ("windows feature", cfg!(windows)),
        ] |available| {
            skip_when!(!available, "platform not supported");
            expect!(true).to_be_true()?;
        }
    }
}
```

## How It Works

### The Skip Sentinel Protocol

`skip_when!` expands to:

```rust,ignore
if condition {
    println!("BEHAVE_SKIP: reason");
    return Ok(());
}
```

**Without `cargo-behave`:** The test prints the sentinel and returns `Ok(())`.
`cargo test` reports it as passed. The skip is silent but harmless.

**With `cargo-behave`:** The CLI uses `--show-output` to capture stdout from
passing tests. When it detects `BEHAVE_SKIP: <reason>` in a test's output, it
reclassifies that test from `Pass` to `Skipped`. The reason is preserved.

### CLI Output

In tree output, skipped tests show as:

```text
└─ integration tests
   ├─ ⊘ requires redis (skipped: REDIS_URL not set)
   └─ ✓ local test
```

The `⊘` symbol in cyan indicates a skipped test. The skip reason is visible in
the output.

### Machine-Readable Output

**JSON:**

```json
{
  "full_name": "integration_tests::requires_redis",
  "outcome": "Skipped",
  "skip_reason": "REDIS_URL not set"
}
```

**JUnit XML:**

```xml
<testcase classname="integration_tests" name="requires_redis">
  <skipped message="skipped: REDIS_URL not set" />
</testcase>
```

### Summary Line

The summary includes a skipped count:

```text
5 passed, 2 skipped, 1 ignored
```

## Comparison with Other Skip Mechanisms

| Mechanism | When evaluated | Visibility | Runs test body |
|-----------|---------------|------------|----------------|
| `pending` | Compile time | Always ignored | No |
| `#[ignore]` | Compile time | Ignored by `cargo test` | No |
| `skip_when!` | Runtime | Skipped (with reason) | Only if condition is false |

## Common Patterns

### Environment variable check

```rust,ignore
skip_when!(
    std::env::var("DATABASE_URL").is_err(),
    "DATABASE_URL not configured"
);
```

### Feature detection

```rust,ignore
skip_when!(
    !has_avx2_support(),
    "AVX2 instructions not available"
);
```

### Conditional on build configuration

```rust,ignore
skip_when!(cfg!(debug_assertions), "only runs in release mode");
```

## See Also

- [`behave!`](behave.md) for the test suite macro and `pending` keyword
- [`expect!`](expect.md) for value-based assertions
- [CLI Guide](../CLI.md) for `--tag` and `--exclude-tag` filtering

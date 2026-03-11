# CLI Workspace Fixture

This fixture is a small multi-package workspace used to exercise
`cargo-behave` end to end.

It exists for two reasons:

- to give maintainers a concrete example of workspace-aware CLI usage
- to let CI verify JSON and `JUnit` output against a real project

The CI script copies this fixture to a temporary directory, rewrites its local
`behave` path dependency to the repository root, and runs:

```bash
cargo-behave behave --output json --manifest-path ./Cargo.toml --package cli-fixture-api
cargo-behave behave --output junit --manifest-path ./crates/api/Cargo.toml
```

The `api` crate contains a `behave.toml` file so the script also verifies that
history output is written relative to the selected package.

The fixture intentionally includes a focused scenario. That lets CI verify
three behaviors at once:

- JSON keeps the raw `checkout::__FOCUS__alpha_case` test name
- the structured tree marks the cleaned leaf as `focused: true`
- JUnit strips the internal marker so CI UIs show `alpha_case`

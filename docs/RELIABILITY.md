# Reliability

This page explains what a user can reasonably rely on in `behave` today.

## Why Teams Trust Libraries Like This

Libraries become credible when they make a few things easy to verify:

- you can reach first success quickly
- examples are runnable, not just aspirational
- there is a task guide and a reference, not only API docs
- support, limitations, and compatibility are stated explicitly
- tests, linting, and documentation checks are visible parts of maintenance

`behave` now follows that pattern with a quick start, user guide, matcher
reference, CLI guide, examples, integration tests, doctests, and strict linting.

## What You Can Rely On Today

- The `behave!` macro expands to ordinary `#[test]` functions, so the runtime
  model stays close to normal Rust tests.
- Public matchers, macros, and CLI modules have executable examples or doctests.
- Runnable examples live in `examples/` and are exercised by integration tests.
- A committed CLI workspace fixture is exercised in CI with real `cargo-behave`
  JSON and `JUnit` runs.
- The crate is checked with unit tests, integration tests, doctests, Clippy, and
  rustdoc warning denial.
- The CLI supports deterministic tree output plus JSON and JUnit reports for CI tooling.
- Flaky detection resolves selected packages through Cargo metadata instead of hashing only the current directory.
- `unsafe` is forbidden by crate lints.
- A security reporting path exists in [SECURITY.md](../SECURITY.md).
- The crate declares an MSRV of Rust 1.75 in `Cargo.toml`.

## What This Crate Does Not Promise

Trust also comes from stating the boundaries clearly.

- `behave` is currently `0.1.0`, so API evolution is still possible.
- `focus` marks scenarios in generated names and CLI output, but it does not
  force focus-only execution.
- Only one `setup` block is allowed per group, and it must appear before child
  scenarios.
- `pending` blocks must be empty.
- Async teardown is error-safe (runs after `?` failures) but not panic-safe
  (no `catch_unwind` across `.await` points). Sync teardown is fully panic-safe.
- Flaky-test detection is heuristic and source-hash based, not a full build
  dependency analysis.

## How To Evaluate It In Your Own Codebase

1. Start with the quick start in [README.md](../README.md).
2. Read [USER_GUIDE.md](USER_GUIDE.md) for the DSL model.
3. Read [MATCHERS.md](MATCHERS.md) for matcher semantics.
4. Try [CLI.md](CLI.md) if you want tree output, machine-readable reports, or flaky detection.
5. Review [tests/smoke.rs](../tests/smoke.rs) for broad behavior coverage.

## Maintenance Signals

- [CHANGELOG.md](../CHANGELOG.md) records user-facing changes.
- [CONTRIBUTING.md](../CONTRIBUTING.md) documents contributor expectations.
- [docs/ARCHITECTURE.md](ARCHITECTURE.md) explains the design model.
- [docs/RELEASE.md](RELEASE.md) documents release flow.

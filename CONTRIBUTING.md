# Contributing

Thank you for your interest in contributing!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/your-username/behave.git`
3. Create a feature branch: `git checkout -b feat/your-feature`
4. Make your changes
5. Run checks: `cargo fmt --all -- --check && cargo clippy --all-features --all-targets -- -D warnings && cargo test --all-features`
6. Commit (see [Commit Messages](#commit-messages))
7. Push and open a pull request

Optionally install [just](https://github.com/casey/just) to run all checks with `just check`.

## Development

```bash
cargo build                      # Build
cargo test --all-features        # Test
cargo clippy --all-features --all-targets -- -D warnings  # Lint
cargo fmt --all                  # Format
```

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`, `ci`.

## Pull Request Checklist

1. **Update CHANGELOG.md** under `[Unreleased]` if your change is user-facing. CI will warn if you forget.
2. Add tests for new functionality.
3. All CI checks pass.
4. Keep PRs focused - one logical change per PR.

## Code Style

See [docs/AGENT.md](docs/AGENT.md) for the full code style guide.

Key points:
- Run `cargo fmt` before committing
- All public items need doc comments with examples
- No `unsafe`, `unwrap()`, `expect()`, `panic!()`, `todo!()`, `println!()` in library code
- Zero Clippy warnings
- Guard clauses over nesting, max 40 lines per function

## License

By contributing, you agree that your contributions will be licensed under the Apache License 2.0.

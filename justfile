# Development task runner
# Install: cargo install just
# Usage: just <recipe>

# Run all checks (mirrors CI)
check: fmt-check clippy test doc markdown-docs cli-fixture

# Format code
fmt:
    cargo fmt --all

# Check formatting without modifying
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
clippy:
    cargo clippy --all-features --all-targets -- -D warnings

# Run all tests
test:
    cargo test --all-features

# Build documentation
doc:
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features

# Test markdown guides and README snippets
markdown-docs:
    bash scripts/test-markdown-docs.sh

# Run the end-to-end CLI fixture check
cli-fixture:
    bash scripts/test-cli-fixture.sh

# Quick dev check (fastest feedback loop)
dev:
    cargo check --all-features

# Run cargo deny checks
deny:
    cargo deny check

# Run semver compatibility check
semver:
    cargo semver-checks check-release

# Test with no default features
test-minimal:
    cargo test --no-default-features

# Test all feature combinations (requires cargo-hack)
test-features:
    cargo hack test --feature-powerset --no-dev-deps

# Prepare release (reads version from VERSION file)
release:
    #!/usr/bin/env bash
    set -euo pipefail
    VERSION=$(cat VERSION | tr -d '[:space:]')
    echo "Preparing release v${VERSION}..."
    sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" Cargo.toml 2>/dev/null || true
    sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" macros/Cargo.toml 2>/dev/null || true
    cargo check
    echo "Ready. Review changes, commit, and tag with: git tag v${VERSION}"

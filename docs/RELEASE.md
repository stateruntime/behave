# Release Process

## Single Source of Truth

The `VERSION` file at the repository root is the single source of truth for the crate version. `Cargo.toml` and git tags must match it. CI enforces the `VERSION` ↔ `Cargo.toml` sync.

## Steps

### 1. Update VERSION and Cargo.toml

```bash
# Set the new version
echo "1.2.0" > VERSION

# Sync Cargo.toml (or manually edit the version field)
just release
```

### 2. Update CHANGELOG.md

Rename `[Unreleased]` to the new version with today's date:

```markdown
## [1.2.0] - 2026-03-10

### Added
- ...
```

Add a fresh `[Unreleased]` section above it. Update comparison links at the bottom.

### 3. Pre-release checks

```bash
just check              # fmt, clippy, test, doc, markdown guides, CLI fixture
cargo publish --dry-run # verify it can be packaged
```

### 4. Commit and tag

```bash
git add VERSION Cargo.toml macros/Cargo.toml CHANGELOG.md
git commit -m "chore: release v1.2.0"
git tag v1.2.0
git push origin main v1.2.0
```

### 5. Publish

The `v*` tag triggers `.github/workflows/release.yml` which:
1. Validates the tag matches the VERSION file
2. Runs the full test suite
3. Publishes `behave` to crates.io (requires `CARGO_REGISTRY_TOKEN` secret)
4. Creates a GitHub Release with notes extracted from CHANGELOG.md

The `behave-macros` subcrate is not published separately - it is included as a path dependency.

If you haven't set up the crates.io token yet, publish manually: `cargo publish`

## Version Bump Rules (SemVer)

| Change | Bump |
|--------|------|
| Breaking API change | **Major** (x.0.0) |
| New feature, backwards compatible | **Minor** (0.x.0) |
| Bug fix, no API change | **Patch** (0.0.x) |
| MSRV bump | **Minor** minimum |
| New optional feature flag | **Minor** |

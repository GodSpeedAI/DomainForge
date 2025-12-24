# Release Process Documentation

This document describes how to create releases for DomainForge.

## Quick Start

```bash
# Patch release (e.g., 0.6.2 -> 0.6.3)
./scripts/release.sh

# Minor release (e.g., 0.6.2 -> 0.7.0)
./scripts/release.sh minor

# Major release (e.g., 0.6.2 -> 1.0.0)
./scripts/release.sh major

# Preview what would happen (dry run)
./scripts/release.sh --dry-run
```

## Release Methods

### Method 1: Local Scripts (Recommended for maintainers)

Use the shell scripts for full control over the release process:

```bash
# Full release with all steps
./scripts/release.sh patch

# Or run individual steps
./scripts/pre-release-check.sh
./scripts/bump-version.sh patch
./scripts/generate-changelog.sh 0.7.0
./scripts/generate-release-notes.sh 0.7.0
./scripts/create-tag.sh 0.7.0
./scripts/build-release.sh
./scripts/create-github-release.sh 0.7.0
```

### Method 2: GitHub Actions (Recommended for CI-driven releases)

1. Go to **Actions** → **Prepare Release** → **Run workflow**
2. Select version bump type (patch/minor/major)
3. Review and merge the created PR
4. Create and push tag: `git tag v<version> && git push --tags`
5. Create a GitHub Release from the tag
6. Publishing workflows trigger automatically

### Method 3: Justfile Recipes

```bash
# Preview release changes
just release-preview patch

# Prepare release locally (updates files, no push)
just prepare-release patch

# Full CI pipeline
just ci-pipeline
```

## Script Reference

### pre-release-check.sh

Validates the repository is ready for release.

```bash
./scripts/pre-release-check.sh [OPTIONS]

Options:
  --dry-run       Preview checks without running tests
  --skip-tests    Skip test suites (faster)
  --verbose       Verbose output

Checks:
  ✓ No uncommitted changes
  ✓ On valid branch (main, dev, release/*)
  ✓ Up to date with remote
  ✓ All tests pass
  ✓ Version sync across files
  ✓ CHANGELOG.md is up to date
```

### bump-version.sh

Updates version across all package files.

```bash
./scripts/bump-version.sh [VERSION_TYPE] [OPTIONS]

Types:
  major           Bump major (X.0.0)
  minor           Bump minor (x.Y.0)
  patch           Bump patch (x.y.Z)
  X.Y.Z           Explicit version
  X.Y.Z-suffix    Pre-release version

Options:
  --dry-run       Preview changes
  --no-commit     Update files without committing

Files updated:
  - sea-core/Cargo.toml
  - pyproject.toml
  - package.json
```

### generate-changelog.sh

Generates changelog entries from conventional commits.

```bash
./scripts/generate-changelog.sh [VERSION] [OPTIONS]

Options:
  --dry-run       Preview without modifying
  --no-commit     Update file without committing

Categories:
  feat:      → 🎉 Added
  fix:       → 🐛 Fixed
  docs:      → 📚 Documentation
  refactor:  → ✨ Changed
```

### generate-release-notes.sh

Creates GitHub release notes from CHANGELOG.md.

```bash
./scripts/generate-release-notes.sh [VERSION] [OPTIONS]

Options:
  --dry-run       Preview content

Output:
  Creates RELEASE_NOTES.md with:
  - Version header
  - What's Changed section
  - Breaking Changes (if any)
  - Contributors list
```

### create-tag.sh

Creates an annotated git tag.

```bash
./scripts/create-tag.sh [VERSION] [OPTIONS]

Options:
  --dry-run       Preview without creating
  --force         Overwrite existing tag

Features:
  - Uses RELEASE_NOTES.md for tag message
  - Supports GPG signing if configured
```

### build-release.sh

Builds release artifacts for the current platform.

```bash
./scripts/build-release.sh [OPTIONS]

Options:
  --dry-run       Preview build commands
  --skip-cli      Skip CLI binary
  --skip-python   Skip Python wheel
  --skip-wasm     Skip WASM bundle

Output:
  dist/
  ├── sea-0.7.0-linux-x86_64.tar.gz
  ├── sea_dsl-0.7.0-cp312-*.whl
  ├── sea-core-wasm-0.7.0.tar.gz
  └── SHA256SUMS.txt
```

### create-github-release.sh

Creates a GitHub release using the gh CLI.

```bash
./scripts/create-github-release.sh [VERSION] [OPTIONS]

Options:
  --dry-run       Preview without creating
  --draft         Create as draft

Requirements:
  - GitHub CLI (gh) installed and authenticated
  - Tag must exist
```

### release.sh

Master orchestration script that runs all steps.

```bash
./scripts/release.sh [VERSION_TYPE] [OPTIONS]

Options:
  --dry-run       Full dry run of all steps
  --skip-tests    Skip test suites
  --skip-build    Skip building artifacts
  --yes, -y       Auto-confirm push

Steps:
  1. Pre-release checks
  2. Version bump
  3. Changelog generation
  4. Release notes generation
  5. Commit changes
  6. Create git tag
  7. Build artifacts
  8. Push to remote
  9. Create GitHub release
```

## Prerequisites

### Required Tools

| Tool      | Purpose         | Installation                             |
| --------- | --------------- | ---------------------------------------- |
| git       | Version control | System package manager                   |
| cargo     | Rust toolchain  | [rustup.rs](https://rustup.rs)           |
| just      | Task runner     | `cargo install just`                     |
| maturin   | Python wheels   | `pip install maturin`                    |
| wasm-pack | WASM bundle     | `cargo install wasm-pack`                |
| gh        | GitHub CLI      | [cli.github.com](https://cli.github.com) |

### GitHub Authentication

```bash
# Authenticate with GitHub CLI
gh auth login

# Verify authentication
gh auth status
```

## Troubleshooting

### "Uncommitted changes detected"

```bash
# Check what's modified
git status

# Stage and commit or stash
git add -A && git commit -m "wip"
# OR
git stash
```

### "Version mismatch across files"

```bash
# Check versions
grep '^version' sea-core/Cargo.toml pyproject.toml
grep '"version"' package.json

# Sync manually or run bump-version
./scripts/bump-version.sh 0.7.0
```

### "Tag already exists"

```bash
# Delete local tag
git tag -d v0.7.0

# Delete remote tag
git push origin --delete v0.7.0

# Or use --force
./scripts/create-tag.sh --force
```

### "GitHub CLI not authenticated"

```bash
gh auth login
# Follow prompts to authenticate
```

## Rollback Procedures

### Before Push

If release failed before pushing:

```bash
# Reset to previous commit
git reset --hard HEAD~1

# Delete local tag
git tag -d v0.7.0
```

### After Push (Breaking Release)

```bash
# Delete remote tag (stops CI publishing)
git push origin --delete v0.7.0

# Delete GitHub release via web UI or CLI
gh release delete v0.7.0

# Revert commit
git revert HEAD
git push origin main
```

### Emergency Hotfix

```bash
# Create hotfix branch from main
git checkout main
git pull
git checkout -b hotfix/0.7.1

# Apply fix, commit, then release
./scripts/release.sh patch
```

## Version Numbering

DomainForge follows [Semantic Versioning](https://semver.org/):

- **Major (X.0.0)**: Breaking API changes
- **Minor (x.Y.0)**: New features, backward compatible
- **Patch (x.y.Z)**: Bug fixes, backward compatible
- **Pre-release (x.y.z-alpha)**: Development versions

## Files Updated During Release

| File                  | Field                | Example                   |
| --------------------- | -------------------- | ------------------------- |
| `sea-core/Cargo.toml` | `version = "X.Y.Z"`  | `version = "0.7.0"`       |
| `pyproject.toml`      | `version = "X.Y.Z"`  | `version = "0.7.0"`       |
| `package.json`        | `"version": "X.Y.Z"` | `"version": "0.7.0"`      |
| `CHANGELOG.md`        | New version section  | `## [0.7.0] - 2025-12-23` |
| `RELEASE_NOTES.md`    | Generated content    | GitHub release body       |

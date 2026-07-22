# GitHub Workflows Documentation

This directory contains the CI/CD workflows for the DomainForge project.

## Workflows Overview

| Workflow                   | Trigger                          | Purpose                                   |
| -------------------------- | -------------------------------- | ----------------------------------------- |
| `ci.yml`                   | Push to main/release/**, PRs     | Continuous Integration                    |
| `release.yml`              | Tag `v*.*.*`                     | Build artifacts, create release, dispatch publishes |
| `release-npm.yml`          | `workflow_call` from release.yml | Publish napi + WASM to npm                |
| `release-pypi.yml`         | `workflow_call` from release.yml | Publish Python wheels to PyPI             |
| `release-crates.yml`       | `workflow_call` from release.yml | Publish to crates.io                      |
| `prepare-release.yml`      | Manual trigger                   | Automate version bump and release PR      |
| `dependabot-automerge.yml` | Dependabot PRs                   | Auto-merge safe dependency updates        |
| `dependency-review.yml`    | PRs                              | Review dependency changes for security    |

## Workflow Details

### `ci.yml` - Continuous Integration

Main CI pipeline that runs on pushes to `main` and `release/**` branches, as well as pull requests targeting `main`.

**Jobs:**

- **lint**: Runs `rustfmt` and `clippy` on all Rust code
- **test-rust**: Runs Rust tests on Linux, macOS, and Windows
- **test-python**: Runs Python tests on Python 3.11 and 3.12
- **test-typescript**: Runs TypeScript/Vitest tests
- **test-integration**: Minimal integration checks including registry ambiguity validation
- **test-wasm**: Builds and validates WASM bundle size
- **security**: Runs `cargo audit` for security vulnerabilities

### `release.yml` - Release Builds

Triggered on version tags (`v*.*.*`). Builds release artifacts for all platforms.

**Supported Targets:**

| Target                      | OS Runner      | Notes                      |
| --------------------------- | -------------- | -------------------------- |
| `x86_64-unknown-linux-gnu`  | ubuntu-latest  | Standard Linux             |
| `x86_64-apple-darwin`       | macos-15-intel | Intel Mac                  |
| `x86_64-pc-windows-msvc`    | windows-2025-vs2026 | Windows                    |
| `aarch64-apple-darwin`      | macos-15       | Apple Silicon              |
| `aarch64-unknown-linux-gnu` | ubuntu-latest  | ARM Linux (cross-compiled) |

**Jobs:**

- **build-release**: Builds CLI binaries for all targets
- **build-python-wheels**: Builds Python wheels for all targets
- **build-wasm-release**: Builds optimized WASM bundle
- **create-release**: Creates GitHub release with all artifacts
- **publish-pypi**: Calls `release-pypi.yml` to publish wheels to PyPI
- **publish-npm**: Calls `release-npm.yml` to publish napi + WASM to npm
- **publish-crates**: Calls `release-crates.yml` to publish `domainforge-core` to crates.io

### `prepare-release.yml` - Release Automation

Manually triggered workflow to automate version bumping and release PR creation.

**Inputs:**

- `version_bump`: Choose `patch`, `minor`, or `major`
- `prerelease`: Optional suffix like `alpha`, `beta`, or `rc1`

**What it does:**

1. Calculates new version based on bump type
2. Updates `domainforge-core/Cargo.toml` with new version
3. Prepares CHANGELOG.md entry
4. Creates a release PR with checklist

### Publishing Workflows

| Workflow             | Registry  | Notes                                            |
| -------------------- | --------- | ------------------------------------------------ |
| `release-npm.yml`    | npm       | Publishes both napi bindings AND WASM package    |
| `release-pypi.yml`   | PyPI      | Publishes wheels for all platforms including ARM |
| `release-crates.yml` | crates.io | Publishes `domainforge-core` crate                       |

## Bundle Size Thresholds

| Artifact     | Limit | Notes                                    |
| ------------ | ----- | ---------------------------------------- |
| WASM bundle  | 2.75MB | Harmonized across ci.yml and release.yml |
| CLI binary   | 50MB  | Per-platform binary                      |
| CLI artifact | 70MB  | Packaged archive (tar.gz/zip)            |

## Local Testing to Match CI

```bash
# Run all tests
just all-tests

# Or individually
just rust-test
just python-test
just ts-test

# WASM build and size check
cd domainforge-core
wasm-pack build --target web --features wasm
SIZE=$(python3 -c "import os; print(os.path.getsize('pkg/domainforge_core_bg.wasm'))")
echo "WASM bundle size: $SIZE bytes (threshold: 2883584)"
[ "$SIZE" -lt 2883584 ] && echo "PASS" || echo "FAIL"

# Lint checks
cargo fmt --all --check
cargo clippy --all-targets --all-features -- -D warnings
```

## Release Process

### Automated (Recommended)

1. Go to Actions → "Prepare Release" → Run workflow
2. Select version bump type (patch/minor/major)
3. Review and merge the created PR
4. Create and push tag: `git tag v<version> && git push --tags`
5. `release.yml` runs automatically on the tag: it builds artifacts, creates the GitHub Release, and then calls the three publish workflows (`release-pypi.yml`, `release-npm.yml`, `release-crates.yml`) via `workflow_call`.

> **Note:** Publishing is dispatched by `release.yml` via `workflow_call`, not by the `release: published` event. This is intentional: events produced by the default `GITHUB_TOKEN` do not trigger downstream workflows, so relying on `release: published` would silently skip publishing when the release is created automatically. Calling the publish workflows directly from `release.yml` is deterministic and requires no extra tokens.

### Manual

1. Bump version in `domainforge-core/Cargo.toml`
2. Update `CHANGELOG.md`
3. Commit and push
4. Create and push tag: `git tag v<version> && git push --tags`
5. `release.yml` builds, creates the release, and dispatches publishes automatically

## Cache Management

All workflows use GitHub Actions cache (v5) with a `CACHE_VERSION` environment variable. To bust all caches:

1. Increment `CACHE_VERSION` in the workflow file
2. This is useful when dependencies are corrupted or need a fresh start

## Secrets Required

| Secret           | Used By               | Purpose                       |
| ---------------- | --------------------- | ----------------------------- |
| `SOPS_AGE_KEY`   | All publish workflows | Decrypt encrypted secrets     |
| `PYPI_API_TOKEN` | release-pypi.yml      | PyPI publishing (fallback)    |
| `GITHUB_TOKEN`   | All workflows         | GitHub API access (automatic) |

## Troubleshooting

### ARM Linux Cross-Compilation

ARM Linux targets use either `cross` (for CLI) or `zig` (for Python wheels) for cross-compilation. If builds fail:

1. Check that the target toolchain is installed
2. Verify cross/zig are working correctly
3. ARM Linux builds cannot be verified locally on x86 runners

### Publish Failures

All publish workflows have `continue-on-error: true` or `--skip-existing` to handle:

- Package already published (re-runs)
- Network issues (will fail but won't block other jobs)

To check if a publish actually succeeded, verify the package on the respective registry.

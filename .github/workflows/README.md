# GitHub Workflows Documentation

This directory contains the CI/CD workflows for the DomainForge project.

## Workflows

### `ci.yml` - Continuous Integration

Main CI pipeline that runs on pushes to `main`, `dev`, and `release/**` branches, as well as pull requests.

**Jobs:**

- **lint**: Runs `rustfmt` and `clippy` on all Rust code
- **test-rust**: Runs Rust tests on Linux, macOS, and Windows
- **test-python**: Runs Python tests on Python 3.11 and 3.12
- **test-typescript**: Runs TypeScript/Vitest tests
- **test-integration**: Minimal integration checks including registry ambiguity validation
- **test-wasm**: Builds and validates WASM bundle size
- **security**: Runs `cargo audit` for security vulnerabilities

**Requirements:**

- Node.js: v22 (matches local development environment)
- Python: 3.11+ (per `pyproject.toml`)
- Rust: stable toolchain

**Bundle Size Thresholds:**

- WASM: 512KB (524,288 bytes)
- CLI binary: 50MB (52,428,800 bytes)

### `release.yml` - Release Builds

Triggered on version tags (`v*.*.*`). Builds release artifacts for all platforms.

**Jobs:**

- **build-release**: Builds CLI binaries for Linux, macOS, Windows
- **build-python-wheels**: Builds Python wheels for all platforms
- **build-wasm-release**: Builds optimized WASM bundle
- **create-release**: Creates GitHub release with all artifacts

**Artifact Size Limits:**

- CLI binary: 50MB
- CLI packaged artifact: 70MB
- WASM bundle: 512KB

### `publish-python.yml` - Python Package Publishing

Triggered on GitHub releases. Publishes Python wheels to PyPI.

**Requirements:**

- Repository secret: `PYPI_API_TOKEN` (optional, publish skipped if not set)
- Only runs on `release` events

### `dependency-review.yml` - Dependency Security

Runs on pull requests to review dependency changes for security issues.

## Local Testing to Match CI

To ensure your local tests match the CI environment:

### 1. Node.js Version

```bash
# Check your Node version
node --version  # Should be v22.x.x

# If using nvm, switch to Node 22
nvm use 22
```

### 2. Run All Tests

```bash
# Using justfile (recommended)
just all-tests

# Or individually
just rust-test
just python-test
just ts-test
```

### 3. WASM Build and Size Check

```bash
cd sea-core
wasm-pack build --target web --features wasm

# Check bundle size (should be < 512KB)
SIZE=$(python3 -c "import os; print(os.path.getsize('pkg/sea_core_bg.wasm'))")
echo "WASM bundle size: $SIZE bytes (threshold: 524288)"
[ "$SIZE" -lt 524288 ] && echo "✅ PASS" || echo "❌ FAIL"
```

### 4. Lint Checks

```bash
# Rust formatting
cargo fmt --all --check

# Rust linting
cargo clippy --all-targets --all-features -- -D warnings
```

## Troubleshooting

### WASM Bundle Size Failure

If the WASM bundle exceeds 512KB:

1. Check what changed in the Rust code that increased bundle size
2. Consider using `wasm-opt` for additional optimization
3. Review dependencies added to the WASM feature
4. If the increase is justified, update the threshold in both `ci.yml` and `release.yml`

### Publish Job Failures

The `publish-python.yml` workflow will skip publishing if:

- `PYPI_API_TOKEN` secret is not configured (expected behavior)
- The workflow is not triggered by a release event

To configure publishing:

1. Generate a PyPI API token at https://pypi.org/manage/account/token/
2. Add it as a repository secret named `PYPI_API_TOKEN`
3. Create a GitHub release to trigger the workflow

### Node Version Mismatch

If you see different behavior locally vs CI:

1. Ensure you're using Node v22 locally
2. Run `npm ci` (not `npm install`) to match CI's dependency resolution
3. Check `package-lock.json` is committed and up-to-date

## Cache Management

All workflows use GitHub Actions cache (v4) with a `CACHE_VERSION` environment variable. To bust all caches:

1. Increment `CACHE_VERSION` in the workflow file
2. This is useful when dependencies are corrupted or need a fresh start

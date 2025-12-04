# Just Recipes Reference

Quick reference for all `just` task runner recipes in DomainForge.

## Usage

```bash
just <recipe-name> [arguments]
```

List all available recipes:

```bash
just --list
```

---

## Testing Recipes

| Recipe | Description | Example |
|--------|-------------|---------|
| `rust-test` | Run Rust tests with CLI feature | `just rust-test` |
| `python-test` | Run Python tests (uses .venv if available) | `just python-test` |
| `ts-test` | Run TypeScript tests via Vitest | `just ts-test` |
| `all-tests` | Run all test suites (Rust, Python, TypeScript) | `just all-tests` |
| `cli-test` | Run CLI-specific tests | `just cli-test` |

---

## Merge & CI Recipes

| Recipe | Description | Example |
|--------|-------------|---------|
| `pre-merge` | Run format, lint, and all tests | `just pre-merge` |
| `ci-check` | Run CI-equivalent checks | `just ci-check` |
| `merge-to-main <branch>` | Full automated merge workflow | `just merge-to-main feature/foo` |
| `merge-to-main-dry <branch>` | Preview merge without changes | `just merge-to-main-dry feature/foo` |

### Merge Workflow Steps

When you run `just merge-to-main <branch>`:

1. Fetch latest from origin
2. Validate branch exists
3. Merge main into feature branch
4. Run pre-merge checks (format, clippy, tests)
5. Merge to main with `--no-ff`
6. Run post-merge tests
7. Push main and delete branch

---

## CI-Specific Recipes

Used by GitHub Actions but can be run locally:

| Recipe | Description | Example |
|--------|-------------|---------|
| `ci-test-rust` | Run Rust tests (verbose, CI variant) | `just ci-test-rust` |
| `ci-test-python` | Run Python tests (CI variant) | `just ci-test-python` |
| `ci-test-ts` | Run TypeScript tests (CI variant) | `just ci-test-ts` |
| `ci-verify-binary <path>` | Verify CLI binary executes | `just ci-verify-binary ./target/release/sea` |
| `ci-check-binary-size <path>` | Check binary size limits | `just ci-check-binary-size ./target/release/sea` |
| `ci-package-binary <in> <out>` | Package binary for release | `just ci-package-binary ./sea ./sea.tar.gz` |
| `ci-verify-package <archive>` | Verify packaged archive | `just ci-verify-package ./sea.tar.gz` |

---

## Setup & Environment Recipes

| Recipe | Description | Example |
|--------|-------------|---------|
| `setup` | Install all dependencies (npm + Python) | `just setup` |
| `python-setup` | Create venv and install Python deps | `just python-setup` |
| `python-clean` | Remove Python virtual environment | `just python-clean` |
| `install-just` | Install just task runner | `just install-just` |

---

## Debug Recipes

| Recipe | Description | Example |
|--------|-------------|---------|
| `prepare-rust-debug` | Build & symlink test binary for codelldb | `just prepare-rust-debug` |
| `clear-rust-debug` | Remove debug symlink | `just clear-rust-debug` |
| `build-rust-tests` | Build tests without running | `just build-rust-tests` |

Set test name via environment variable:

```bash
RUST_TEST_NAME=parser_tests just prepare-rust-debug
```

---

## CLI Validation Recipes

| Recipe | Description | Example |
|--------|-------------|---------|
| `cli-validate` | Run CLI validation examples | `just cli-validate` |
| `cli-workflow` | Run CLI import/export workflow | `just cli-workflow` |

---

## Default Recipe

Running `just` without arguments executes:

```bash
just ai-validate  # Runs: cargo test -p sea-core --features cli
```

---

## Common Workflows

### Daily Development

```bash
# Run tests during development
just rust-test

# Run all tests before committing
just all-tests
```

### Feature Branch Workflow

```bash
# While developing
just pre-merge          # Check everything passes

# Ready to merge
just merge-to-main feature/my-feature
```

### Setting Up a New Environment

```bash
# Full setup
just setup

# Or just Python
just python-setup
```

### Debugging Rust Tests

```bash
# Prepare for debugging
RUST_TEST_NAME=instance_parsing_tests just prepare-rust-debug

# After debugging
just clear-rust-debug
```

---

## Related Documentation

- [How to Merge Feature Branches](../Howto/Merge_Feature_Branches.md)
- [Merge Automation Workflow](../plans/merge-automation-workflow.md)
- [CI/CD Integration Guide](../guides/cicd_integration.md)


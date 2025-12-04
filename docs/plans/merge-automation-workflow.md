# Merge Automation Workflow

## Overview

This document describes the automated workflow for merging feature branches into `main` with full CI-equivalent validation. The workflow ensures code quality, prevents regressions, and maintains a clean git history.

## Goals

1. **Automate pre-merge validation** - Run all checks locally before merge
2. **Prevent regressions** - Ensure tests pass on both feature branch and main after merge
3. **Maintain consistency** - Apply same checks as GitHub Actions CI
4. **Clean up branches** - Automatically delete merged branches
5. **Provide immediate feedback** - Fail fast on issues

## Workflow Steps

```
┌─────────────────────────────────────────────────────────────────┐
│                    Merge Automation Workflow                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Fetch & Validate Branch                                      │
│     └── git fetch --all --prune                                  │
│     └── Verify feature branch exists                             │
│                                                                  │
│  2. Update Feature Branch                                        │
│     └── git checkout <feature-branch>                            │
│     └── git merge origin/main (catch conflicts early)            │
│                                                                  │
│  3. Pre-Merge Checks (on feature branch)                         │
│     └── cargo fmt --all -- --check                               │
│     └── cargo clippy -p sea-core -- -D warnings                  │
│     └── just all-tests (Rust, Python, TypeScript)                │
│                                                                  │
│  4. Merge to Main                                                │
│     └── git checkout main                                        │
│     └── git pull --ff-only origin main                           │
│     └── git merge --no-ff <feature-branch>                       │
│                                                                  │
│  5. Post-Merge Validation (on main)                              │
│     └── just all-tests                                           │
│                                                                  │
│  6. Push & Cleanup                                               │
│     └── git push origin main                                     │
│     └── git branch -d <feature-branch>                           │
│     └── git push origin --delete <feature-branch>                │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Usage

### Quick Merge (recommended)

```bash
# Merge a feature branch into main with full validation
just merge-to-main feature/my-feature
```

### Manual Steps

```bash
# Run pre-merge checks only (without merging)
just pre-merge

# Run CI-equivalent checks
just ci-check
```

### Dry Run

```bash
# Preview what would happen without making changes
just merge-to-main-dry feature/my-feature
```

## Just Recipes

The following recipes are added to the `justfile`:

| Recipe | Description |
|--------|-------------|
| `pre-merge` | Run format, lint, and all tests |
| `ci-check` | Run CI-equivalent checks (format + clippy + tests) |
| `merge-to-main <branch>` | Full automated merge workflow |
| `merge-to-main-dry <branch>` | Dry run of merge workflow |

## GitHub Actions Integration

The existing `.github/workflows/ci.yml` already runs:

- `cargo fmt --all --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- Rust tests on Linux, macOS, Windows
- Python tests
- TypeScript tests

The local `just` recipes mirror these checks for immediate feedback.

## Best Practices

### Before Merging

1. Ensure your feature branch is up-to-date with `main`
2. Run `just pre-merge` to catch issues early
3. Address any CodeRabbit or review feedback
4. Squash fixup commits if desired (optional)

### During Merge

1. Use `just merge-to-main <branch>` for automation
2. Review the merge commit message
3. Monitor test output for failures

### After Merging

1. Verify the branch was deleted locally and remotely
2. Check GitHub Actions CI passes on `main`
3. Update any related PRs or issues

## Troubleshooting

### Merge Conflicts

If conflicts occur during `git merge origin/main`:

```bash
# Resolve conflicts manually
git status
# Edit conflicting files
git add <resolved-files>
git commit
# Then re-run pre-merge checks
just pre-merge
```

### Test Failures

If tests fail:

1. Check the specific test output
2. Fix the issue on the feature branch
3. Re-run `just merge-to-main <branch>`

### Clippy Warnings

If clippy fails with warnings:

```bash
# See all warnings
cargo clippy -p sea-core -- -D warnings 2>&1 | head -100

# Auto-fix where possible
cargo clippy --fix -p sea-core --allow-dirty
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `MERGE_SKIP_TESTS` | `false` | Skip tests (dangerous, not recommended) |
| `MERGE_DRY_RUN` | `false` | Preview without making changes |

## Related Documentation

- [Contributing Guide](../../CONTRIBUTING.md)
- [CI/CD Integration](../guides/cicd_integration.md)
- [Cross-Language Development](../cross_language_development.md)

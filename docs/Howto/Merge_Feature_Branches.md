# How to Merge Feature Branches

This guide walks you through merging feature branches into `main` using the automated workflow.

## Prerequisites

- Git installed and configured
- `just` task runner installed (run `just install-just` if needed)
- All dependencies installed (`just setup`)

## Quick Start

Merge a feature branch with full validation:

```bash
just merge-to-main feature/my-feature
```

That's it! The workflow handles everything automatically.

## Step-by-Step Guide

### 1. Verify Your Branch is Ready

Before merging, ensure your feature branch:

- Has all changes committed
- Has been pushed to origin (if applicable)
- Passes local tests

```bash
# Check current branch
git status

# Run tests locally
just all-tests
```

### 2. Preview the Merge (Optional)

See what the merge workflow will do without making changes:

```bash
just merge-to-main-dry feature/my-feature
```

This shows all 7 steps that will execute.

### 3. Run the Merge

Execute the full automated merge:

```bash
just merge-to-main feature/my-feature
```

The workflow performs these steps automatically:

1. **Fetch** - Gets latest changes from origin
2. **Validate** - Ensures the branch exists
3. **Update** - Merges `main` into your branch to catch conflicts early
4. **Pre-merge checks** - Runs format, lint, and all tests
5. **Merge** - Creates a merge commit on `main`
6. **Post-merge validation** - Runs tests again on `main`
7. **Cleanup** - Pushes and deletes the branch

### 4. Verify Success

After a successful merge, you'll see:

```
✅ Successfully merged 'feature/my-feature' into main!
   - All checks passed
   - Branch deleted locally and remotely
   - Main pushed to origin
```

## Manual Pre-Merge Checks

If you want to run validation without merging:

```bash
# Run format, lint, and all tests
just pre-merge

# Or run CI-equivalent checks
just ci-check
```

## Handling Failures

### Merge Conflicts

If conflicts occur during the merge with `main`:

```bash
# The workflow stops and shows:
# ❌ Merge conflict with main. Resolve manually.

# Resolve conflicts:
git status                    # See conflicting files
# Edit files to resolve conflicts
git add <resolved-files>
git commit -m "Resolve merge conflicts"

# Re-run the workflow
just merge-to-main feature/my-feature
```

### Test Failures

If tests fail:

```bash
# The workflow shows which step failed:
# ❌ Tests failed on feature branch

# Fix the issues on your branch
git checkout feature/my-feature
# Make fixes...
git commit -m "Fix test failures"

# Try again
just merge-to-main feature/my-feature
```

### Lint/Format Issues

```bash
# If format check fails:
cargo fmt --all

# If clippy fails:
cargo clippy --fix -p sea-core --allow-dirty

# Commit fixes and retry
git add -A && git commit -m "fix: Address lint issues"
just merge-to-main feature/my-feature
```

## Best Practices

### Before Merging

1. **Pull latest main** into your branch regularly during development
2. **Run `just pre-merge`** before the final merge attempt
3. **Address code review feedback** (check CodeRabbit issues)
4. **Write descriptive commit messages**

### During Merge

1. **Monitor the output** for any warnings or failures
2. **Don't interrupt** the workflow once started
3. **Review the merge commit message** in the output

### After Merging

1. **Check GitHub Actions** to ensure CI passes on `main`
2. **Update related issues/PRs** to reference the merge
3. **Notify team members** if the merge affects their work

## Common Scenarios

### Merge Without Remote Branch

If your branch only exists locally:

```bash
# Works fine - remote deletion step is skipped
just merge-to-main feature/local-only-branch
```

### Merge Someone Else's Branch

```bash
# Fetch their branch first
git fetch origin feature/their-branch:feature/their-branch

# Then merge
just merge-to-main feature/their-branch
```

### Skip Remote Branch Deletion

The workflow automatically skips remote deletion if the branch doesn't exist on origin.

## Related Documentation

- [Merge Automation Workflow Plan](../plans/merge-automation-workflow.md) - Detailed workflow specification
- [CI/CD Integration Guide](../guides/cicd_integration.md) - GitHub Actions setup
- [Contributing Guide](../../CONTRIBUTING.md) - General contribution guidelines

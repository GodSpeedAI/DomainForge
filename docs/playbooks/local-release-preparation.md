# Local Release Preparation

This playbook describes how to prepare a release locally using the just recipes, allowing you to preview changes, generate replay scripts, and test the release process before triggering the GitHub Actions workflow.

## Available Recipes

| Recipe                         | Purpose                                                         |
| ------------------------------ | --------------------------------------------------------------- |
| `just release-preview [bump]`  | Preview all changes + generate replay script (no modifications) |
| `just prepare-release [bump]`  | Apply version bump to all files                                 |
| `just changelog-entry VERSION` | Add a changelog entry for a specific version                    |
| `just test-changelog-logic`    | Test changelog insertion logic (restores original)              |

Where `[bump]` is one of: `patch`, `minor`, `major`

## Prerequisites

- `jq` installed (for JSON processing)
- `just` command runner
- Clean git working directory (recommended)

## Workflow

### 1. Preview Changes (Recommended First Step)

```bash
just release-preview minor
```

This will:

- Calculate the new version based on bump type
- Show unified diffs for all affected files:
  - `sea-core/Cargo.toml`
  - `pyproject.toml`
  - `package.json`
  - `CHANGELOG.md`
- Generate an executable replay script (e.g., `release-0.5.0.sh`)

**No files are modified** - this is purely a preview.

### 2. Apply Changes

You have two options:

**Option A: Run the generated script**

```bash
./release-0.5.0.sh
```

**Option B: Use the just recipe directly**

```bash
just prepare-release minor
```

Both approaches produce identical results.

### 3. Review and Commit

```bash
# Review the changes
git diff

# Edit CHANGELOG.md with actual release notes
vim CHANGELOG.md

# Commit
git add -A
git commit -m "chore: bump version to 0.5.0"
```

### 4. Continue with Release

After preparing locally, you can either:

1. **Push and let CI handle it**: Push to `dev`, merge to `main`, create a tag
2. **Use the GitHub workflow**: Trigger `prepare-release.yml` which will create a PR

## Pre-release Versions

To create a pre-release (e.g., `0.5.0-beta.1`):

```bash
# Preview
just release-preview minor beta.1

# Apply
just prepare-release minor beta.1
```

## Debugging

### Test Changelog Logic Only

```bash
just test-changelog-logic
```

This adds a test entry to CHANGELOG.md, displays the result, then restores the original file.

### Verbose Diff

The `release-preview` recipe shows diffs for all files. If you need to inspect a specific file more closely:

```bash
just release-preview patch 2>&1 | grep -A 50 "Cargo.toml"
```

## Integration with GitHub Actions

The local recipes mirror the logic in `.github/workflows/prepare-release.yml`:

| GitHub Action Step    | Local Equivalent                                      |
| --------------------- | ----------------------------------------------------- |
| Calculate new version | Same algorithm in `release-preview`/`prepare-release` |
| Bump Cargo.toml       | `sed` replacement                                     |
| Sync pyproject.toml   | `sed` replacement                                     |
| Sync package.json     | `npm version`                                         |
| Update CHANGELOG.md   | `just changelog-entry`                                |

This ensures you can validate the release logic locally before triggering the workflow.

## See Also

- [Releasing Beta](./releasing-beta.md) - Full release process
- [Secret Management](./secret-management.md) - Managing release tokens

# Handoff: Release Pipeline Fix — Publish to npm, PyPI, crates.io

**Date**: 2026-06-19
**Branch**: `fix/release-pipeline-multi-component`
**PR**: https://github.com/GodSpeedAI/DomainForge/pull/87
**Status**: Code changes complete, blocked on npm org creation

---

## Context

The v0.12.0 release was published on GitHub but packages never reached npm, PyPI, or crates.io. Root cause was a broken multi-component release pipeline.

## What Was Done

### Code changes (committed as `3d59358`, pushed to branch)

1. **`release-please-config.json`** — Added `linked-versions` plugin (group: `domainforge`, components: `sea-core`, `sea-dsl`, `sea`). Removed `separate-pull-requests: true`. This ensures all three components release together with the same version and all three tags are created.

2. **`.github/workflows/release.yml`** — Deleted. It triggered on `v*.*.*` tags which release-please no longer produces (now uses `{component}-v*` tags). It was unreachable.

3. **`.github/workflows/release-pypi.yml`** — Removed redundant `--manifest-path ../sea-core/Cargo.toml` CLI flags from both maturin publish commands. Already specified in `sea-dsl/pyproject.toml` under `[tool.maturin]`.

4. **Version sync to 0.12.0** — Updated `sea-core/Cargo.toml`, `sea-dsl/pyproject.toml`, `sea-typescript/package.json`, `.release-please-manifest.json`, and `Cargo.lock`.

### Root causes identified

| # | Issue | Status |
|---|-------|--------|
| 1 | `separate-pull-requests: true` + all Rust source in `sea-core/` meant release-please never detected changes for sea-dsl or sea-typescript, so no tags were created for them | **Fixed** (linked-versions) |
| 2 | deploy.yml at tagged commit `7137599` had **stub** build/deploy steps and a `prod` environment gate — publish workflows were never called | **Fixed** (correct deploy.yml is on this branch) |
| 3 | Old `release.yml` triggered on `v*` tags, unreachable after switch to component-prefixed tags | **Fixed** (deleted) |
| 4 | `@godspeedai` npm org does not exist — npm publish returns E404 | **Blocked — user action required** |
| 5 | Redundant `--manifest-path` in release-pypi.yml | **Fixed** |

## What Remains

### Step 1: Create @godspeedai npm org (BLOCKER)

1. Go to https://www.npmjs.com/org/create
2. Create the `godspeedai` organization
3. Generate an automation token with publish access to `@godspeedai` scope
4. Update the npm token in SOPS-encrypted secrets (used by `.github/actions/decrypt-secrets` as `npm-token`)

### Step 2: Merge PR #87

```bash
gh pr merge 87 --squash
```

### Step 3: Delete old tags and recreate from new main

The old tags point to commit `7137599` which has the stub deploy.yml. They must be recreated from the new main HEAD which has the correct routing deploy.yml.

```bash
# Delete remote tags
git push origin :refs/tags/sea-core-v0.12.0 :refs/tags/sea-dsl-v0.12.0 :refs/tags/sea-v0.12.0

# Delete local tags
git tag -d sea-core-v0.12.0 sea-dsl-v0.12.0 sea-v0.12.0

# Pull latest main
git checkout main && git pull

# Recreate tags from new main HEAD
git tag sea-core-v0.12.0 main
git tag sea-dsl-v0.12.0 main
git tag sea-v0.12.0 main

# Push tags (triggers deploy.yml -> publish workflows)
git push origin sea-core-v0.12.0 sea-dsl-v0.12.0 sea-v0.12.0
```

### Step 4: Verify publishes

After pushing tags, three Deploy workflow runs should fire. Monitor:

```bash
gh run list --workflow=deploy.yml --limit 3
```

Each should route to:
- `sea-core-v0.12.0` -> `release-crates.yml` -> crates.io
- `sea-dsl-v0.12.0` -> `release-pypi.yml` -> PyPI
- `sea-v0.12.0` -> `release-npm.yml` -> npm (@godspeedai/domainforge + @godspeedai/domainforge-wasm)

## Key Files

| File | Purpose |
|------|---------|
| `release-please-config.json` | Release-please config with linked-versions plugin |
| `.release-please-manifest.json` | Version manifest (all at 0.12.0) |
| `.github/workflows/deploy.yml` | Tag router: parses component from tag, dispatches to publish workflow |
| `.github/workflows/release-crates.yml` | Publishes sea-core to crates.io |
| `.github/workflows/release-pypi.yml` | Publishes sea-dsl wheels to PyPI via maturin |
| `.github/workflows/release-npm.yml` | Publishes @godspeedai/domainforge (napi) and @godspeedai/domainforge-wasm to npm |
| `.github/actions/decrypt-secrets` | SOPS-based secret decryption (npm-token, pypi-api-token, cargo-registry-token) |

## Verification Checklist

- [ ] `@godspeedai` npm org created on npmjs.com
- [ ] npm automation token generated with `@godspeedai` publish access
- [ ] Token updated in SOPS secrets as `npm-token`
- [ ] PR #87 merged to main
- [ ] Old tags deleted from remote
- [ ] New tags created from main HEAD and pushed
- [ ] Deploy workflow runs succeed for all 3 tags
- [ ] `sea-core` 0.12.0 visible on https://crates.io/crates/sea-core
- [ ] `sea-dsl` 0.12.0 visible on https://pypi.org/project/sea-dsl/
- [ ] `@godspeedai/domainforge` 0.12.0 visible on npm
- [ ] `@godspeedai/domainforge-wasm` 0.12.0 visible on npm

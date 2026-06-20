---
name: release-management
description: Use when cutting, shipping, promoting, diagnosing, or previewing DomainForge releases, release-please PRs, component tags, deploy triggers, or release readiness.
---

# Release Management

## Overview

DomainForge releases use release-please manifest mode on `main`, independent
per-package versions, component-prefixed tags, and a tag-triggered deploy
workflow that dispatches to existing publish workflows. All protected-branch
checks on `main` must pass.

## Current Release Model

| Package key | Component | Tag | Layout |
|---|---|---|---|
| `domainforge-core` | `domainforge-core` | `domainforge-core-vX.Y.Z` | `domainforge-core/` |
| `domainforge-python` | `domainforge` | `domainforge-vX.Y.Z` | `domainforge-python/` |
| `domainforge-typescript` | `domainforge-typescript` | `domainforge-typescript-vX.Y.Z` | `domainforge-typescript/` |

Release PRs are opened or updated by `.github/workflows/release-please.yml`
after pushes to `main`. Merging a release-please PR creates the matching
component tag. Component tags trigger `.github/workflows/deploy.yml`, which
parses the tag and dispatches to the appropriate reusable publish workflow:

- `domainforge-core-v*` → `release-crates.yml` (crates.io)
- `domainforge-v*` → `release-pypi.yml` (PyPI wheels, multi-platform matrix)
- `domainforge-typescript-v*` → `release-npm.yml` (npm + WASM)

There is intentionally no separate `prod` GitHub environment gate. This is a
solo-maintainer repository: CI on `main` is the merge gate, and a component
tag push is the publish signal. The publish workflows handle their own
idempotency (`--skip-existing`, `npm view` checks, cargo "already published"
detection).

Treat older `v*.*.*` release scripts and `docs/RELEASE_PROCESS.md` as legacy
unless the user explicitly asks for that path.

## Hard Rules

1. Never hand-edit changelogs, manifests, or package versions for a release.
   release-please owns those files.
2. Never push release tags manually. Tags come from merging release-please PRs.
3. Always run a dry-run before saying what a release will contain.
4. Check commit scopes before blaming release-please. Invalid scopes can place
   changes in the wrong package or hide them from a component changelog.
5. Do not assume a `prod` environment or branch exists. This repo only has
   `main`. Tag push → publish workflow → registry.

## Quick Commands

| Need | Command |
|---|---|
| Preview release plan | `.codex/skills/release-management/scripts/release-dryrun.sh` |
| Required checks on a PR | `gh pr checks <number>` |
| Release PRs | `gh pr list --search "release in:title" --state open` |
| Tags on origin | `git ls-remote --tags origin 'refs/tags/*-v*'` |
| Deploy runs | `gh run list --workflow deploy.yml --limit 10` |

## Workflows

### What will the next release contain?

Run the dry-run script from the repo root. Report each component's current
version, proposed version, bump reason, and changelog delta. Do not commit.

### Cut or ship a release

1. Confirm `main` is green with the latest required checks.
2. Run the dry-run and summarize proposed component releases.
3. Confirm release PRs exist. If missing, inspect `release-please.yml` runs and
   `CREATE_PR_TOKEN`; `GITHUB_TOKEN` cannot trigger downstream tag workflows.
4. If the user explicitly authorizes merging, merge the release PR. Otherwise
   tell the user which release PR to merge.
5. After merge, verify the component tag exists and the expected publish
   workflow (`release-crates.yml` / `release-pypi.yml` / `release-npm.yml`)
   started via `deploy.yml`.

### Publish did not trigger

Check in order:

1. Tag exists: `git ls-remote --tags origin 'refs/tags/*-v*'`.
2. Tag matches `deploy.yml`: `domainforge-core-v*`, `domainforge-v*`, or `domainforge-typescript-v*`.
3. Tag came from release-please using `CREATE_PR_TOKEN`, not `GITHUB_TOKEN`.
4. `deploy.yml` has not been disabled and Actions permissions are healthy.
5. The dispatchable publish workflow file exists and is not disabled.

### Add a package to release-please

1. Add package config to `release-please-config.json`.
2. Add current version to `.release-please-manifest.json`.
3. Create a directory at repo root named after the package key (release-please
   resolves file paths as `<package-key>/<file>`).
4. Move the package's version file (pyproject.toml, package.json, Cargo.toml)
   into that directory with relative paths adjusted to reach `domainforge-core/`.
5. Add a `deploy-<component>` job to `deploy.yml` that dispatches to the
   appropriate publish workflow, gated by `if: needs.identify.outputs.component == '<component>'`.
6. Add the component to `commitlint.config.cjs` scope-enum (if you maintain one).
7. Update `docs/governance.md`.
8. Run the dry-run and focused workflow validation.

## Common Mistakes

| Mistake | Correction |
|---|---|
| Looking for `dev`, `stage`, or `prod` branches | Use `main`; this repo has only `main`. Tags trigger publish directly. |
| Looking for `prod` environment gate | None exists. Publish workflows are dispatched by `deploy.yml` on tag push. |
| Asking for approval before merge | Required approvals are `0`; required checks still block merge. |
| Manually creating tags to unblock publish | Fix release-please or merge the release PR. |
| Treating old local release scripts as canonical | Use release-please unless the user requests legacy scripts. |
| Putting package version files at repo root | release-please requires them under `<package-key>/`. Root-level packages must have a directory. |
| Treating `deploy.yml` as a builder | `deploy.yml` is a router. The publish workflows own build + publish logic. |

## References

- Config: `release-please-config.json`, `.release-please-manifest.json`
- Router: `.github/workflows/deploy.yml`
- Publish workflows: `.github/workflows/release-crates.yml`, `release-pypi.yml`, `release-npm.yml`
- Release-please trigger: `.github/workflows/release-please.yml`
- Governance: `docs/governance.md`
- Commit rules: `commitlint.config.cjs`

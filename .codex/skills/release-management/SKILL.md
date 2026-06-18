---
name: release-management
description: Use when cutting, shipping, promoting, diagnosing, or previewing DomainForge releases, release-please PRs, component tags, deploy triggers, prod approval gates, or release readiness.
---

# Release Management

## Overview

DomainForge releases use release-please manifest mode on `main`, independent
per-package versions, component-prefixed tags, and a tag-triggered deploy
workflow. All protected-branch checks on `main` must pass.

## Current Release Model

| Package key | Component | Tag |
|---|---|---|
| `sea-core` | `sea-core` | `sea-core-vX.Y.Z` |
| `sea-dsl` | `sea-dsl` | `sea-dsl-vX.Y.Z` |
| `sea-typescript` | `sea` | `sea-vX.Y.Z` |

Release PRs are opened or updated by `.github/workflows/release-please.yml`
after pushes to `main`. Merging a release-please PR creates the matching
component tag. Component tags start `.github/workflows/deploy.yml`; `prod`
waits for the GitHub environment approval by `SPRIME01`.

Treat older `v*.*.*` release scripts and `docs/RELEASE_PROCESS.md` as legacy
unless the user explicitly asks for that path.

## Hard Rules

1. Never hand-edit changelogs, manifests, or package versions for a release.
   release-please owns those files.
2. Never push release tags manually. Tags come from merging release-please PRs.
3. Never bypass the `prod` environment gate.
4. Always run a dry-run before saying what a release will contain.
5. Check commit scopes before blaming release-please. Invalid scopes can place
   changes in the wrong package or hide them from a component changelog.

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
5. After merge, verify the component tag exists and `deploy.yml` started.

### Promote to prod or explain a paused deploy

Stage deploy is automatic after the component tag. Prod waits on the `prod`
environment approval and 60-second timer. If the run is paused for approval,
surface that state; do not bypass it.

### Deploy did not trigger

Check in order:

1. Tag exists: `git ls-remote --tags origin 'refs/tags/*-v*'`.
2. Tag matches `deploy.yml`: `sea-core-v*`, `sea-dsl-v*`, or `sea-v*`.
3. Tag came from release-please using `CREATE_PR_TOKEN`, not `GITHUB_TOKEN`.
4. `deploy.yml` has not been disabled and Actions permissions are healthy.

### Add a package to release-please

1. Add package config to `release-please-config.json`.
2. Add current version to `.release-please-manifest.json`.
3. Add a component tag trigger to `.github/workflows/deploy.yml`.
4. Add the component to `commitlint.config.cjs` `scope-enum`.
5. Update `docs/governance.md`.
6. Run the dry-run and focused workflow validation.

## Common Mistakes

| Mistake | Correction |
|---|---|
| Looking for `dev` or `stage` branches | Use `main`; `stage` is a deploy job, not a branch or environment. |
| Asking for approval before merge | Required approvals are `0`; required checks still block merge. |
| Manually creating tags to unblock deploy | Fix release-please or merge the release PR. |
| Treating old local release scripts as canonical | Use release-please unless the user requests legacy scripts. |

## References

- Config: `release-please-config.json`, `.release-please-manifest.json`
- Workflows: `.github/workflows/release-please.yml`, `.github/workflows/deploy.yml`
- Governance: `docs/governance.md`
- Commit rules: `commitlint.config.cjs`

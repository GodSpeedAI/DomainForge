# Repository Governance

Branch protection, deployment environments, and release routing for DomainForge.

## Branch Protection: `main`

`main` is the only protected branch. All changes must arrive via pull request.

| Rule | Setting |
|---|---|
| Pull request required | Yes |
| Required approvals | 1 |
| Dismiss stale reviews | Yes |
| Require code owner reviews | No |
| Enforce for admins | Yes |
| Linear history | Required (squash-merge or rebase-merge; no merge commits) |
| Force pushes | Blocked |
| Branch deletions | Blocked |
| Branch must be up to date | Yes (`strict: true`) |

### Required status checks

A PR cannot merge until all of these pass:

| Check | Source workflow | Job |
|---|---|---|
| `Lint & Format` | `ci.yml` | `lint` |
| `Test Rust (ubuntu-latest)` | `ci.yml` | `test-rust` matrix leg |
| `Test Rust (macos-latest)` | `ci.yml` | `test-rust` matrix leg |
| `Test Rust (windows-2025-vs2026)` | `ci.yml` | `test-rust` matrix leg |
| `Test Python (3.11)` | `ci.yml` | `test-python` matrix leg |
| `Test Python (3.12)` | `ci.yml` | `test-python` matrix leg |
| `Test TypeScript` | `ci.yml` | `test-typescript` |
| `Minimal Integration Check` | `ci.yml` | `test-integration` |
| `Test WASM` | `ci.yml` | `test-wasm` |
| `Security Audit` | `ci.yml` | `security` (`cargo audit`, blocking) |
| `dependency-review` | `dependency-review.yml` | `dependency-review` |

### What this means for contributors

- **No direct pushes to `main`.** Push to a feature branch and open a PR.
- **PR must be up to date** with `main` before merging. Rebase or merge `main` into your branch if CI fails on staleness.
- **Linear history required.** Use squash-merge or rebase-merge when merging PRs. Do not use create-merge-commit.
- **Admins are not exempt.** The `enforce_admins` flag is on.

## Environments

### `prod`

Production deployment gate. Used by the `deploy-prod` job in `deploy.yml`.

| Rule | Setting |
|---|---|
| Required reviewer | `SPRIME01` |
| Wait timer | 60 seconds |
| Deployment branch policy | Protected branches only |

When a tag matching a component prefix (e.g. `sea-core-v0.12.0`) is pushed, the `deploy-prod` job pauses until `SPRIME01` approves the deployment review and the 60-second timer elapses.

### `stage` (not configured)

The `deploy-stage` job in `deploy.yml` runs build and smoke tests without a GitHub environment gate. There is no `stage` environment in repo settings. This is intentional: stage is a build-and-verify waypoint, not a governance boundary.

## Deploy Pipeline

```
tag push (sea-core-v*, sea-dsl-v*, sea-v*)
  │
  ▼
deploy.yml
  ├── identify    (parse tag → component + version)
  ├── deploy-stage (build, smoke test, fail-closed)
  └── deploy-prod  (needs: identify + deploy-stage, environment: prod, reviewer gate)
```

Tag shape: `<component>-v<version>` where component is one of `sea-core`, `sea-dsl`, `sea`.

The deploy jobs contain `TODO` placeholders for the real build and deploy commands. They do not publish to registries (PyPI, npm, crates.io) — that is handled separately by the `release.yml` build-and-publish chain triggered by `v*.*.*` tags.

## Release Routing (release-please)

`release-please-config.json` and `.release-please-manifest.json` configure independent per-package versioning. Each package gets its own release PR and component-prefixed tag.

| Package | Component | Release type | Tag form |
|---|---|---|---|
| `sea-core` (Rust) | `sea-core` | rust | `sea-core-vX.Y.Z` |
| `sea-dsl` (Python) | `sea-dsl` | python | `sea-dsl-vX.Y.Z` |
| `@domainforge/sea` (TypeScript) | `sea` | node | `sea-vX.Y.Z` |

### Known issue: release-please trigger

`.github/workflows/release-please.yml` currently triggers on `push: branches: [dev]`. The `dev` branch was retired and no longer exists on origin. As a result, release-please will not fire until the trigger is updated.

To activate release-please, either:
1. Change the trigger to `branches: [main]` and move the release-please workflow to run on the protected branch.
2. Reactivate `dev` as a dedicated integration branch that feeds release-please.

## Conventional Commits Enforcement

`commitlint` + `lefthook` enforce conventional commit format on every commit via a `commit-msg` hook.

- Config: `commitlint.config.cjs`
- Hook: `lefthook.yml` → `commit-msg` → `npx commitlint --edit {1}`

### Required format

```
<type>(<scope>): <subject>
```

- `type` must be a conventional type (`feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `build`, `ci`, `perf`, `revert`, etc.)
- `scope` is **required** and must be one of 29 allowed values (see `commitlint.config.cjs` for the full list)
- `subject` must be non-empty

### Allowed scopes

**Release-please components:** `sea-core`, `sea-dsl`, `sea`

**Meta:** `ci`, `deps`, `repo`

**Historical (from commit history):** `bindings`, `calm`, `cli`, `code`, `core`, `docs`, `format`, `grammar`, `graph`, `new_docs`, `parser`, `pkg`, `pyo3`, `policy`, `python`, `registry`, `release`, `semantic-pack`, `authority`, `test`, `tests`, `units`, `wasm`

### Bypass

The hook is local-only (lefthook installs to `.git/hooks/`). CI does not run commitlint. To bypass locally, use `git commit --no-verify` (discouraged).

## Secrets

| Secret | Used by | Purpose |
|---|---|---|
| `GITHUB_TOKEN` | All workflows | Automatic GitHub API token (cannot trigger downstream workflows) |
| `CREATE_PR_TOKEN` | `release-please.yml`, `prepare-release.yml` | PAT for creating PRs and tags that trigger downstream workflows |
| `SOPS_AGE_KEY` | `release-pypi.yml`, `release-npm.yml`, `release-crates.yml` | Decrypts `secrets/secrets.yaml` via sops |
| `PYPI_API_TOKEN` | `release-pypi.yml` | PyPI publishing fallback (primary token is sops-decrypted) |

## See Also

- [Release Process](./RELEASE_PROCESS.md) — How to cut a release
- [Workflows](../.github/workflows/README.md) — CI/CD workflow documentation
- [Contributing](../CONTRIBUTING.md) — Developer setup and PR process

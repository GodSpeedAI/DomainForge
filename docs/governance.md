# Repository Governance

Branch protection, deployment environments, and release routing for DomainForge.

## Branch Protection: `main`

`main` is the only protected branch. All changes must arrive via pull request.

| Rule | Setting |
|---|---|
| Pull request required | Yes |
| Required approvals | 0 |
| Dismiss stale reviews | Yes |
| Require code owner reviews | No |
| Enforce for admins | Yes |
| Linear history | Required (squash-merge or rebase-merge; no merge commits) |
| Force pushes | Blocked |
| Branch deletions | Blocked |
| Branch must be up to date | Yes (`strict: true`) |

### Required status checks

A PR cannot merge until all of these pass. Approvals are not required because
this is currently a solo-maintainer repository; CI remains the merge gate.

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

This repository does not use GitHub deployment environments. There is no
`stage` or `prod` environment gate.

Publishing is triggered directly by component tag pushes. The publish
workflows (`release-crates.yml`, `release-pypi.yml`, `release-npm.yml`)
handle their own idempotency:
- crates.io: detects "already published" from cargo output
- PyPI: `--skip-existing` flag on maturin
- npm: `npm view <pkg>@<version>` pre-check

This matches the solo-maintainer model: CI on `main` is the merge gate, and
a component tag push is the publish signal.

## Deploy Pipeline

```
tag push (domainforge-core-v*, domainforge-v*, domainforge-typescript-v*)
  │
  ▼
deploy.yml (router)
  ├── identify    (parse tag → component + version)
  └── dispatch to publish workflow based on component:
      ├── domainforge-core → release-crates.yml  (crates.io)
      ├── domainforge      → release-pypi.yml    (PyPI, multi-platform matrix)
      └── domainforge-typescript → release-npm.yml     (npm + WASM)
```

Tag shape: `<component>-v<version>` where component is one of `domainforge-core`, `domainforge`, `domainforge-typescript`.

`deploy.yml` does NOT contain build or publish logic itself. It dispatches to
the same reusable publish workflows that the legacy `release.yml` uses. This
avoids duplication and keeps a single source of truth for each registry's
publish process.

## Release Routing (release-please)

`release-please-config.json` and `.release-please-manifest.json` configure independent per-package versioning. Each package gets its own release PR and component-prefixed tag.

| Package | Component | Release type | Tag form |
|---|---|---|---|
| `domainforge-core` (Rust) | `domainforge-core` | rust | `domainforge-core-vX.Y.Z` |
| `domainforge` (Python) | `domainforge` | python | `domainforge-vX.Y.Z` |
| `domainforge` (TypeScript) | `domainforge-typescript` | node | `domainforge-typescript-vX.Y.Z` |

### Release-please trigger

`.github/workflows/release-please.yml` triggers on `push: branches: [main]` and opens/updates release PRs for each configured package.

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

**Release-please components:** `domainforge-core`, `domainforge`, `domainforge-typescript`

**Meta:** `ci`, `deps`, `repo`

**Historical (from commit history):** `bindings`, `calm`, `cli`, `code`, `core`, `docs`, `format`, `grammar`, `graph`, `new_docs`, `parser`, `pkg`, `pyo3`, `policy`, `python`, `registry`, `release`, `semantic-pack`, `authority`, `test`, `tests`, `units`, `wasm`

**Note:** commitlint uses `scope-case: kebab-case` enforcement only (no static allowlist). The components above reflect the release-please package keys and historical usage.

### Bypass

The hook is local-only (lefthook installs to `.git/hooks/`). CI does not run commitlint. To bypass locally, use `git commit --no-verify` (discouraged).

## Package Layout

release-please v4 manifest mode resolves file paths as `<package-key>/<file>`.
Each package's version file must live in a directory matching its key:

| Package | Directory | Version file |
|---|---|---|
| `domainforge-core` | `domainforge-core/` | `Cargo.toml` |
| `domainforge` | `domainforge-python/` | `pyproject.toml` |
| `domainforge-typescript` | `domainforge-typescript/` | `package.json` |

The root `package.json` is a private workspace root for dev dependencies
(commitlint, lefthook, vitest). It is NOT published. The root `Cargo.toml`
defines the Rust workspace; `domainforge-core/Cargo.toml` is the sole published crate.

## Secrets

| Secret | Used by | Purpose |
|---|---|---|
| `GITHUB_TOKEN` | All workflows | Automatic GitHub API token (cannot trigger downstream workflows) |
| `CREATE_PR_TOKEN` | `release-please.yml`, `prepare-release.yml` | PAT for creating PRs and tags that trigger downstream workflows |
| `SOPS_AGE_KEY` | `release-pypi.yml`, `release-npm.yml`, `release-crates.yml` | Decrypts `secrets/secrets.yaml` via sops |
| `PYPI_API_TOKEN` | `release-pypi.yml` | PyPI publishing fallback (primary token is sops-decrypted) |

## Tool Versions

| Tool | Required version | Verification |
|---|---|---|
| `release-please-action` | `v4.4.1`, pinned in `.github/workflows/release-please.yml` by commit SHA | Inspect the workflow `uses:` line |
| `@commitlint/cli` | `^21.0.2`, installed from `package.json` | `npx commitlint --version` |
| `@commitlint/config-conventional` | `^21.0.2`, installed from `package.json` | `npm ls @commitlint/config-conventional` |
| `lefthook` | `^2.1.9`, installed from `package.json` | `npx lefthook version` |

Keep commitlint packages on the same major version. Lefthook v2 syntax is used
in `lefthook.yml`; do not downgrade to v1 without testing hook compatibility.

## See Also

- [Release Process](./RELEASE_PROCESS.md) — How to cut a release
- [Workflows](../.github/workflows/README.md) — CI/CD workflow documentation
- [Contributing](../CONTRIBUTING.md) — Developer setup and PR process

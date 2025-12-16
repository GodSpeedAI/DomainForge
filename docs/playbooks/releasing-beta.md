# Releasing Beta

This playbook describes the process for releasing a new beta version of DomainForge.

## Prerequisites

- Clean git working directory.
- Passing CI (`just all-tests`).
- Write access to crates.io, PyPI, and NPM.

## Steps

1. **Version Bump**:

> **Recommended**: Use `just release-preview minor` to preview changes first, then `just prepare-release minor` to apply them. See [Local Release Preparation](./local-release-preparation.md) for details.

> **Automated (GitHub)**: Use the `prepare-release.yml` workflow (Actions → Prepare Release → Run workflow) to bump all versions and create a PR automatically.

If bumping manually, update version in all files (they must stay in sync):

- `sea-core/Cargo.toml` (source of truth)
- `pyproject.toml`
- `package.json`

> Note: WASM `pkg/package.json` is auto-generated from `sea-core/Cargo.toml` during build.

1. **Changelog**:

Update `CHANGELOG.md` with new features and breaking changes.

1. **Build & Test**:

```bash
just setup
just all-tests
```

1. **Git Tag**:

```bash
git tag -a v0.x.0 -m "Release v0.x.0"
git push origin v0.x.0
```

1. **Publish Rust Core**:

```bash
cargo publish -p sea-core
```

1. **Publish Python**:

```bash
maturin publish
```

1. **Publish TypeScript**:

```bash
npm publish
```

1. **GitHub Release**:

Draft a new release on GitHub using the changelog notes.

If a critical bug is found immediately:

- **Rust**: Yank the crate (`cargo yank --vers <version>`).

- **NPM**: Deprecate the package version and suggest replacement:

```bash
npm deprecate @domainforge/sea@0.x.0 "Critical security bug; use @domainforge/sea@0.x.1"
```

- **PyPI**: Yank/unyank through standard tooling (e.g., `twine` or `maturin publish --yank`) or publish a hotfix patch:

```bash
# Publish a hotfix
git checkout -b hotfix/0.x.1
bumpversion patch
git push origin hotfix/0.x.1
# Create release and upload via twine/maturin
maturin publish --skip-existing
```

# Release Plan: v0.1.0 — DomainForge (SEA DSL)

This document outlines the release plan, publishing order, and best practices for publishing the DomainForge artifacts across ecosystems (crates.io, PyPI, npm, and GitHub). Treat this plan as a canonical checklist for future releases.

## Goals

- Publish a single, canonical release (v0.1.0) across all publish targets.
- Maintain cross-language parity and consistent versioning across Rust/Python/TypeScript/WASM packages.
- Provide a reproducible, documented release process suitable for CI automation.

## Versioning & Manifests

- Ensure version parity across:
  - `sea-core/Cargo.toml` (Rust)
  - `pyproject.toml` (Python - sea-dsl)
  - `package.json` (TypeScript - @domainforge/sea)
  - `pkg/package.json` (WASM - @domainforge/sea-wasm)
- Choose a semantic version (e.g., 0.1.0). All manifests must match before tagging.
- Where appropriate, add a `CHANGELOG.md` entry summarizing changes in v0.1.0.

## Pre-release Checklist

1. Confirm tests pass locally and in CI:
   - `just rust-test`
   - `just python-test`
   - `just ts-test`
2. Ensure the `README.md` no longer contains roadmap or "coming soon" statements; replace with instructions that reflect the current release state.
3. Clean up repository: remove or secure plaintext secrets; ensure `secrets/secrets.yaml` is encrypted.
4. Create or update `docs/plans/release-plan.md` (this file) and commit.
5. Ensure `pyproject.toml`, `sea-core/Cargo.toml`, and `package.json` set the new version.
6. Add or update `CHANGELOG.md` and verify the package metadata (`license`, `authors`, `keywords`).
7. Run a release smoke test locally by building all artifacts (see below).

## Building & Verifying Artifacts Locally

- Rust:

  ```bash
  cargo build -p sea-core --release
  cargo test -p sea-core
  ```

- Python:

  ```bash
  .venv/bin/python -m maturin build --release -o dist
  .venv/bin/pip install --index-url https://test.pypi.org/simple --extra-index-url https://pypi.org/simple --force-reinstall dist/sea_dsl-<version>.whl
  python -c "import sea_dsl; print(sea_dsl.__version__)"
  ```

- TypeScript/NAPI:

  ```bash
  npm run build # or `just ts-build` (use local binding to `napi-rs` build)
  npm pack
  npm pack --pack-destination ./dist
  ```

- WASM (pkg/):

  ```bash
  cd pkg
  npm pack
  npm pack --pack-destination ../dist
  ```

## Publish Order & Rationale

1. **Publish Rust core crate to crates.io** (`cargo publish`) — optional but recommended if Rust users will consume `sea-core` directly.
   - Auth: `CARGO_REGISTRY_TOKEN` set in CI or environment.
2. **Publish WASM package** (`@domainforge/sea-wasm`) to npm (pkg/)
   - Auth: `NPM_TOKEN` (or CI secret).
   - Reason: Browser & edge consumers rely on the WASM artifact.
3. **Publish TypeScript bindings** (`@domainforge/sea`) to npm
   - Auth: `NPM_TOKEN`.
   - Reason: Node consumers and the TypeScript API should be available for consumption.
4. **Publish Python** (`sea-dsl`) to PyPI (maturin/publish)
   - Auth: `PYPI_API_TOKEN` or `PYPI_TEST_API_TOKEN` for TestPyPI.
   - Reason: Python package distribution typically bundles native wheels; publish last so any native artifacts are stable.
5. **Tag and GitHub release**: Create a signed tag `v0.1.0`, push it to the repo, and create a GitHub Release with the changelog and attachments.
   - Use the `gh` CLI to create a release and attach distributables if needed.

> Note: The Rust crate's release is optional for downstream build targeting; the Python and TypeScript artifacts bundle the needed native binaries themselves.

## Publishing Commands (CI-friendly)

- Rust:

  ```bash
  cargo publish --manifest-path sea-core/Cargo.toml
  ```

- WASM (pkg/):

  ```bash
  cd pkg
  npm publish --access public
  ```

- TypeScript (root):

  ```bash
  npm publish --access public
  ```

- Python (maturin):

  ```bash
  MATURIN_PYPI_TOKEN="$PYPI_API_TOKEN" maturin publish --repository-url https://upload.pypi.org/legacy --non-interactive
  ```

## GitHub Release

- Tag locally and push:

  ```bash
  git tag -a v0.1.0 -m "Release v0.1.0"  # optionally sign with -s
  git push origin v0.1.0
  ```

- Create release using `gh` (GitHub CLI):

  ```bash
  gh release create v0.1.0 --title "v0.1.0" --notes-file CHANGELOG.md
  ```

## Post-Release

- Update `README.md` to show how to install packages from the package registries.
- Update any docs that previously said "coming soon".
- Verify published packages are available:
  - Rust: <https://crates.io/crates/sea-core>
  - PyPI: <https://pypi.org/project/sea-dsl/>
  - npm: <https://www.npmjs.com/package/@domainforge/sea>
  - WASM package page
- Create release notes and highlight breaking changes (if any).

## Rollback Plan

1. For crates.io: Follow crates.io unpublishing policies; prefer yanking vs full-undo.
2. For PyPI/NPM: Versions cannot be re-used; republish a new patch version if needed.
3. Re-tag in GitHub and push a new release in case of an urgent fix.

## CI and Automation Notes

- Ensure CI secrets are configured in GitHub Actions for `CARGO_REGISTRY_TOKEN`, `PYPI_API_TOKEN`, and `NPM_TOKEN`.
- Use `npm ci` in CI and `--non-interactive` flags for scripts.
- Keep a `release.yml` workflow that runs on `release/` branch or manual dispatch.

## Checklist

- [ ] Confirm version numbers
- [ ] Run all tests and checks in CI
- [ ] Update `README.md` to remove roadmap statements
- [ ] Update `CHANGELOG.md` entries
- [ ] Create local tag and push
- [ ] Publish artifacts in order
- [ ] Create GitHub Release and attach artifacts
- [ ] Post-release verification

---

*Documentation maintained by the release manager.*

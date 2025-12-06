# Releasing Beta

This playbook describes the process for releasing a new beta version of DomainForge.

## Prerequisites

- Clean git working directory.
- Passing CI (`just all-tests`).
- Write access to crates.io, PyPI, and NPM.

## Steps

1. **Version Bump**:
   Update version in:
   - `Cargo.toml` (workspace members)
   - `pyproject.toml`
   - `package.json`
   - `sea-core/Cargo.toml`

2. **Changelog**:
   Update `CHANGELOG.md` with new features and breaking changes.

3. **Build & Test**:

   ```bash
   just setup
   just all-tests
   ```

4. **Publish Rust Core**:

   ```bash
   cargo publish -p sea-core
   ```

5. **Publish Python**:

   ```bash
   maturin publish
   ```

6. **Publish TypeScript**:

   ```bash
   npm publish
   ```

7. **Git Tag**:

   ```bash
   git tag -a v0.x.0 -m "Release v0.x.0"
   git push origin v0.x.0
   ```

8. **GitHub Release**:
   Draft a new release on GitHub using the changelog notes.

## Rollback Plan

If a critical bug is found immediately:

- **Rust**: Yank the crate (`cargo yank`).
- **NPM/PyPI**: Deprecate the version or publish a patch immediately.

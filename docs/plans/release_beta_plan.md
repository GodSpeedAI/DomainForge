# Release & Packaging Plan (Alpha → Beta)

## Goals
- Ship reproducible artifacts for CLI, Python, TypeScript, and WASM.
- Keep behavior consistent with Rust core as canonical source.
- Reduce friction for downstream integrators via prebuilt packages.

## Target Channels
- **CLI (Rust)**: crates.io and GitHub Release binaries (Linux/macOS/Windows).
- **Python**: PyPI wheels (manylinux, macOS, Windows) built with maturin.
- **TypeScript/Node**: npm package wrapping napi `.node` builds for major triples.
- **WASM**: wasm-pack output published alongside npm (web target).

## Requirements to Ship Beta
1) **Secrets/Access**
   - Configure CI secrets: `CRATES_IO_TOKEN`, `PYPI_API_TOKEN`, `NPM_TOKEN`, signing keys if desired.
2) **Versioning/Tagging**
   - Adopt semver and tag `v0.1.x` for beta; keep changelog entries aligned.
3) **CI Validation**
   - Ensure `.github/workflows/ci.yml` runs fmt, clippy, Rust/Python/TS tests on PRs.
   - Keep WASM size checks in place; add npm pack smoke if possible.
4) **Release Automation**
   - Validate `release.yml` with a dry-run tag to confirm CLI artifacts and wheels upload.
   - Validate `publish-python.yml` for PyPI upload (skip without token).
   - Add npm publish step (existing CI builds bindings; hook `npm publish` gated on `NPM_TOKEN`).
5) **Docs/Comms**
   - Update README with install commands for crates.io/npm/pip once live.
   - Add release notes covering module import/export fixes and cross-language parity.

## Open Actions
- Wire secrets in GitHub repository settings.
- Add npm publish job (can extend `release.yml` or a dedicated workflow).
- Decide on binary signing and artifact retention policy.
- Run a tagged pre-release (e.g., `v0.1.0-beta.1`) to exercise the full pipeline.

# scripts — Build, Release, CI, and Proof Automation

## Overview
Repository automation orchestrated by the `just` CLI: language-binding builds,
version bumping, changelog/release-note generation, git tagging, GitHub release
creation, CI helper utilities, workflow linters, and the Rust debug-binary
preparation helpers. The proof and fixture-verification suites live in the
`prove/` and `verify/` children.

## Interface
- `[build-python.sh:maturin develop]` — creates/activates `.venv`, installs
  `maturin`/`pytest` if absent, then runs `maturin develop` from
  `domainforge-python/` (the pyproject must be resolved from that directory).
- `[build-typescript.sh:npm run build]` — `cd domainforge-typescript && npm run
  build` (the napi config and `--cargo-cwd ../domainforge-core` live there).
- `[build-wasm.sh:wasm-pack]` — builds `--target web --release --features wasm`
  into `target/wasm-pkg`, optionally `wasm-opt -Oz`, patches `package.json`
  (`name=domainforge-wasm`, version from `domainforge-core/Cargo.toml`), gzips, and
  reports bundle size against a 500 KB gzip target.
- `[optimized-wasm-build.sh:wasm-pack build]` — production WASM build with an
  explicit `wasm-opt -Oz` verification pass and a 2 MB `MAX_SIZE` warning
  threshold.
- `[build-release.sh:usage]` — `--dry-run`, `--skip-cli`, `--skip-python`,
  `--skip-wasm`; builds a CLI archive, Python wheel (`maturin build --release
  --features python`), and WASM bundle into `dist/`, then writes
  `SHA256SUMS.txt`.
- `[bump-version.sh:usage]` — `major|minor|patch|X.Y.Z`; rewrites the version in
  `domainforge-core/Cargo.toml`, `domainforge-python/pyproject.toml` (via `sed`)
  and `domainforge-typescript/package.json` (via `jq`), then commits unless
  `--no-commit`/`--dry-run`.
- `[create-tag.sh:git tag]` — creates annotated tag `v{VERSION}` from
  `RELEASE_NOTES.md` (or a default message), with `-s` when `user.signingkey` is
  configured; `--force` deletes and recreates; never pushes.
- `[create-github-release.sh:gh release create]` — requires `gh`, auth, and an
  existing `v{VERSION}` tag; uploads every file in `dist/`, marks pre-release when
  the version matches `(alpha|beta|rc|pre)`, supports `--draft`.
- `[generate-changelog.sh:ENTRY]` — parses conventional commits since the last tag
  into `Added`/`Changed`/`Fixed`/`Documentation` sections and inserts the entry
  above the first `## [` heading in `CHANGELOG.md`.
- `[generate-release-notes.sh:NOTES]` — extracts the `CHANGELOG.md` section for the
  version, adds `BREAKING CHANGE` commits and contributors, writes
  `RELEASE_NOTES.md`.
- `[pre-release-check.sh:usage]` — `--dry-run`, `--verbose`, `--skip-tests`;
  checks git status, branch, upstream freshness, version sync across
  `domainforge-core/Cargo.toml` / `pyproject.toml` / `package.json`, the
  `CHANGELOG.md` entry, `just all-tests`, and a release `cargo build`.
- `[release.sh:run_script]` — 9-step orchestrator (pre-release checks → bump →
  changelog → release notes → commit → tag → build → push → GitHub release); flags
  `--dry-run`, `--skip-tests`, `--skip-build`, `--yes/-y`.
- `[test-all.sh:FAILED]` — runs `cargo test` on `domainforge-core/Cargo.toml`,
  `pytest tests/` (building Python bindings if missing), and `npm test`, then exits
  `1` if any suite failed.
- `[ci_tasks.py:main]` — argparse subcommands `check-size`, `package`,
  `verify-cli`, `unpack-verify`.
- `[check_unused_test_imports.py:USE_RE]` — heuristic reporter for single-identifier
  `use` statements in `domainforge-core/tests/*.rs` that are never referenced again.
- `[resolve_rust_binary.py:find_binary]` — locates a compiled Rust binary under
  `target/` or `domainforge-core/target/` by exact/prefix/contains match with an
  optional fallback mode; prints the absolute path.
- `[patch_napi_types.py:PATCHES]` — idempotent post-`napi build` patcher for
  `index.d.ts` (`Self | null` → `NamespaceRegistry | null`, `function:` →
  `aggregateFunction:`).
- `[lint_release_security.py:lint_release_security]` — flags `curl | sh` without
  checksum verification, `continue-on-error` on publish steps, publish failures
  masked by `|| echo`/`|| true`, and secret tokens in build-step `env`.
- `[lint_release_workflows.py:lint_workflow]` — flags `continue-on-error` on
  publish steps, unverified `curl` downloads, `gh` calls in jobs without checkout
  that lack `GH_REPO`/`--repo`, and unfrozen `npm install` in release workflows.
- `[lint_workflow_gates.py:check_artifact_gates]` — asserts every
  `upload-artifact` job appears in `create-release.needs`, and that dependabot
  automerge does not treat an empty `[]` check result as success.
- `[check-sea-language-change.sh:WATCHED_PATTERN]` — CI lint gate: if the diff
  touches `domainforge-core/grammar/*.pest`, `ast(_schema|_convert)?.rs`,
  `printer.rs`, or `schemas/ast-*.schema.json`, the diff must also add an ADR
  (`docs/specs/ADR-*.md`) and a test or fixture.
- `[find_and_link_test_binary.sh:resolve_rust_binary.py]` — builds test binaries
  with `cargo test -p domainforge-core --no-run`, resolves the hashed binary via
  `resolve_rust_binary.py`, and symlinks (or copies) it to
  `target/debug/deps/sea_debug_test`.
- `[prepare_rust_debug.sh:update_launch_program.py]` — wraps
  `find_and_link_test_binary.sh`, then updates the `.vscode/launch.json`
  `Debug Rust Test (auto)` program path via `update_launch_program.py`.
- `[update_launch_program.py:main]` — `--program`, `--launch`
  (default `.vscode/launch.json`), `--config-name` (default
  `Debug Rust Test (auto)`); rewrites the path relative to `${workspaceFolder}`
  when inside the workspace, creates the file when missing, exits `1` on malformed
  JSON.
- `[clear_debug_test.sh:DEST]` — removes `target/debug/deps/sea_debug_test` if it
  is a link or file; exits `0` either way.
- `[run_commitlint.sh:run_lint]` — requires exactly one argument (the commit
  message file); prefers `./node_modules/.bin/commitlint`, then `bunx`, `npx`, and
  `npm exec`, failing with the `<type>(<scope>): <subject>` hint.
- Child contracts: `./prove/EGM.md` — `just prove` self-proving gates and evidence
  collection; `./verify/EGM.md` — baseline and per-projection-target verification
  gates.

## Invariants
- `[release.sh:CURRENT_BRANCH]` — the release orchestrator refuses to run unless the
  current branch is exactly `main`; release/`*` branches are only allowed for
  `pre-release-check.sh` preparation.
- `[pre-release-check.sh:CARGO_VERSION]` — all three manifest versions
  (`domainforge-core/Cargo.toml`, `pyproject.toml`, `package.json`) must be equal,
  or `FAILED=1` and the script exits `1`.
- `[check-sea-language-change.sh:BASE_REF]` — the gate skips only when
  `GITHUB_BASE_REF` is unset; a failed `git fetch` or unresolved base ref is a hard
  failure (fail closed), never a widened skip.
- `[create-tag.sh:git rev-parse]` — an existing tag is an error unless `--force`,
  which deletes it first; the script never pushes.
- `[create-github-release.sh:gh auth status]` — `gh` presence, authentication, and a
  pre-existing `v{VERSION}` tag are all preconditions that abort with exit `1`.
- `[build-wasm.sh:VERSION]` — an empty version extracted from
  `domainforge-core/Cargo.toml` aborts the build rather than emitting a package with
  a blank version.
- `[patch_napi_types.py:patch_file]` — the patcher is idempotent: patterns that no
  longer match leave the file untouched and report "No patches needed".
- `[resolve_rust_binary.py:fail_with_diagnostics]` — when no binary matches (after
  the optional fallback mode) the script prints searched directories to stderr and
  exits `1` instead of printing a wrong path.
- `[find_and_link_test_binary.sh:ln -s]` — the stable path is always
  `target/debug/deps/sea_debug_test`; when symlinking fails it copies and `chmod +x`
  instead.
- `[ci_tasks.py:_validate_tar_member]` / `[ci_tasks.py:_validate_zip_member]` —
  archive members with absolute paths, `..` components, or (for tar) symlink/hardlink
  entries are rejected before extraction.
- `[test-all.sh:FAILED]` — the script exits `1` if any suite failed, so a partial
  pass is never reported as success.
- `[run_commitlint.sh:node_modules/.bin/commitlint]` — if no compatible runner is
  found the script exits `1` rather than skipping the commit-message check.

## Side Effects
- `[build-python.sh:.venv]` — creates `.venv` and `pip install`s `maturin`/`pytest`
  when missing; mutates the active shell environment via `source`.
- `[build-wasm.sh:package.json]` — rewrites `target/wasm-pkg/package.json` in place
  (via `python3` or `node`) and creates a `.wasm.gz` with `gzip -k -f`.
- `[build-release.sh:SHA256SUMS.txt]` — writes build artifacts and
  `SHA256SUMS.txt` into `dist/`.
- `[bump-version.sh:git commit]` — edits three manifest files in place via `sed`/`jq`
  and, unless `--no-commit`/`--dry-run`, stages and commits them.
- `[bump-version.sh:jq]` — writes `domainforge-typescript/package.json.tmp` and
  `mv`s it over the original.
- `[create-tag.sh:git tag]` — creates (or with `--force`, deletes then recreates) a
  local annotated or GPG-signed tag.
- `[generate-changelog.sh:/tmp/changelog_new.md]` — builds the new changelog in
  `/tmp/changelog_new.md` and `mv`s it over `CHANGELOG.md`; commits unless
  `--no-commit`.
- `[generate-release-notes.sh:RELEASE_NOTES.md]` — writes/overwrites
  `RELEASE_NOTES.md` in the repo root.
- `[release.sh:git add -A]` — stages the entire worktree and creates
  `chore: release v<version>`; then pushes branch and tag (`git push origin ...`)
  and invokes `create-github-release.sh` after interactive confirmation.
- `[create-github-release.sh:eval]` — executes the assembled `gh release create`
  command, uploading every file in `dist/`.
- `[pre-release-check.sh:git fetch origin]` — performs a network fetch (failures are
  downgraded to a warning) and runs `just all-tests` and a release `cargo build`.
- `[check-sea-language-change.sh:git fetch origin]` — fetches `origin/$BASE_REF` at
  depth 1 and reads the PR diff via `git diff --name-only`.
- `[find_and_link_test_binary.sh:cargo test]` — builds test binaries into `target/`
  and creates/overwrites `target/debug/deps/sea_debug_test`.
- `[prepare_rust_debug.sh:update_launch_program.py]` — rewrites
  `.vscode/launch.json`, creating the file (and `.vscode/`) if absent.
- `[clear_debug_test.sh:rm -f]` — deletes `target/debug/deps/sea_debug_test` if
  present.
- `[ci_tasks.py:unpack_and_verify]` — extracts archives into `tmp_verify_unpack/`,
  then removes that directory in a `finally` block; executes the extracted binary
  with `--version` (10 s timeout) and adjusts `LD_LIBRARY_PATH`/`DYLD_LIBRARY_PATH`/
  `PATH` for the child process.
- `[check_unused_test_imports.py:print]` — read-only: prints candidate unused imports
  to stdout and never edits files.
- `[lint_release_security.py:main]` / `[lint_release_workflows.py:main]` /
  `[lint_workflow_gates.py:main]` — read `.github/workflows/*.yml` and `*.yaml` and
  `sys.exit(1)` on any finding; no writes.
- `[run_commitlint.sh:bunx]` — may download and execute `commitlint` through
  `bunx`/`npx`/`npm exec` over the network.

## Verification
- Run: `just all-tests`
- Run: `bash scripts/test-all.sh`
- Run: `bash scripts/pre-release-check.sh --dry-run`
- Run: `python3 scripts/lint_release_security.py`
- Run: `python3 scripts/lint_release_workflows.py`
- Run: `python3 scripts/lint_workflow_gates.py`

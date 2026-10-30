# DomainForge Blueprint

## System Overview

DomainForge is a Semantic Enterprise Architecture DSL. A single canonical Rust
core (`domainforge-core`) owns all business logic: it parses the SEA DSL, builds
a deterministic `Graph` of primitives, evaluates three-valued policies, and
projects that graph into 22 target ecosystems. The Python (PyO3), TypeScript
(napi-rs), and WASM (wasm-bindgen) surfaces are thin wrappers over that core —
they expose the same primitives and re-export the same functions, and none of
them reimplement DSL semantics.

The data path is: `.sea` source → `parser::parse_to_graph` → `Graph` →
`policy::Policy::evaluate` (three-valued `True`/`False`/`Null`) and/or
`projection::<family>::emit` → artifacts written through one shared
`ArtifactSink`. Determinism is a first-class contract: identity is minted by
`projection::ids` (content hashing, family tags) and ordering is preserved by
`IndexMap`, so two isolated runs over the same source produce byte-identical
output.

Repository layout splits along the publish boundary: `domainforge-core/` is the
canonical crate and CLI, `domainforge-python/` and `domainforge-typescript/` are
distribution packages whose wheels/addons are compiled from that crate, and
`scripts/` + `justfile` drive build, release, and proof automation.

## Local Interface & Invariants

Source files directly at the repository root:

- `[index.js:module.exports]` — re-exports `./domainforge-typescript/index.js`
  so `typescript-tests` and `example.js` share one loader; it deliberately holds
  no logic of its own, which is what prevents root/package loader drift.
- `[index.d.ts:export *]` — re-exports the canonical napi type declarations from
  `./domainforge-typescript/index`.
- `[example.js:require('./index')]` — runnable demo building a `Graph` from
  `Entity`/`Resource`/`Flow` and then parsing SEA source; prints to stdout.
- `[vitest.config.ts:test.include]` — pins the TS suite to
  `typescript-tests/**/*.test.ts`, `environment: 'node'` for NAPI-RS
  compatibility, and `pool: 'forks'` with `fileParallelism: false` for native
  binding stability.
- `[test_integration.py:domainforge.Graph]` — smoke script exercising the Python
  binding end to end (entities, resources, flows, graph statistics).
- `[release-0.7.0.sh:sed -i]` — **stale, do not run.** It targets a `sea-core/`
  path that no longer exists and bumps from `0.6.2` to `0.7.0`. Current releases
  go through `scripts/release.sh` (`[scripts/EGM.md]`).

Invariants that hold across the root:

- `[justfile:rust-test]` — the canonical Rust gate is
  `cargo test -p domainforge-core --features cli`; the `cli` feature is required
  for the binary and CLI tests to compile (`[justfile:cargo]` falls back to
  `devbox run -- cargo` when `cargo` is absent).
- `[justfile:enterprise-verify]` — the release gate runs `cargo fmt --all
  --check`, `cargo clippy --workspace --all-targets --all-features -- -D
  warnings`, the full test suite, doctests, `just all-tests`, then `just audit`.
- `[Cargo.toml:members]` — the workspace contains exactly one member,
  `domainforge-core`; bindings are not separate crates.
- `[package.json:workspaces]` — the npm workspace root declares
  `domainforge-typescript` as its only workspace and is `private: true`.
- `[vitest.config.ts:fileParallelism]` — native-binding tests must not run in
  parallel, so tests cannot be safely re-enabled concurrently without changing
  the loader's global state assumptions.

## Subdirectories & System Boundaries

- domainforge-core: `./domainforge-core/EGM.md` — canonical Rust crate: library
  modules, Pest grammar, the `domainforge` CLI binary, and the PyO3/napi-rs/
  wasm-bindgen binding implementations under `src/{python,typescript,wasm}/`.
- domainforge-python: `./domainforge-python/EGM.md` — PyPI distribution
  (`domainforge` 0.21.2, maturin backend) whose wheel compiles the core crate
  with the `python` feature; contains no Python business logic.
- domainforge-typescript: `./domainforge-typescript/EGM.md` — napi-rs
  distribution: the `index.js` loader that resolves native vs. WASI targets and
  the `index.d.ts` surface generated from `domainforge-core/src/typescript/`.
- scripts: `./scripts/EGM.md` — build/release/CI automation plus the `prove/`
  self-proving harness and `verify/` projection gates.
- lib: `./lib/EGM.md` — `validateNativeExports`, the shared guard that throws a
  descriptive error when a native addon is missing a required export.
- packaging: `./packaging/EGM.md` — per-platform installers and archives
  (`deb/`, `linux/`, `macos/`) plus the single-version guard, Linux ELF/ABI
  gate, and CLI smoke test. `windows/` holds only `.ps1`/`.iss` and has no
  contract.
- schemas: `./schemas/EGM.md` — published JSON Schemas for the application
  contract, AST (`v1`/`v2`/`v3`), canonical semantic envelope, and registry,
  plus vendored target schemas.
- python: no contract — `python/sea_dsl/` is empty and `python/domainforge/`
  holds only gitignored `.so` build artifacts; `python/tests/` is test-only.
- typescript-tests: no contract — test-only directory (all `*.test.ts`),
  excluded by policy from documentation scope.

Directories excluded from this bootstrap as non-source: `docs/` (Markdown),
`fixtures/`, `examples/`, `evidence/`, `secrets/`, `graft/`, `target/`, `dist/`,
`pkg/`, `node_modules/`, and the gitignored `docs/archive/`. The 26 `.sea` DSL
samples and 24 `.json` schemas under `fixtures/` and `schemas/` are data, not
source code, so they carry no contracts beyond `schemas/EGM.md`.

## Global Invariants

- **Canonical core, zero-logic bindings.** No Python, TypeScript, or WASM file
  may contain DSL semantics. Bindings convert types and delegate; the AST,
  policy engine, and projection renderers exist exactly once, in Rust.
  Cross-surface changes must update the Rust core,
  `domainforge-core/src/{python,typescript,wasm}/`, and the corresponding
  `tests/`, `typescript-tests/`, and `domainforge-core/tests/wasm_tests.rs`
  suites together.
- **Feature gating.** `cli`, `python`, `typescript`, and `wasm` modules are
  compiled conditionally (`[domainforge-core/src/lib.rs]`), so a change behind
  one feature can pass the default build while breaking another. Verify with the
  feature that owns the code.
- **Byte-level determinism.** Identity comes from content hashing
  (`[domainforge-core/src/projection/ids.rs:element_id]`), ordering from
  `IndexMap`/sorted iteration, and every family must emit through the shared
  `ArtifactSink` (`[domainforge-core/src/projection/sink.rs:ArtifactSink]`).
  Wall-clock time and random per-parse UUIDs must never reach artifact contents;
  caller-supplied `created_at` is the only sanctioned clock input.
- **Dangling references are hard errors.** Flow resolution is centralized in
  `[domainforge-core/src/projection/flows.rs:collect_flows]`; a family must not
  silently drop an unresolvable reference or reimplement resolution
  (ADR-011 §2).
- **Three-valued policy logic.** `True`/`False`/`Null` with `Null` propagation is
  the evaluation contract (`[domainforge-core/src/policy/mod.rs]` exports
  `ThreeValuedBool`); policy preconditions fail closed on `UNKNOWN`
  (`[domainforge-core/src/application/policy_context.rs:evaluate_precondition]`).
- **Grammar changes are ADR-gated.** `[domainforge-core/grammar/sea.pest]` is the
  single grammar for every surface; adding a construct requires the ADR process
  plus parser and projection verification via `just`.
- **Documentation contracts.** `AGENTS.md` files are behavioural instructions;
  `EGM.md` files are these generated architectural contracts. `README.md` is
  human-maintained and was not overwritten by this bootstrap.

## Global Side Effects

- **Filesystem writes** are confined to caller-supplied output roots and are
  path-validated before every write
  (`[domainforge-core/src/projection/sink.rs:write_artifact]` calls
  `protobuf::validate_output_path`).
- **Network** is never touched by the library or by projection code. Only
  release/CI tooling reaches the network: `bump-version.sh`,
  `create-github-release.sh` (`gh`), `create-tag.sh` push paths,
  `run_commitlint.sh` (`bunx`/`npx`), and the `git fetch` calls in
  `pre-release-check.sh` and `check-sea-language-change.sh`.
- **External process spawns** occur only in verification and packaging gates:
  `buf` (optional, degrades gracefully), `opa` 1.21.1 (`rego.sh`),
  `openshell-prover` (`openshell.sh`), `readelf` (`check-linux-binary.sh`), and
  `java`/SANY/TLC in `tla.sh`. Each of those gates fails closed when its tool is
  missing or the wrong version.
- **Local git state mutation** happens only in `scripts/release.sh`,
  `bump-version.sh`, `create-tag.sh`, and `generate-changelog.sh`. None of them
  run as part of the test suites.
- **Import-time registration.** Importing `domainforge` runs the PyO3 module
  init, and requiring `domainforge-typescript` loads a platform-native `.node`
  binary (or falls back to WASI), so both fail at import time when the compiled
  artifact is missing.

## System Verification

- Run: `just rust-test` — Rust core suite (`cargo test -p domainforge-core --features cli`)
- Run: `just python-test` — Python binding suite (`pytest`, testpaths `tests/`)
- Run: `just ts-test` — TypeScript binding suite (`bun vitest run`)
- Run: `just all-tests` — all three language suites
- Run: `just enterprise-verify` — fmt, clippy `-D warnings`, tests, doctests, all-tests, audit
- Run: `just prove` — self-proving evidence harness written to `evidence/latest/`
- Run: `bash scripts/verify/projection-targets/all.sh` — baseline plus 14 per-target projection gates

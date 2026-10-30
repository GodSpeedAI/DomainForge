# domainforge-core — Canonical Rust Core Library and CLI

## Overview
The canonical DomainForge crate: Rust library (`rlib` + `cdylib`) containing the
SEA DSL parser, semantic graph, policy engine, projections, and the
Python/TypeScript/WASM binding layers, plus the `domainforge` CLI binary. Python,
TypeScript, and WASM packages project this crate's behavior and must not
duplicate its business logic.

## Interface
- `[build.rs:main]` — build script; declares `extern crate napi_build;` and calls `napi_build::setup()` only under `#[cfg(feature = "typescript")]`, so Node-API symbol setup happens for the napi-rs build and is compiled out otherwise.
- `[Cargo.toml]` — crate `domainforge-core` v0.21.2, edition 2021, `rust-version = "1.89"`; library target `domainforge_core` at `src/lib.rs` with `crate-type = ["rlib", "cdylib"]`.
- `[Cargo.toml]` — binary target `domainforge` at `src/bin/domainforge.rs` with `required-features = ["cli"]`.
- `[Cargo.toml]` — feature `default = []`; `python` (`pyo3`, `pythonize`, `signing`), `typescript` (`napi`, `napi-derive`, `signing`), `wasm` (`wasm-bindgen`, `serde-wasm-bindgen`, `uuid/js`, `lol_alloc`), `wasm-projections`, `shacl` (`oxigraph`), `formatting` (`icu_decimal`, `icu_locale_core`, `fixed_decimal`), `cli` (`three_valued_logic`, `clap`, `colored`, `signing`), `three_valued_logic`, `json-schema` (`schemars`), `signing` (`ed25519-dalek`).
- `[Cargo.toml]` — `[build-dependencies]` `napi-build = "2.1"`.
- `[fix_api.py:main]` — deprecated regex-based migration script; its docstring marks it replaced by `fix_api_v2.py` and `main()` prints an error directing callers to the replacement, then exits.
- `[fix_api_v2.py]` — AST-based Rust source transformation tool using `tree-sitter` + `tree_sitter_rust`, with `--dir`/`--file`, `--transform`, `--dry-run`, backup/rollback helpers, and a hard `sys.exit(1)` when `tree-sitter-rust` is not installed.
- Child contracts: `./src/EGM.md` — the library and binding source tree; `./grammar/EGM.md` — the Pest grammar `sea.pest`; `./examples/EGM.md` — DSL samples and the parser demo.

## Invariants
- `[build.rs:main]` — `napi_build::setup()` runs only when the `typescript` feature is enabled; a non-TypeScript build must not link or run napi build setup.
- `[Cargo.toml]` — the `domainforge` binary is only built with the `cli` feature (`required-features`), so `cargo build` without `cli` produces the library only.
- `[Cargo.toml]` — `python`, `typescript`, and `cli` all pull in the `signing` feature, so pack/fact signing is available on every surface that exposes the semantic-pack or authority API.
- `fix_api.py` / `fix_api_v2.py` — committed scratch migration scripts, not part of the build, test, or runtime path; `fix_api.py` is explicitly deprecated in favor of `fix_api_v2.py`.

## Side Effects
- `fix_api_v2.py` — rewrites Rust source files in place when not run with `--dry-run`; it writes `<file>.bak` backups via `shutil.copy2` before transforming, restores from them on rollback, and writes parse-failure logs to `/tmp/rollback/`.
- `fix_api.py` — prints a deprecation error directing callers to `fix_api_v2.py`, then calls `sys.exit(1)`; no filesystem work.

## Verification
- Run: `cargo test -p domainforge-core --features cli`

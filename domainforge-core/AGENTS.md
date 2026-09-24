# AGENTS.md — domainforge-core

Scoped operating contract for `domainforge-core/`. Governs the canonical Rust core library, CLI binary, and foreign language binding implementations. Follow root `AGENTS.md` for repository-wide invariants and verification gates.

## 1. Scope and Architecture

`domainforge-core` contains the authoritative compiler and semantic reasoning engine for the SEA DSL.

Logical layering flows strictly top-to-bottom:

1. **Ingestion & Resolution**: `grammar/sea.pest`, `src/parser/`, `src/module/`
2. **Semantic Kernel**: `src/graph/`, `src/primitives/`, `src/policy/`, `src/units/`
3. **Contracts & Governance**: `src/application/`, `src/semantic_pack/`, `src/authority/`
4. **Projections**: `src/projection/`, `src/calm/`, `src/kg.rs`
5. **Host Interfaces & CLI**: `src/cli/`, `src/bin/domainforge.rs`, `src/python/`, `src/typescript/`, `src/wasm/`

Layering invariants:
* The Semantic Kernel has no dependencies on Projections, CLI, or Bindings.
* Host language bindings (`src/python/`, `src/typescript/`, `src/wasm/`) wrap core types through FFI; never implement domain logic or policy evaluation in binding modules.
* Projections adhere to the Operator Family Pattern (ADR-011) via `ArtifactSink`. Projections are CLI-only by default (WASM excludes projection families to respect the 2.5 MB bundle limit).

## 2. Grammar-First Evolution

All syntax modifications begin in `domainforge-core/grammar/sea.pest` before AST or graph representations are altered.

Grammar change workflow:
1. Modify `domainforge-core/grammar/sea.pest`.
2. Update parser AST and conversion (`src/parser/ast.rs`, `src/parser/ast_convert.rs`).
3. Add or update parser and compiler tests in `tests/`.
4. Update CALM/KG and projection serializers when exported semantics change.
5. Create an ADR under `docs/specs/ADR-*.md` documenting the change (enforced by `scripts/check-sea-language-change.sh`).

Never edit parser or AST code directly without synchronizing `sea.pest`.

## 3. Core Invariants

* **Deterministic iteration (`IndexMap`)**: `std::collections::HashMap` is forbidden for policy-relevant state, graph storage, or projection emission. Use `indexmap::IndexMap` to ensure byte-identical outputs across runs.
* **Content-addressed identity (`ConceptId`)**: Concept IDs are minted deterministically via UUID v5 using fixed DNS namespace UUID (`6ba7b810-9dad-11d1-80b4-00c04fd430c8`) hashed with `<namespace>::<name>`.
* **Namespaces**: `namespace()` returns `&str` and defaults to `"default"`.
* **Independent IDs**: Use UUID v4 for independent unique identifiers where established.
* **Precise quantities**: Use `rust_decimal::Decimal` for all arithmetic quantities.
* **Units**: Parse and validate all units through the canonical units API (`src/units/`).
* **Flows**: `Flow::new()` takes `ConceptId`s for resource, from, and to—not entity/resource objects.
* **Three-valued policy logic**: Policies evaluate under Kleene three-valued logic (`True`, `False`, `Null`/`Unknown`) in `src/policy/`.
* **Workspace profiles**: Profiles are managed at the workspace root (`Cargo.toml`). Never add `[profile]` tables to `domainforge-core/Cargo.toml`.

## 4. Commands and Verification

Use `just` recipes where available, falling back to scoped `cargo` commands:

* **Focused Rust test**: `cargo test -p domainforge-core --features cli --test <test_name>`
* **Crate test suite**: `just rust-test` (or `cargo test -p domainforge-core --features cli`)
* **CLI tests**: `just cli-test`, `just cli-validate`, `just cli-workflow`
* **WASM test suite**: `just wasm-test`
* **Doctests**: `cargo test -p domainforge-core --features cli --doc`
* **Rust quality gate**: `cargo clippy --workspace --all-targets --all-features -- -D warnings` and `cargo fmt --all --check`
* **Self-proving harness**: `just prove` (see `PROOFS.md`)
* **Cell projection verify**: `just cell-verify`

A passing Rust suite proves core behavior only. When touching public primitives or FFI surfaces, run `just all-tests` from the repository root.

## 5. Local Hazards

* **Non-deterministic maps**: Using `HashMap` in graph structures or projections silently breaks byte-determinism proofs (`just prove`).
* **Grammar desynchronization**: Updating AST without updating `sea.pest` fails CI's language evolution check.
* **Binding drift**: Adding or changing public primitives without updating `src/python/`, `src/typescript/`, and `src/wasm/` breaks foreign language consumers.
* **WASM bloat**: Compiling projection families into default WASM features exceeds bundle size constraints.

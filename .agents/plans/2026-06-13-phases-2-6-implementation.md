# Implementation Plan: Audit Phases 2–6

**Source:** `.agents/reports/semantic_infrastructure_audit_DomainForge_2026-06-12.md` §7
**Prerequisite state:** Phase 0 (canonical entrypoints + EvaluationMode) ✅; Phase 1 (conformance corpus items 01–07 wired to `sea` CLI oracle) ✅.
**Date:** 2026-06-13

## Current facts (verified)

- `conformance/01_minimal_domain` … `07_evolution` have `manifest.json` + `expected/*.json`; `08_authority` exists but is NOT wired (no manifest).
- Harness `sea-core/tests/conformance_corpus_tests.rs` runs `sea parse|validate input.sea --format json` via `CARGO_BIN_EXE_sea`, normalizes flow UUIDs → `flow:0`, `flow:1`, compares JSON.
- `Graph` derives `Serialize`; CLI canonical output = `serde_json::to_string_pretty(&graph)` (`cli/parse.rs:73`).
- **Python Graph binding has NO `to_json()`**; **TS Graph binding has NO `to_json()`**; **WASM Graph binding HAS `to_json()`** (`wasm/graph.rs`).
- CI `test-wasm` job only builds + size-checks; does NOT run `wasm_tests.rs`. No wasm-pack test step.
- `tests/test_golden_payment_flow.py` and `typescript-tests/golden-payment-flow.test.ts` inline an identical `PAYMENT_DSL` (the same model as `conformance/01_minimal_domain/input.sea`).
- KG import: `kg_import.rs::validate_and_convert` → `kg.rs::KnowledgeGraph::to_graph()`. Lossy subset lives in `to_graph()`; policies/units/temporal not reconstructed.
- `AuthorityTrace` (`authority/trace.rs`) serializes via serde; 7+ hash fields present.
- `ConceptChange` stored in `Graph.concept_changes` IndexMap but NEVER enforced (G8 gap confirmed).

## Phase 2 — Cross-language parity (Python + TS)

**Goal:** Python and TS load the SHARED `conformance/` corpus, parse via bindings, emit canonical JSON, normalize flow IDs, byte-compare against Rust `expected/` files.

**Prerequisite binding change (each language owns its own):**
- `sea-core/src/python/graph.rs`: add `fn to_json(&self) -> PyResult<String>` wrapping `serde_json::to_string(&self.graph)`.
- `sea-core/src/typescript/graph.rs`: add `pub fn to_json(&self) -> Result<String>` wrapping `serde_json::to_string(&self.inner)`.

**Tests:**
- `tests/test_conformance_parity.py`: iterate `conformance/*/manifest.json`; for `parse` items call `Graph.parse(open(input).read()).to_json()`; for `validate` items call the binding's policy-evaluation path; normalize flow UUIDs → positional; `json.loads` + deep-equal against `expected/*.json`.
- `typescript-tests/conformance-parity.test.ts`: same logic in TS via `Graph.parse` + `graph.toJson()`.
- Delete inlined `PAYMENT_DSL` from `tests/test_golden_payment_flow.py` and `typescript-tests/golden-payment-flow.test.ts`; replace with a redirect to the shared corpus or delete the now-redundant golden tests.

**Settlement:** a deliberately introduced binding divergence (e.g., quantity coercion) fails the test.

## Phase 3 — WASM conformance

**Goal:** run corpus items 01–03 (parse, evaluate tristate, pack hash) under wasm-pack/node.

**Harness:** `typescript-tests/wasm-conformance.test.ts` (or a node script) that imports the wasm pkg, parses each corpus `input.sea`, calls `graph.to_json()`, normalizes flow IDs, compares to `expected/`.

**CI:** `test-wasm` job adds `wasm-pack test --node --features wasm` step (runs `wasm_tests.rs`) AND the corpus parity step.

## Phase 4 — KG round-trip honesty

**Goal:** every construct is either round-trip-tested or documented-lossy; import errors loudly on unsupported constructs.

**Files:**
- `sea-core/src/kg.rs` (`to_graph()`): enumerate reconstructed constructs; on encountering unsupported triples (policy/quantifier/unit/temporal), return `Err` instead of silently dropping.
- `docs/specs/kg_projection_loss.md`: manifest table (construct → exported → importable → loss reason).
- `conformance/10_kg_roundtrip/`: `input.sea` (entity+resource+flow+policy+unit), manifest, expected surviving-subset assertion.
- `sea-core/tests/kg_roundtrip_tests.rs` (or extend existing): assert loud failure on policy/unit import; assert surviving subset equality.

## Phase 5 — SEA-Forge contract fixture

**Goal:** versioned JSON schema for `pack + trace + decision`; pinned fixture.

**Files:**
- `schemas/seaforge-contract-v1.json`: JSON Schema (draft 2020-12) describing the three artifacts.
- `conformance/12_seaforge_fixture/`: `pack.json`, `trace.json`, `decision.json` generated from the acme_procurement domain; pinned.
- `sea-core/tests/seaforge_contract_tests.rs`: validate the three fixtures against the schema (use `jsonschema` crate or a lightweight validator); assert schema version field.

## Phase 6 — Provenance hardening

Five tests in `sea-core/tests/`:

1. **Signed-pack tamper** (`provenance_tamper_tests.rs`): build a signed pack, flip one byte in the canonical JSON, assert `validate_hash` returns `pack_hash_mismatch`.
2. **Derived-fact Null-poisoning** (`derived_fact_null_poisoning_tests.rs`): derive a fact from a Null premise; assert the derived fact is Null (propagation), not silently True/False.
3. **Order-permutation hash** (`order_permutation_hash_tests.rs`): parse two declaration-order permutations of the same model; assert canonical pack hash is equal OR document order-sensitivity explicitly (G2 caveat).
4. **Evolution enforcement** (`evolution_enforcement_tests.rs`): model v1 + policy → apply breaking `ConceptChange` → assert system refuses or flags evaluation of the v1 policy against v2 concepts. If no enforcement exists, add a minimal `Graph::check_concept_change_compatibility` gate and test it.
5. **Golden-trace byte-stability** (`golden_trace_stability_tests.rs`): build the same authority decision twice; assert byte-identical `AuthorityTrace` JSON. Cross-binding stability tracked as follow-up (needs binding trace serialization).

## Execution order

- **Wave 1 (parallel, non-conflicting scopes):** Phase 4 (KG code+docs), Phase 5 (schema+fixtures), Phase 6 (test files).
- **Wave 2 (after Wave 1 compiles):** Phase 2 Python, Phase 2 TS (binding `to_json` + parity tests), Phase 3 WASM.
- **Wave 3:** CI workflow YAML, conformance README, agent-state docs, final `cargo test` + clippy + fmt integration.

## Verification gates

- `cargo test -p sea-core --features cli` green.
- `cargo fmt --all --check` + `cargo clippy --all-targets --all-features -- -D warnings` green.
- Python parity test passes (`pytest tests/test_conformance_parity.py`).
- TS parity test passes (`bun test` / `npm run test:node`).
- Each new conformance item has a manifest + expected file committed.

# SEA-DSL Conformance Corpus

> This corpus **is** the standard. Everything else is implementation.

A single directory of small, frozen `.sea` models whose canonical output is pinned
and executed by `cargo test` against the `sea` CLI — the reference oracle. It
settles **canonical meaning** and **determinism** for the semantic spine
(parser → graph → three-valued policy) and is the first artifact downstream
consumers (SEA-Forge, bindings) can pin.

This implements **Phase 1** of the Semantic Infrastructure Audit
(`.agents/reports/semantic_infrastructure_audit_DomainForge_2026-06-12.md`). See
`docs/specs/canonical_entrypoints.md` for the canonical-entrypoint contract.

## How it works

Each item is a directory containing:

- `input.sea` — the frozen model.
- `manifest.json` — `{ description, command, input, expected }` where `command` is
  `parse` (canonical graph JSON) or `validate` (canonical policy-evaluation JSON).
- `expected/…json` — the pinned canonical output.

The harness `sea-core/tests/conformance_corpus_tests.rs` runs the CLI for each
item and compares normalized JSON. **Flow IDs are normalized** to positional
tokens (`flow:0`, …) on both sides because flows are events with random UUIDs;
every other concept ID is content-derived and stable.

```bash
cargo test --features cli --test conformance_corpus_tests
```

### Regenerating an expected file (only for intentional changes)

```bash
sea parse    conformance/01_minimal_domain/input.sea       --format json  > conformance/01_minimal_domain/expected/graph.json
sea validate conformance/02_policy_complete_facts/input.sea --format json > conformance/02_policy_complete_facts/expected/validate.json
```

A change to any expected file must be a deliberate, reviewed spec change.

## Items

| Item | Oracle | What it pins |
|---|---|---|
| `01_minimal_domain` | `parse` | Canonical graph for the payment domain (2 entities, 1 resource, 1 flow, 1 relation). |
| `02_policy_complete_facts` | `validate` | Obligation with all facts present → tristate `true`, 0 violations, mode `three_valued`. |
| `03_policy_null_facts` | `validate` | Missing-attribute reference → genuine NULL: tristate `null`, `is_satisfied:false`, `UNKNOWN (NULL)` violation. |
| `04_quantifiers` | `validate` | Vacuous `forall` over empty collection (True), `exists` over NULL element (Null), `exists_unique`. |
| `05_units` | `validate` | Cross-unit (kg vs g) quantity comparison — pins current behavior. |
| `06_temporal` | `validate` | `before` over flow timestamp attributes — pins current behavior. |
| `07_evolution` | `parse` | Entity versions + a breaking `ConceptChange` captured in the canonical graph. |
| `13_evolution_enforcement` | `validate` | Breaking `ConceptChange` plus an active policy → canonical validation reports `concept_change_compatibility`. |

### Roadmap (later phases of the audit)

| Item | Status | Phase |
|---|---|---|
| `08_authority` | **Wired** — `conformance/08_authority/{config,request,facts,packs,trace,decision}.json` pin a full `AuthorityTrace` (two policies of differing specificity; `deny` with one `conflict_resolution_step`), schema-valid against `seaforge-contract-v1.json` and byte-pinned by `sea-core/tests/authority_fixture_tests.rs`. All seven hashes are deterministic (canonical sorted-key hashing) and pinned byte-for-byte. **Cross-binding parity is wired**: the shared inputs drive `AuthorityEnvironment.evaluate` (Python), `evaluateAuthority` (TypeScript), and `evaluateAuthority` (WASM) with volatile-normalized byte-comparison against the committed goldens (`tests/test_authority_parity.py`, `typescript-tests/authority-parity.test.ts`, `sea-core/tests/wasm_tests.rs::test_authority_08_trace_parity`). Additional Phase 6 provenance proofs live as Rust tests (`golden_trace_stability_tests.rs`, `provenance_tamper_tests.rs`, `derived_fact_null_poisoning_tests.rs`, `order_permutation_hash_tests.rs`, `evolution_enforcement_tests.rs`). | Phase 6 / G6 ✅ |
| `10_kg_roundtrip` | **Wired** — surviving subset + loud-error assertions | Phase 4 ✅ |
| `11_parity` | **Wired** — both `parse` items (`01`, `07`) and `validate` items (`02`–`06`, `13`) produce byte-identical canonical JSON across Rust/Python/TS (flow IDs normalized). The single core producer `validate_to_canonical_json` in `validation_result.rs` is wrapped by every binding. | Phase 2 ✅ |
| `12_seaforge_fixture` | **Wired** — `conformance/12_seaforge_fixture/{pack,trace,decision}.json` are committed, schema-valid against `schemas/seaforge-contract-v1.json`, and byte-pinned by `sea-core/tests/seaforge_fixture_tests.rs`. | Phase 5 ✅ |

WASM conformance (Phase 3): `wasm_tests.rs` runs under `wasm-pack test --node` in CI.
WASM is now in the **byte-compared** parity matrix: items `01` (parse), `02`
(validate, tristate `true`), and `03` (validate, tristate `null`) are parsed,
evaluated, and byte-compared against the Rust `expected/` files (flow IDs
normalized for `01`). Three new `#[wasm_bindgen_test]` functions cover parse
canonical JSON, validate tristate-true, and validate tristate-null.

Cross-language parity (Phase 2): Python (`tests/test_conformance_parity.py`) and
TypeScript (`typescript-tests/conformance-parity.test.ts`) load the shared corpus,
select both `parse` and `validate` items, parse via bindings, serialize canonical
JSON, normalize volatile flow UUIDs, and byte-compare against these `expected/`
files. This proves the **structural spine** (`.sea` → canonical graph JSON) AND the
**policy-evaluation aggregate** (`sea validate --format json`) are identical across
Rust/Python/TS. Both suites parametrize over all 8 active parse/validate items
(`01`–`07`, `13`).

## Notes on pinned behavior

The corpus pins the **actual** behavior of the engine, which is the point: any
future change (improvement or regression) must update the corpus deliberately.
Items `05_units` and `06_temporal` currently resolve to `UNKNOWN (NULL)` for the
cross-unit and timestamp-attribute comparison paths; pinning this documents the
current contract and will flag the day that behavior changes.

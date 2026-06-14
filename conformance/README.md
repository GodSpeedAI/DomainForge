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

### Roadmap (later phases of the audit)

| Item | Status | Phase |
|---|---|---|
| `08_authority` | Pending corpus fixture — Phase 6 provenance proofs live as Rust tests (`sea-core/tests/golden_trace_stability_tests.rs`, `provenance_tamper_tests.rs`, `derived_fact_null_poisoning_tests.rs`, `order_permutation_hash_tests.rs`, `evolution_enforcement_tests.rs`) | Phase 6 / G6 |
| `10_kg_roundtrip` | **Wired** — surviving subset + loud-error assertions | Phase 4 ✅ |
| `11_parity` | **Partial** — `parse`-command items (`01`, `07`) produce identical canonical graph JSON across Rust/Python/TS (flow IDs normalized). `validate`/tristate items (`02`–`06`) are **not yet** parity-checked cross-language. | Phase 2 ◑ |
| `12_seaforge_fixture` | **Schema wired** — `schemas/seaforge-contract-v1.json` + `sea-core/tests/seaforge_contract_tests.rs` validate a real pack/trace/decision against the contract. A pinned corpus fixture directory for SEA-Forge to import is the remaining (joint) Phase 5 step. | Phase 5 ◑ |

WASM conformance (Phase 3): `wasm_tests.rs` runs under `wasm-pack test --node` in CI
(closing the "WASM untested in CI" gap), including a corpus-minimal parse + canonical
JSON test. WASM is not yet in the **byte-compared** parity matrix — running policy
evaluation / tristate / pack-hash on the shared corpus and comparing canonical JSON
against the Rust `expected/` files is the remaining Phase 3 step.

Cross-language parity (Phase 2): Python (`tests/test_conformance_parity.py`) and
TypeScript (`typescript-tests/conformance-parity.test.ts`) load the shared corpus,
select the `parse`-command items, parse via bindings, serialize canonical graph
JSON, normalize volatile flow UUIDs, and byte-compare against these `expected/`
files. This proves the **structural spine** (`.sea` → canonical graph JSON) is
identical across Rust/Python/TS. Policy-result (tristate) parity for the `validate`
items (`02`–`06`) is the remaining Phase 2 step: it needs a binding method that emits
the aggregate `validate` JSON the CLI oracle produces, so it is intentionally not
claimed here yet. The inlined `PAYMENT_DSL` fixtures were removed; both golden tests
now load from `conformance/01_minimal_domain/input.sea`.

## Notes on pinned behavior

The corpus pins the **actual** behavior of the engine, which is the point: any
future change (improvement or regression) must update the corpus deliberately.
Items `05_units` and `06_temporal` currently resolve to `UNKNOWN (NULL)` for the
cross-unit and timestamp-attribute comparison paths; pinning this documents the
current contract and will flag the day that behavior changes.

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

These items are intentionally **not yet wired** (no `manifest.json`), to keep the
suite green while the underlying oracles land:

- `08_authority` — full `AuthorityTrace` JSON byte-stability (needs an authority-CLI
  golden fixture; audit Phase 6 / G6).
- `09_calm_roundtrip`, `10_kg_roundtrip` — projection round-trip honesty (Phases 4).
- `11_parity` — items 01–06 byte-identical across Rust / Python / TypeScript / WASM
  (Phases 2–3). Today only the Rust CLI side is pinned.
- `12_seaforge_fixture` — versioned `pack + trace + decision` contract (Phase 5).

## Notes on pinned behavior

The corpus pins the **actual** behavior of the engine, which is the point: any
future change (improvement or regression) must update the corpus deliberately.
Items `05_units` and `06_temporal` currently resolve to `UNKNOWN (NULL)` for the
cross-unit and timestamp-attribute comparison paths; pinning this documents the
current contract and will flag the day that behavior changes.

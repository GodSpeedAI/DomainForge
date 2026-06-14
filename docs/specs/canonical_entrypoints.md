# Canonical Entrypoints & Logic Mode (Phase 0)

This document defines the **one documented way to get one canonical answer** from
DomainForge / SEA-DSL. It is the settlement of Phase 0 of the Semantic
Infrastructure Audit (`.agents/reports/semantic_infrastructure_audit_DomainForge_2026-06-12.md`).

A newcomer should be able to produce the canonical JSON for a model in under five
minutes using only the `sea` CLI.

## 1. Installing the CLI

The `sea` binary is **feature-gated**. `cargo install sea-core` alone does **not**
build the CLI. You must enable the `cli` feature:

```bash
# Build locally
cargo build --features cli --bin sea

# Or install
cargo install sea-core --features cli
```

The `cli` feature implies `three_valued_logic` (the canonical semantics, see §3),
`clap`, `colored`, and `signing`.

## 2. The reference oracles

There are exactly two canonical entrypoints. Both emit deterministic JSON
(`serde_json` object keys are emitted in sorted order; structural collections use
insertion-ordered `IndexMap`s).

| Question | Command | Canonical output |
|---|---|---|
| "What is the canonical graph for this model?" | `sea parse MODEL.sea --format json` | The serialized `Graph` (entities, roles, resources, flows, relations, policies, …). |
| "How do this model's policies evaluate?" | `sea validate MODEL.sea --format json` | Per-policy evaluation results **plus** aggregate diagnostics. |

### `sea parse --format json`

Emits the full `Graph`. All concept IDs except flows are content-derived
(deterministic UUID v5). **Flow IDs are deliberately random (UUID v4)** because
flows model events, not concepts (see `sea-core/src/primitives/flow.rs`). Flow IDs
are therefore **not** part of the canonical surface; any byte-comparison of graph
JSON across runs or runtimes must normalize flow IDs to positional placeholders.
The conformance corpus harness does exactly this
(`sea-core/tests/conformance_corpus_tests.rs`).

### `sea validate --format json`

This is the canonical **policy evaluation oracle**. Shape:

```json
{
  "evaluation_mode": "three_valued",
  "error_count": 0,
  "policies": [
    {
      "name": "MoneyCap",
      "namespace": "default",
      "modality": "obligation",
      "kind": "constraint",
      "is_satisfied": true,
      "is_satisfied_tristate": true,
      "evaluation_mode": "three_valued",
      "violations": []
    }
  ],
  "violations": []
}
```

Every policy reports both its tri-state result (`is_satisfied_tristate`: `true` /
`false` / `null`) and the **logic mode that produced it**. The legacy boolean
`is_satisfied` collapses `null` to `false` (fail-closed) and is kept only for
backward compatibility — consumers of the standard layer **must** read
`is_satisfied_tristate`.

## 3. Logic mode decision: three-valued is canonical

The audit flagged a runtime toggle (`Graph::use_three_valued_logic`) that gave one
model two possible meanings, with no record of which semantics applied.

**Decision (settled): three-valued (Kleene) logic is the canonical semantics of the
standard layer.** Boolean mode is legacy/back-compat only.

To make a result self-describing rather than removing the toggle outright (which
would be a breaking change to existing consumers), every `EvaluationResult` now
carries an `evaluation_mode` field (`crate::policy::EvaluationMode`):

- `EvaluationMode::ThreeValued` → serialized as `"three_valued"` (canonical, the
  default for new `Graph`s).
- `EvaluationMode::Boolean` → serialized as `"boolean"` (legacy; collapses NULL to
  `false` with no NULL semantics).

This closes the G1/G7 gap: the same pack evaluated under different toggle settings
now yields results that each declare which semantics produced them. The authority
path already hashed `unknown_handling_config`; plain policy evaluation now records
its mode too.

`EvaluationMode` is surfaced identically across all four runtimes:

- Rust: `EvaluationResult.evaluation_mode` (enum) / `.as_str()`.
- Python: `EvaluationResult.evaluation_mode` (str).
- TypeScript: `EvaluationResult.evaluationMode` (string, via napi `evaluation_mode`).
- WASM: `EvaluationResult.evaluationMode` (string).

## 4. Determinism notes

- Concept IDs (entities, roles, resources, relations, policies) are content-derived
  and stable across runs and runtimes.
- Flow IDs are random per-run (event identity) and are normalized out of the
  canonical surface — see §2.
- Canonical form is **parse-order dependent** (`IndexMap` insertion order). Two
  semantically identical models written in different declaration order may serialize
  differently. This is acceptable and documented (audit G2); an order-permutation
  hash test is tracked for Phase 6.

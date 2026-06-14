# sea-core/src — Rust core (authoritative)

Single canonical engine. Every surface (CLI, Python, TypeScript, WASM) is a thin
wrapper over the types here — never reimplement logic in a binding.

## Semantic spine (conformance-locked — break here = stack-fatal)

`grammar/sea.pest` → `parser/` (AST) → `graph/` (store) → `policy/` (three-valued
eval) → `semantic_pack/` (canonical-JSON pack hash) → `authority/` (decision trace).
Pinned end-to-end by `../../conformance/` (see `docs/specs/canonical_entrypoints.md`).

## Module map

| Module | Role | Deeper doc |
|--------|------|-----------|
| `primitives/` | Domain types: Entity, Resource, Flow, Instance, Role, Relation, Pattern, ConceptChange, Metric | — |
| `graph/` | `Graph` store; IndexMap-backed (deterministic); `validate()`; logic-mode flag | — |
| `parser/` | Pest grammar → internal AST → `Graph`; `ast.rs` is the giant | `parser/AGENTS.md` |
| `policy/` | Expression eval, **three-valued (Kleene) logic**, quantifiers, normalize | `policy/AGENTS.md` |
| `authority/` | Trust/provenance: resolver, conflict steps, hashed `AuthorityTrace` | `authority/AGENTS.md` |
| `semantic_pack/` | Portable spine format: canonical_json + hashing + signing + diff | — |
| `cli/` | `sea` binary subcommands — the reference oracles | `cli/AGENTS.md` |
| `calm/`, `kg.rs`, `kg_import.rs`, `sbvr.rs`, `projection/` | Export/import surfaces (CALM proven; KG export one-way; protobuf large) | — |
| `python/`, `typescript/`, `wasm/` | Binding wrappers (pyo3 / napi / wasm-bindgen) | — |
| `units/`, `registry/`, `module/`, `formatter/`, `error/`, `validation_error.rs` | Units+dimensions, namespace registry, imports, pretty-printer, diagnostics | — |

## Where to look

| Task | Location |
|------|----------|
| Add a primitive | `primitives/` + all three binding dirs (root cross-binding rule) |
| Change evaluation semantics | `policy/core.rs` (`evaluate_with_mode`, `EvaluationResult`) |
| Canonical graph/eval JSON | `cli/parse.rs`, `cli/validate.rs` |
| Pack hash / signing | `semantic_pack/canonical_json.rs`, `semantic_pack/signing.rs` |
| Decision provenance | `authority/trace.rs`, `authority/resolver.rs` |

## Notes

- Three-valued logic is **canonical**; every `EvaluationResult` records its
  `evaluation_mode` (`policy/core.rs`). Boolean mode is legacy.
- Flow IDs are random UUID v4 (event identity); all other concept IDs are
  content-derived v5 and stable. Normalize flow IDs before byte-comparing graphs.
- Largest/most-careful files: `projection/protobuf.rs` (3.3k), `parser/ast.rs` (2.9k),
  `kg.rs` (1.5k), `calm/import.rs` (1.2k), `policy/core.rs` (1.2k).

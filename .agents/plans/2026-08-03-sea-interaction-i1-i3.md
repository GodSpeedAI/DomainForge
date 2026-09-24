# SEA Interaction Interventions I1-I3 Implementation Plan

> **For agentic workers:** Execute inline with `test-driven-development`,
> `incremental-implementation`, and `code-review-and-quality`. Steps use
> checkbox (`- [ ]`) syntax. This task does not authorize sub-agent delegation.

**Goal:** Preserve and enforce typed entity state in `Graph`, resolve filesystem
CLI inputs through the existing deterministic module closure, and expose entity
instances to policies without changing legacy bodyless models or projections.

**Architecture:** `ResolvedModuleSet` remains the sole import/symbol authority.
Its existing `ApplicationContract` supplies resolved `EntityContract`,
`EnumContract`, `FieldType`, and `FieldConstraint` values to a graph-owned,
additive validation index; the graph does not copy parser-only declarations.
Filesystem input is adapted into the existing `SourceMap`, and policy projection
continues to expose JSON values without changing stored instance fields.

**Tech Stack:** Rust 2021, Pest, Serde, IndexMap, rust_decimal, PyO3, napi-rs,
wasm-bindgen, pytest, Bun/Vitest.

## Global Constraints

- Governing request is I1-I3 only; exclude I4-I6, new import forms, journey or
  transition grammar, semantic-pack loading, CMMN, and projection enhancements.
- Keep `ResolvedModuleSet` and `SourceMap` as the one module-resolution owner.
- Reuse application-contract types and existing `Pattern`, `Graph`,
  `ApplicationDiagnostic`, `ParseError`, and `ValidationResult` abstractions.
- Typed validation covers required/optional/unknown fields, scalar and enum
  types, list elements, all supported constraints, entity-reference targets,
  per-entity key uniqueness, and dangling references.
- Bodyless entities remain schemaless. Their instances and serialized Graph
  shape remain accepted. Empty validation indexes use serde defaults and are
  omitted so legacy Graph JSON remains byte-compatible.
- Validate a completed instance set so forward references are legal; public
  single-instance insertion validates atomically against the candidate graph.
- Filesystem closure traversal follows authored imports, rejects escapes and
  ambiguity, and feeds normalized logical paths to the existing resolver.
- The grammar change is limited to adding `entity_instances` to the existing
  policy collection vocabulary.
- Preserve unrelated staged and unstaged work. No files are deleted.

---

### Task 1: Typed Graph Contracts and Instance Validation

**Files:**
- Create: `domainforge-core/src/graph/entity_validation.rs`
- Create: `domainforge-core/tests/entity_instance_validation_tests.rs`
- Modify: `domainforge-core/src/graph/mod.rs`
- Modify: `domainforge-core/src/application/resolve.rs`
- Modify: `domainforge-core/src/parser/mod.rs`
- Modify: `domainforge-core/src/parser/ast.rs`

**Interfaces:**
- `Graph::entity_contract(&ConceptId) -> Option<&EntityContract>`
- `Graph::validate_entity_instances() -> Result<(), Vec<String>>`
- `Graph::attach_application_contract(ApplicationContract) -> Result<(), Vec<String>>`
- Internal graph construction inserts the whole instance set before the final
  validation pass; `Graph::add_entity_instance` remains atomic.

- [ ] Write focused tests proving resolved fields remain queryable and legacy
  bodyless instances remain accepted.
- [ ] Run `cargo test -p domainforge-core --test entity_instance_validation_tests --features cli`
  and confirm RED because the graph has no resolved entity contract.
- [ ] Attach resolved entity/enum contract indexes in `build_graph_from_set`,
  preserve empty-index serialization, and route single-source graph parsing
  through the same resolved contract construction.
- [ ] Add RED cases for missing required fields, optional omission, unknown
  fields, scalar mismatches, invalid enum wire values, all constraint families,
  duplicate keys, valid and dangling typed references, and forward references.
- [ ] Implement value/constraint/key/reference validation using `FieldContract`,
  `FieldType`, `FieldConstraint`, graph patterns, and typed target key fields.
- [ ] Re-run the focused target and existing instance/application compatibility
  tests until GREEN.

### Task 2: Filesystem CLI Closure Resolution

**Files:**
- Create: `domainforge-core/tests/cli_module_closure_tests.rs`
- Modify: `domainforge-core/src/module/resolver.rs`
- Modify: `domainforge-core/src/parser/mod.rs`
- Modify only if dispatch cannot stay centralized:
  `domainforge-core/src/cli/{validate,parse,project}.rs`

**Interfaces:**
- `resolve_filesystem_graph(entry_path, source, registry, default_namespace)`
  builds a deterministic transitive filesystem source set and calls the
  existing `resolve_source_map`/`build_graph_from_set` path.

- [ ] Write a CLI regression fixture where `instances.sea` imports an exported
  typed entity from `types.sea` under the same namespace.
- [ ] Run the focused CLI test and confirm RED with the current
  `Entity '<type>' not found` failure.
- [ ] Implement deterministic filesystem-to-`SourceMap` collection for relative,
  registry-backed namespace, and built-in std imports without changing import
  syntax or symbol rules.
- [ ] Route filesystem parse/validate/project graph inputs through the adapter;
  leave AST-only parse and Cell's AST-owned projection path unchanged.
- [ ] Prove imported-instance success plus existing APP014/APP015 closure
  diagnostics and projection dispatch compatibility.

### Task 3: Entity-Instance Policy Bindings

**Files:**
- Modify: `domainforge-core/grammar/sea.pest`
- Modify: `domainforge-core/src/policy/quantifier.rs`
- Modify: `domainforge-core/tests/policy_tests.rs`
- Modify: parser/formatter tests only where the new closed collection token
  requires round-trip coverage.

**Interfaces:**
- Plural collection: `entity_instances`.
- Singular aggregate/filter binder: `entity_instance`.
- Projected members: reserved `id`, `name`, `entity`, and `namespace`, followed
  by authored fields without allowing them to overwrite reserved members.

- [ ] Add RED quantifier and aggregation/filter tests over entity-instance
  fields, including an end-to-end parsed SEA policy.
- [ ] Add the grammar token, JSON collection projection, and singular binder.
- [ ] Re-run focused policy/parser targets and confirm GREEN.

### Task 4: Cross-Binding Parity and Public Documentation

**Files:**
- Modify: `domainforge-core/src/{python,typescript,wasm}/graph.rs`
- Modify: `tests/test_parser.py`
- Modify: `typescript-tests/native-binding.test.ts`
- Modify: `domainforge-core/tests/wasm_tests.rs`
- Modify: `docs/specs/ADR-013-sea-application-contract.md`
- Modify: `docs/reference/{sea-application-contract,cli-commands}.md`

**Interfaces:**
- Thin binding methods return Rust-produced entity-contract JSON; bindings do
  not resolve or validate types independently.

- [ ] Add binding parity tests for graph-visible typed entity contracts.
- [ ] Add thin Rust-backed accessors in Python, TypeScript, and WASM.
- [ ] Update ADR-013's Graph consumer row to record the additive validation
  index and document filesystem CLI closure plus `entity_instances` semantics.
- [ ] Rebuild bindings and run each focused cross-language test.

### Task 5: SEA Forge Faithful Fixture, Compatibility, and Full Gates

**Files:**
- Create: `fixtures/sea_forge_interaction_i1_i3/{types,instances,invalid}.sea`
- Create: `domainforge-core/tests/sea_forge_interaction_i1_i3_tests.rs`
- Update: `.agents/current_state.md`
- Update: `.agents/next_steps.md`

**Interfaces:**
- The fixture models a typed `CanonicalJourney`, enum maturity, and an imported
  same-namespace journey instance; the invalid variant contains a bad enum or
  dangling journey reference and must fail closed.

- [ ] Prove the prior imported-instance case succeeds via the real CLI.
- [ ] Prove intentionally invalid typed data fails via the real CLI with the
  offending entity/instance/field in the diagnostic.
- [ ] Run compatibility tests and projection gates, explicitly confirming
  bodyless entity AST/formatter hashes and legacy projection outputs.
- [ ] Run `cargo fmt --all -- --check`,
  `cargo clippy -p domainforge-core --all-targets --all-features -- -D warnings`,
  `just all-tests`, and `just prove`; record exact exit results and any skips.
- [ ] Review the final diff across correctness, security/path handling,
  compatibility, tests, complexity, and architecture; update handoff state.


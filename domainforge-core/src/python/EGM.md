# python — PyO3 Bindings (`domainforge` extension module)

## Overview
PyO3 wrappers over the canonical Rust core. `lib.rs` declares the
`#[pymodule] fn domainforge` entry point (under `feature = "python"`), which
registers every class and function defined here. These modules translate between
Python values and core types; they hold no business logic of their own.

## Interface
- `[mod.rs]` — module list: `authority`, `error`, `formatter`, `graph`, `policy`, `primitives`, `registry`, `semantic_pack`, `units`.
- `[graph.rs:Graph]` — `#[pyclass]` wrapper over `crate::graph::Graph`; `parse`, `parse_to_ast_json`, `resolve_application_contract_json`, `domain_model_identity_json`, `world_ref_from_identity_json`, `verify_world_ref`, `parse_world_ref`, plus `add_*`/`has_*`/`get_*`/`remove_*`/`find_*_by_name`/`all_*`/`*_count` accessors for entities, resources, flows, instances, roles, relations, policies, metrics, mappings, projections, dimensions, units, patterns, and concept changes.
- `[registry.rs:NamespaceBinding]` — `#[pyclass]` with `path`/`namespace` getters, `from_rust`, `into_inner`.
- `[registry.rs:NamespaceRegistry]` — `#[pyclass]` with `from_file`, `discover`, `resolve_files`, `namespace_for`, `root`, `default_namespace`, `from_rust`, `into_inner`.
- `[authority.rs:FinalDecision]`, `[authority.rs:PolicyModality]`, `[authority.rs:SourceClass]`, `[authority.rs:ClaimLevel]` — `#[pyclass]` mirrors of the core enums.
- `[authority.rs:AuthorityEnvironment]`, `[authority.rs:AuthorityErrorCode]`, `[authority.rs:evaluate_authority]` — authority surface.
- `[error.rs:to_python_exception]` — converts a `ValidationError` into a `PyErr`.
- `[primitives.rs:Entity]`/`Resource`/`Flow`/`ResourceInstance`/`Instance`/`Metric`/`Mapping`/`Projection`/`Pattern`/`ConceptChange`/`Policy`/`SourceDimension`/`SourceUnit`/`Role`/`Relation` — `#[pyclass]` wrappers; `Entity`, `Resource`, and `Flow` expose `from_rust`/`into_inner`.
- `[policy.rs:AggregateFunction]`, `BinaryOp`, `UnaryOp`, `Quantifier`, `Severity`, `WindowSpec`, `Expression`, `NormalizedExpression`, `Violation`, `EvaluationResult` — policy wrappers.
- `[units.rs:Dimension]` (`+parse/__repr__/__str__/from_rust/into_inner`), `[units.rs:Unit]` (`+new/symbol/name/base_unit/base_factor/__repr__/from_rust/into_inner`) — unit wrappers.
- `[formatter.rs:format_source]`, `[formatter.rs:check_format]` — `#[pyfunction]` formatter entry points.
- `[semantic_pack.rs:SemanticPack]`, `SemanticValidationResult`, and the enum mirrors `SemanticTruth`, `DiagnosticSeverity`, `ValidationMode`, `ApprovalState`, `SignatureState`, `ConceptStatus`, `ConceptKind`, `AliasStatus`, `UnknownConceptPolicy`, `DeprecatedPolicy`, `SemanticValidationStatus` — pack wrappers.
- `[semantic_pack.rs:build_semantic_pack]`, `validate_semantic_pack`, `validate_graph_with_pack`, `sign_pack`, `verify_pack_signature`, `diff_packs`, `compute_pack_hash`, `normalize_lookup_key`, `resolve_concept` — `#[pyfunction]` pack entry points.

## Invariants
- `[graph.rs:parse_concept_id]` — every `String` ID crossing the boundary is parsed into a `ConceptId` and a parse failure becomes a `PyResult` error, not a panic.
- `[error.rs:to_python_exception]` — core validation failures always surface as Python exceptions carrying the core error's code and message.
- `[mod.rs]` — these modules carry no duplicated business logic; behavior is delegated to `crate::graph`, `crate::policy`, `crate::semantic_pack`, and friends (repository rule: bindings must not duplicate Rust core logic).
- `[semantic_pack.rs:sign_pack]`/`[semantic_pack.rs:verify_pack_signature]` — the `#[cfg(not(feature = "signing"))]` variants return an error instead of silently succeeding.

## Side Effects
- `[registry.rs:NamespaceRegistry::from_file]`/`discover`/`resolve_files` — reach the filesystem through the core `NamespaceRegistry`, which reads `.sea-registry.toml` and glob-walks the tree.

## Verification
- Run: `cargo test -p domainforge-core --features cli`

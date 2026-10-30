# typescript — napi-rs Bindings

## Overview
napi-rs (`#[napi]`) wrappers over the canonical Rust core, exposed as the
TypeScript/Node package. `build.rs` calls `napi_build::setup()` under the
`typescript` feature. These modules translate JS values to core types and hold no
business logic of their own.

## Interface
- `[mod.rs]` — module list: `authority`, `error`, `formatter`, `graph`, `policy`, `primitives`, `registry`, `semantic_pack`, `units`.
- `[graph.rs:Graph]` — `#[napi]` wrapper (`#[napi(constructor)]`); `new`, `parse_to_ast_json`, `resolve_application_contract_json`, `domain_model_identity_json`, `world_ref_from_identity_json`, `verify_world_ref`, `parse_world_ref`, plus `add_entity`/`add_resource`/`add_flow`/`add_instance`/`add_role`/`add_relation`/`add_policy` and `*_count` counters (`entity_count`, `resource_count`, `flow_count`, `instance_count`, `pattern_count`, `role_count`, `relation_count`) plus `all_*`, `has_*`, `get_*`, `find_*_by_name`.
- `[registry.rs:NamespaceBinding]` — `#[napi]` with `#[napi(constructor)] new(path, namespace)` and `path`/`namespace` getters.
- `[registry.rs:NamespaceRegistry]` — `#[napi]` with `#[napi(factory)] from_file`, `discover`, `resolve_files`, `namespace_for`, `root`, `default_namespace`.
- `[authority.rs:FinalDecision]`, `[authority.rs:PolicyModality]`, `[authority.rs:SourceClass]`, `[authority.rs:ClaimLevel]` — `#[napi]` enum mirrors.
- `[authority.rs:evaluate_authority]`, `[authority.rs:EvaluateAuthorityResult]` — authority entry point and its result wrapper.
- `[error.rs:to_napi_error]` — converts a `ValidationError` into a napi `Error`.
- `[primitives.rs:Entity]`/`Resource`/`Flow`/`ResourceInstance`/`Instance`/`Metric`/`Mapping`/`Projection`/`Policy`/`DeclaredDimension`/`DeclaredUnit`/`Pattern`/`ConceptChange`/`Role`/`Relation` — `#[napi]` wrappers. Note `DeclaredDimension`/`DeclaredUnit` exist here but not in the Python surface.
- `[policy.rs:AggregateFunction]`, `BinaryOp`, `UnaryOp`, `Quantifier`, `Severity`, `WindowSpec`, `Expression`, `NormalizedExpression`, `Violation`, `EvaluationResult` — policy wrappers.
- `[units.rs:Dimension]` (`+parse/name`), `[units.rs:Unit]` (`+new/symbol/name/dimension/base_factor/base_unit/from_rust`) — unit wrappers.
- `[formatter.rs:FormatOptions]`, `[formatter.rs:format_source]`, `[formatter.rs:check_format]` — formatter entry points.
- `[semantic_pack.rs]` — the same enum mirrors as Python (`SemanticTruth`, `DiagnosticSeverity`, `ValidationMode`, `ApprovalState`, `SignatureState`, `ConceptStatus`, `ConceptKind`, `AliasStatus`, `SemanticValidationStatus`) plus `semantic_pack_build`, `semantic_pack_validate`, `semantic_pack_validate_graph`, `semantic_pack_sign`, `semantic_pack_verify`, `semantic_pack_diff`, `semantic_pack_hash`, `semantic_normalize_key`, `semantic_resolve_concept`.

## Invariants
- `[error.rs:to_napi_error]` — core validation failures surface as napi `Error::from_reason` carrying the core code and message; no panic crosses the boundary.
- `[registry.rs:NamespaceRegistry::from_file]` — uses `#[napi(factory)]` rather than a constructor, so the JS surface calls `NamespaceRegistry.from_file(...)` instead of `new`.
- `[mod.rs]` — these modules hold no duplicated business logic; all behavior is delegated to the Rust core (repository rule: bindings must not duplicate core logic).
- `[semantic_pack.rs:semantic_pack_sign]`/`semantic_pack_verify` — the `#[cfg(not(feature = "signing"))]` variants return an error instead of silently succeeding.

## Side Effects
- `[registry.rs:NamespaceRegistry::from_file]`/`discover`/`resolve_files` — reach the filesystem through the core `NamespaceRegistry` (reads `.sea-registry.toml`, glob-walks the tree).

## Verification
- Run: `cargo test -p domainforge-core --features cli`

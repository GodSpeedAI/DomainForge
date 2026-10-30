# domainforge — Python Package Module (`domainforge` import root)

## Overview
Import-root package for the `domainforge` Python distribution. `__init__.py` is a
pure re-export shim over the compiled PyO3 extension `domainforge.domainforge`
(the `*.so` files here are gitignored build outputs, not source);
`domainforge.pyi` is the checked-in type stub for that extension.

## Interface
- `[__init__.py:from .domainforge import (...)]` — re-exports 45 names from the compiled extension: `Entity`, `Resource`, `Flow`, `ResourceInstance`, `Instance`, `Role`, `Relation`, `Graph`, `Severity`, `Violation`, `EvaluationResult`, `Expression`, `NormalizedExpression`, `BinaryOp`, `UnaryOp`, `Quantifier`, `AggregateFunction`, `WindowSpec`, `FinalDecision`, `PolicyModality`, `SourceClass`, `ClaimLevel`, `AuthorityEnvironment`, `evaluate_authority`, `SemanticTruth`, `DiagnosticSeverity`, `ValidationMode`, `ApprovalState`, `SignatureState`, `ConceptStatus`, `ConceptKind`, `AliasStatus`, `SemanticValidationStatus`, `SemanticPack`, `SemanticValidationResult`, `build_semantic_pack`, `validate_semantic_pack`, `validate_graph_with_pack`, `sign_pack`, `verify_pack_signature`, `diff_packs`, `compute_pack_hash`, `normalize_lookup_key`, `resolve_concept`, `__version__`.
- `[__init__.py:__all__]` — declares exactly those 45 names as the public surface.
- `[domainforge.pyi:Entity]`, `[domainforge.pyi:Resource]`, `[domainforge.pyi:Flow]`, `[domainforge.pyi:ResourceInstance]`, `[domainforge.pyi:Instance]`, `[domainforge.pyi:Role]`, `[domainforge.pyi:Relation]` — domain primitives; each carries an `id`, optional `namespace`, and `set_attribute`/`get_attribute` (`set_field`/`get_field` plus a `fields_json` property on `Instance`).
- `[domainforge.pyi:Graph]` — the parse/query/export aggregate: `parse`, `parse_to_ast_json`, `import_calm`, `export_calm`, `export_protobuf`, `add_*`, `has_*`, `get_*`, `find_*_by_name`, `flows_from`, `flows_to`, `all_*`, `*_count`, `evaluate_policy`, `set_evaluation_mode`, `use_three_valued_logic`.
- `[domainforge.pyi:Expression]` — policy-expression constructors (`literal`, `variable`, `quantity`, `time`, `interval`, `binary`, `unary`, `cast`, `quantifier`, `member_access`, `aggregation`, `aggregation_comprehension`, `group_by`) plus `normalize`, `is_equivalent`, `__eq__`.
- `[domainforge.pyi:NormalizedExpression]` — `stable_hash`, `inner_expression`, `__hash__`, `__eq__`.
- `[domainforge.pyi:SemanticPack]` / `[domainforge.pyi:SemanticValidationResult]` — pack accessors (`to_json`, `pack_id`, `schema_version`, `approval_state`, `signature_state`, `concept_count`, `alias_count`, `meaning_version`, `meaning_fingerprint`, `pack_content_hash`; `status`, `diagnostics_json`, `unsigned_fixture_bypass_used`, `first_approved_version_bypass_used`) with `from_json` static constructors.
- `[domainforge.pyi:build_semantic_pack]`, `[domainforge.pyi:validate_semantic_pack]`, `[domainforge.pyi:validate_graph_with_pack]`, `[domainforge.pyi:sign_pack]`, `[domainforge.pyi:verify_pack_signature]`, `[domainforge.pyi:diff_packs]`, `[domainforge.pyi:compute_pack_hash]`, `[domainforge.pyi:normalize_lookup_key]`, `[domainforge.pyi:resolve_concept]` — module-level pack functions; all JSON-in/JSON-out (`build_semantic_pack` returns a `Tuple[str, List[str]]`).
- `[domainforge.pyi:AuthorityEnvironment]`, `[domainforge.pyi:evaluate_authority]` — authority evaluation; `evaluate(config_json, request_json, facts_json)` and the function form both return a `Tuple[str, str]` (trace JSON, decision JSON).
- `[domainforge.pyi:NamespaceRegistry]`, `[domainforge.pyi:Dimension]`, `[domainforge.pyi:Unit]`, `[domainforge.pyi:format_source]`, `[domainforge.pyi:check_format]`, `[domainforge.pyi:Policy]`, `[domainforge.pyi:Metric]`, `[domainforge.pyi:Mapping]`, `[domainforge.pyi:Projection]`, `[domainforge.pyi:Pattern]`, `[domainforge.pyi:ConceptChange]`, `[domainforge.pyi:SourceDimension]`, `[domainforge.pyi:SourceUnit]` — declared by the stub but **not** re-exported by `__init__.py`; reaching them requires addressing the extension module directly.

## Invariants
- `[__init__.py:__all__]` — the package-level public surface is exactly the 45 names listed in `__all__`; every one is also imported from `.domainforge`, and no other name is exported.

## Side Effects
- `[__init__.py:from .domainforge import (...)]` — importing the package loads the platform-specific compiled extension (`domainforge.cpython-*.so`); a missing or incompatible binary fails at import time.

## Verification
- Run: `just python-test`

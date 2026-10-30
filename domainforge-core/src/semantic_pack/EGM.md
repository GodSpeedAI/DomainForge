# semantic_pack — Semantic Pack Schema, Canonicalization, and Validation

## Overview
Implements the semantic pack document: schema types, canonical JSON and hashing
(content hash, meaning fingerprint, definition hash), pack-set merging, diffing,
alias/concept resolution, review-manifest coverage, and signature verification.
`signing.rs` and its re-exports are gated behind `#[cfg(feature = "signing")]`.

## Interface
- `[schema.rs:SemanticPack]` — top-level pack document (`pack_id`, `schema_version`, `trust`, `concepts`, ...).
- `[schema.rs:PackTrust]`, `[schema.rs:ApprovalState]`, `[schema.rs:SignatureState]`, `[schema.rs:GeneratorInfo]` — trust and provenance.
- `[schema.rs:ConceptDef]`, `[schema.rs:ConceptDefinition]`, `[schema.rs:ConceptKind]`, `[schema.rs:ConceptStatus]`, `[schema.rs:AliasDef]`, `[schema.rs:AliasStatus]`, `[schema.rs:RelationDef]`, `[schema.rs:MetricDef]`, `[schema.rs:DimensionDef]`, `[schema.rs:UnitDef]`, `[schema.rs:MappingRuleDef]`, `[schema.rs:CompatibilityInfo]` — concept surface.
- `[schema.rs:SourceRef]` (`+synthetic/workspace_root/pack_uri`), `[schema.rs:PackRef]`, `[schema.rs:PackSetRef]`, `[schema.rs:ReviewRecord]`, `[schema.rs:ReviewDecision]`, `[schema.rs:Suggestion]` — reference and review types.
- `[canonical_json.rs:canonical_json]`, `[canonical_json.rs:compute_sha256]`, `[canonical_json.rs:hash_canonical_json]`, `[canonical_json.rs:sort_pack_for_canonicalization]`, `[canonical_json.rs:sort_diagnostics]`, `[canonical_json.rs:compute_pack_content_hash]`, `[canonical_json.rs:compute_meaning_fingerprint]`, `[canonical_json.rs:compute_definition_hash]` — canonicalization and hashing.
- `[builder.rs:build_semantic_pack]`, `[builder.rs:PackBuildInput]`, `[builder.rs:PackBuildOutput]` — pack construction.
- `[validator.rs:validate_semantic_pack]`, `[validator.rs:validate_graph_with_pack]`, `[validator.rs:validate_term]`, `[validator.rs:SemanticPackError]` (`IoError`/`JsonError`/`ValidationError`) — validation entry points.
- `[diagnostics.rs:SemanticTruth]`, `[diagnostics.rs:DiagnosticSeverity]`, `[diagnostics.rs:SemanticDiagnosticCode]` (`+as_str`), `[diagnostics.rs:SemanticDiagnostic]`, `[diagnostics.rs:SemanticValidationResult]`, `[diagnostics.rs:SemanticValidationStatus]`, `[diagnostics.rs:ExpectedHashCheck]` — diagnostic model.
- `[diagnostics.rs:ValidationMode]`, `[diagnostics.rs:UnknownConceptPolicy]`, `[diagnostics.rs:DeprecatedPolicy]`, `[diagnostics.rs:ValidationOptions]` (`+strict`) — validation policy.
- `[diff.rs:diff_packs]`, `[diff.rs:PackDiff]`, `[diff.rs:DiffEntry]`, `[diff.rs:DiffClassification]`, `[diff.rs:DiffSummary]` — pack diffing.
- `[pack_set.rs:merge_packs]`, `[pack_set.rs:PackSet]`, `[pack_set.rs:PackConflict]`, `[pack_set.rs:ConflictType]` — pack-set merging.
- `[resolver.rs:normalize_lookup_key]`, `[resolver.rs:is_valid_canonical_id]`, `[resolver.rs:resolve_concept]`, `[resolver.rs:ResolveRequest]`, `[resolver.rs:ResolveResult]` (`+valid`), `[resolver.rs:detect_alias_conflicts]`, `[resolver.rs:AliasConflictReport]` — concept and alias resolution.
- `[review.rs:parse_review_manifest]`, `[review.rs:validate_review_coverage]`, `[review.rs:validate_definition_hashes]`, `[review.rs:compute_review_manifest_hash]`, `[review.rs:ReviewGap]`, `[review.rs:DefinitionHashResult]`, `[review.rs:ReviewError]` — review-manifest coverage.
- `[signing.rs:sign_pack]`, `[signing.rs:verify_pack_signature]`, `[signing.rs:derive_signer_id]`, `[signing.rs:build_signing_payload]`, `[signing.rs:SignOutput]`, `[signing.rs:SigningError]`, `[signing.rs:VerificationError]` — pack signing (feature-gated).
- `[mod.rs]` — re-exports the builder, canonical-JSON, diagnostics, diff, pack-set, resolver, schema, validator, and (under `signing`) signing surfaces.

## Invariants
- `[validator.rs:CURRENT_SCHEMA_VERSION]` — a pack whose `schema_version` is not `"0.3"` produces a `SemanticDiagnosticCode::PackSchemaMismatch` error diagnostic.
- `[validator.rs:validate_graph_with_pack]` — with `options.require_signed_pack`, an unsigned pack blocks validation (`SemanticValidationStatus::Blocked` with `PackUnsigned`) unless `options.allow_unsigned_test_fixtures` is set.
- `[canonical_json.rs:canonical_json]` — canonical JSON is UTF-8 with recursively sorted object keys and no whitespace; `sort_pack_for_canonicalization` must be applied before hashing so collection order cannot change the digest.
- `[canonical_json.rs:compute_pack_content_hash]` — the content hash excludes signature and build-variant metadata (`created_at`, `review_manifest_hash`, `trust.signed_by`, `trust.signature_alg`, `trust.signature`, and `signature_state` is reset to `Unsigned`), so signing a pack does not change its content hash.
- `[canonical_json.rs:compute_pack_content_hash]` / `[canonical_json.rs:compute_meaning_fingerprint]` / `[canonical_json.rs:compute_definition_hash]` — content hash, meaning fingerprint, and per-definition hash are distinct digests over distinct projections of the pack; they are not interchangeable.
- `[signing.rs:verify_pack_signature]` — the signature must verify against `build_signing_payload(pack, content_hash)`; a mismatch yields `SigningError`/`VerificationError`, never a silent pass.
- `[resolver.rs:is_valid_canonical_id]` — a resolved `ResolveResult` must carry a valid canonical concept ID; `ResolveResult::valid` is the only constructor for a successful resolution.
- `[mod.rs]` — `signing` is compiled only under the `signing` feature; callers must not assume `sign_pack` exists without it.

## Side Effects
- None observed in source. `SemanticPackError::IoError` exists as an error variant but this module performs no filesystem writes; pack I/O is done by `src/cli/pack.rs`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`

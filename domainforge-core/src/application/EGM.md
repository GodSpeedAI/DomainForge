# application — Application Contract, Canonical Envelope, and World Identity

## Overview
Resolves SEA source sets into the Application Contract document and the Canonical
Semantic Envelope (CEP-0008), plus typed policy precondition evaluation and
named semantic-world identity (`WorldRef`). This is the multi-module front end
that `parser::parse_to_graph` delegates to when a file declares entity bodies.

## Interface
- `[envelope.rs:ENVELOPE_SCHEMA_VERSION]` — `"domainforge-semantic-envelope/v1"` schema tag.
- `[envelope.rs:CANONICALIZATION_VERSION]` — `"domainforge-canonicalization/v1"` canonicalization tag.
- `[envelope.rs:CEP_SNAPSHOT_KNOWN_OMISSIONS]` / `[envelope.rs:CEP_SNAPSHOT_OMISSION_REASON]` / `[envelope.rs:CEP_SNAPSHOT_OMISSION_RECOVERABLE_FROM]` — declared snapshot gaps and how they can be recovered.
- `[envelope.rs:CEP_ENVELOPE_VERSION]` (`"1.0.0"`), `[envelope.rs:CEP_ENVELOPE_KIND_SEMANTIC_SNAPSHOT]`, `[envelope.rs:CEP_PRODUCER_NAME]`, `[envelope.rs:CEP_VERSION]`, `[envelope.rs:CEP_PROFILE_SEMANTIC_SNAPSHOT]`, `[envelope.rs:CEP_PROFILE_VERSION]` — envelope constants.
- `[envelope.rs:CanonicalSemanticEnvelopeDocument]`, `[envelope.rs:CanonicalSemanticEnvelope]`, `[envelope.rs:CanonicalModuleRef]`, `[envelope.rs:CanonicalImportEdge]`, `[envelope.rs:CanonicalNamespaceBinding]`, `[envelope.rs:CanonicalSemanticPackRef]` — document/envelope scaffolding.
- `[envelope.rs:CanonicalSemanticDeclaration]`, `[envelope.rs:CanonicalDeclarationId]`, `[envelope.rs:CanonicalReferenceTarget]`, `[envelope.rs:CanonicalResolvedReference]`, `[envelope.rs:CanonicalSemanticPayload]` — declaration and reference model.
- `[envelope.rs:CanonicalDimensionDecl]`, `[envelope.rs:CanonicalUnitDecl]`, `[envelope.rs:CanonicalEntityDecl]`, `[envelope.rs:CanonicalResourceDecl]`, `[envelope.rs:CanonicalFlowDecl]`, `[envelope.rs:CanonicalPatternDecl]`, `[envelope.rs:CanonicalRoleDecl]`, `[envelope.rs:CanonicalRelationDecl]`, `[envelope.rs:CanonicalRoleBindingDecl]`, `[envelope.rs:CanonicalInstanceDecl]`, `[envelope.rs:CanonicalPolicyDecl]`, `[envelope.rs:CanonicalConceptChangeDecl]`, `[envelope.rs:CanonicalMetricDecl]` — per-kind payload structs.
- `[envelope.rs:resolve_semantic_envelope]`, `[envelope.rs:resolve_semantic_envelope_with_packs]` — resolve a source set into the canonical envelope.
- `[envelope.rs:semantic_closure_hash]`, `[envelope.rs:envelope_document_self_hash]` — content-addressing helpers.
- `[envelope.rs:validate_application_contract_document_json]`, `[envelope.rs:validate_semantic_envelope_document_json]` — JSON-document validators.
- `[envelope.rs:build_cep_envelope]` with `[envelope.rs:CepEnvelopeParams]` / `[envelope.rs:CepWorldParams]` — emit the CEP snapshot envelope.
- `[envelope.rs:declaration_entity_ref]`, `[envelope.rs:invalid_declared_checkpoint_hash]` — reference/hash helpers.
- `[envelope.rs:DomainModelIdentity]` (`+canonical_digest`) — model identity used for world refs.
- `[resolve.rs:resolve_application_contract]`, `[resolve.rs:resolve_application_contract_with_packs]`, `[resolve.rs:resolve_application_contract_json]`, `[resolve.rs:resolve_application_graph]` — resolution entry points.
- `[resolve.rs:APPLICATION_CONTRACT_SCHEMA_VERSION]` (`"domainforge-application-contract/v1"`), `[resolve.rs:LANGUAGE_SCHEMA_VERSION]` (`"domainforge-ast/v3"`), `[resolve.rs:INTERPRETATION_VERSION]` (`"domainforge-interpretation/v1"`), `[resolve.rs:SOURCE_MAP_MAX_MODULES]` (256), `[resolve.rs:SOURCE_MAP_MAX_BYTES]` (2 MiB) — resolution limits and tags.
- `[contract.rs:ApplicationContract]`, `[contract.rs:ApplicationContractDocument]`, `[contract.rs:ApplicationContractInputs]`, `[contract.rs:ProducerIdentity]`, `[contract.rs:ApplicationSymbolId]` — contract shell.
- `[contract.rs:EntityContract]`, `[contract.rs:RecordContract]`, `[contract.rs:EnumContract]`, `[contract.rs:EnumMember]`, `[contract.rs:FieldContract]`, `[contract.rs:FieldType]`, `[contract.rs:FieldConstraint]`, `[contract.rs:ScalarType]` — typed entity surface.
- `[contract.rs:OperationContract]` with clause types `[contract.rs:Direction]`, `[contract.rs:ActorRef]`, `[contract.rs:EnforcementPoint]`, `[contract.rs:PolicyBinding]`, `[contract.rs:AccessMode]`, `[contract.rs:EffectKind]`, `[contract.rs:TransactionBoundary]`, `[contract.rs:FailureKind]`, `[contract.rs:FailureContract]`, `[contract.rs:NotApplicable]`, `[contract.rs:IdempotencyStrategy]`, `[contract.rs:ConcurrencyStrategy]`, `[contract.rs:EvidenceKind]`, `[contract.rs:LifecycleKind]` — operation surface.
- `[contract.rs:ApplicationPolicyContext]`, `[contract.rs:TypedValue]` — typed evaluation context.
- `[policy_context.rs:evaluate_precondition]`, `[policy_context.rs:EvaluationResult]` — application-owned typed precondition evaluation.
- `[canonical.rs:nfc]`, `[canonical.rs:canonical_decimal]`, `[canonical.rs:canonical_typed_value]`, `[canonical.rs:source_set_hash]`, `[canonical.rs:semantic_pack_set_hash]`, `[canonical.rs:input_fingerprint]`, `[canonical.rs:document_self_hash]` — canonicalization primitives.
- `[world.rs:SUPPORTED_IDENTITY_SCHEME]` (`"v2-full-preimage"`), `[world.rs:WorldRefError]`, `[world.rs:WorldName]`, `[world.rs:WorldRef]` (`+from_identity/name/digest/verify_identity`), `[world.rs:WorldAlias]`, `[world.rs:WorldLabel]`, `[world.rs:WorldCatalog]` (`+register/lookup/verify/retarget_alias/resolve_alias/alias_history/pin`), `[world.rs:domain_model_identity_json]`, `[world.rs:world_ref_from_identity_json]`, `[world.rs:verify_world_ref_json]`, `[world.rs:canonical_world_ref]` — semantic-world identity and alias catalog.
- `[diagnostic.rs:ApplicationDiagnosticCode]` (`+all/all_code_slugs/code/severity/slug`), `[diagnostic.rs:ApplicationDiagnostic]` (`+new/closure_error/at/with_document_kind`), `[diagnostic.rs:ApplicationDiagnosticContext]`, `[diagnostic.rs:sort_diagnostics]` — diagnostic model.
- `[diagnostic.rs:APP014_SYMBOL_COLLISION]`, `[diagnostic.rs:APP014_IMPORT_CYCLE]`, `[diagnostic.rs:APP014_UNRESOLVED_SPECIFIER]`, `[diagnostic.rs:APP014_UNRESOLVED_ALIAS]`, `[diagnostic.rs:APP014_NOT_EXPORTED]` — stable APP014 diagnostic slugs.
- `[verification_contract.rs:SEMANTIC_VERIFICATION_PROFILE]` (`"domainforge-semantic-verification/v1"`), `[verification_contract.rs:ENVELOPE_KIND_WORK_REQUEST]` (`"work_request"`), `[verification_contract.rs:ObligationKind]`, `[verification_contract.rs:VerificationObligation]`, `[verification_contract.rs:build_verification_contract]` — §14.0 work-request verification obligations.
- `[validate.rs]` — resolver-side validation module, declared as `pub(crate) mod validate;` in `[mod.rs]`; not part of the public surface.
- `[mod.rs]` — re-exports `evaluate_precondition`, `EvaluationResult`, the `resolve_*` functions, canonicalization helpers, and the diagnostic/envelope/contract types.

## Invariants
- `[policy_context.rs:evaluate_precondition]` — missing facts, `UNKNOWN`, and evaluator errors fail closed: `False`, `Unknown`, and `Error` are mapped by the caller to the bound failure code; the evaluator never reads `Graph` evaluation state.
- `[resolve.rs:SOURCE_MAP_MAX_MODULES]` / `[resolve.rs:SOURCE_MAP_MAX_BYTES]` — a resolved source map may not exceed 256 modules or 2 MiB.
- `[envelope.rs:CEP_SNAPSHOT_KNOWN_OMISSIONS]` — the emitted snapshot must enumerate its known omissions rather than silently dropping them; each omission is paired with `CEP_SNAPSHOT_OMISSION_REASON` and `CEP_SNAPSHOT_OMISSION_RECOVERABLE_FROM`.
- `[world.rs:WorldRef::verify_identity]` / `[world.rs:WorldCatalog::verify]` — a world ref must re-derive to the same identity digest; `[world.rs:SUPPORTED_IDENTITY_SCHEME]` pins the scheme to `v2-full-preimage`.
- `[canonical.rs:document_self_hash]` / `[envelope.rs:envelope_document_self_hash]` — the document self-hash excludes the hash field itself, so byte-identical inputs must produce identical output.
- `[canonical.rs:nfc]` — canonical text is NFC-normalized before hashing.
- `[verification_contract.rs:ObligationKind]` — each obligation kind maps to a fixed evidence type and projection family (`flow_behavior`→`test_result`/`gauge`, `policy_proof`→`conformance_result`/`lean`, `availability`→`configuration_snapshot`/`gauge`).

## Side Effects
- None observed in source. This module performs no filesystem, network, or global-state mutation; filesystem walking is delegated to `crate::module::resolver::source_map_from_filesystem`, and output writing is done by `src/cli/envelope.rs`.

## Verification
- Run: `cargo test -p domainforge-core --features cli`

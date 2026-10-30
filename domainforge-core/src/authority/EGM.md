# authority — Deterministic Authority Decision Engine

## Overview
Evaluates authority requests against a versioned policy pack set, producing a
`FinalDecision` with a full conflict-resolution trace. Resolution orders
candidates by specificity vectors, evaluates three-valued fact conditions, and
records every step for audit.

## Interface
- `[types.rs:FinalDecision]` (`+precedence_rank`) — `Allow`/`Deny`/`Escalate`/`NotApplicable`/`Reject`, ranked 2/4/3/1/5.
- `[types.rs:PolicyModality]` — `Permission`/`Prohibition`/`Obligation`/`Override`.
- `[types.rs:SourceClass]`, `[types.rs:ClaimLevel]` — fact provenance and claim strength.
- `[types.rs:AuthorityRequest]` (`+validate/action_hash_input`), `[types.rs:ActorContext]`, `[types.rs:ResourceRef]` — request model.
- `[types.rs:FactEnvelope]` (`+is_fresh/is_trusted/is_caller_supplied/effective_trust`), `[types.rs:FactRequirement]` (`+is_satisfied_by`), `[types.rs:DerivedFactLineage]` — fact model.
- `[types.rs:FactSource]`, `[types.rs:FactTransform]`, `[types.rs:TransformInput]`, `[types.rs:TransformOutput]`, `[types.rs:PurityFlags]` (`+is_pure`) — fact sourcing and derivation.
- `[types.rs:SpecificityProfile]` (`+default_profile`), `[types.rs:SpecificityVector]` (`+new/compare`), `[types.rs:SpecificityComparison]` — specificity ordering.
- `[types.rs:UnknownHandlingConfig]`, `[types.rs:UnknownHandlingDefault]` (`+defaults/for_modality/validate`) — fail-closed defaults per modality.
- `[types.rs:AuthorityDecision]`, `[types.rs:UnknownDecision]`, `[types.rs:FactTrustDecision]`, `[types.rs:CompatibilityLoweringDecision]`, `[types.rs:ConditionEvaluation]`, `[types.rs:ThreeValuedResult]` — decision/trace payloads.
- `[types.rs:validate_fact_path]`, `[types.rs:compute_hash]`, `[types.rs:compute_deterministic_hash]`, `[types.rs:generate_decision_id]` — validation and deterministic hashing.
- `[resolver.rs:AuthorityResolver]` (`+new/resolve/validate_versions`), `[resolver.rs:PolicyEvaluation]`, `[resolver.rs:ConflictResolutionStep]`, `[resolver.rs:ResolverOutput]` — the resolution pipeline.
- `[policy.rs:AuthorityPolicy]` (`+validate/is_candidate/evaluate_conditions/check_fact_requirements/compute_specificity`), `[policy.rs:StructuralPredicates]` (`+matches/specificity_score`), `[policy.rs:ConditionPredicates]` (`+evaluate/condition_specificity_score`), `[policy.rs:OverrideSpec]`, `[policy.rs:ObligationSpec]` — policy model and predicate scoring.
- `[compiler.rs:PolicyCompiler]` (`+new/compile/semantics_version/compatibility_version`), `[compiler.rs:CompatibilityLoweringAuditor]` (`+new/version/audit_expression`), `[compiler.rs:CompatibilityExpression]`, `[compiler.rs:LoweredPolicy]`, `[compiler.rs:RawPolicy]`, `[compiler.rs:RawFactRequirement]` — compatibility-expression lowering.
- `[environment.rs:AuthorityEnvironment]` (`+new/validate/evaluate/packs/config`), `[environment.rs:AuthorityEnvironmentConfig]` — the host environment wrapper.
- `[pack.rs:AuthorityPack]` (`+validate/validate_hash/validate_semantics_version/identity_key`), `[pack.rs:compute_pack_hash]` — policy pack integrity.
- `[fact_resolver.rs:FactResolver]` (`+new/wrap_context_as_caller_supplied/resolve_trusted_facts`), `[fact_resolver.rs:FactSourceRegistry]` (`+new/register/get/validate`), `[fact_resolver.rs:TrustedFacts]` (`+as_slice`) — trusted-fact resolution.
- `[transform.rs:FactTransformRegistry]` (`+new/register/get/validate`), `[transform.rs:DerivedFactEngine]` (`+new/compute_derived_facts`) — derived facts.
- `[trace.rs:AuthorityTrace]`, `[trace.rs:FactEnvelopeSummary]`, `[trace.rs:ConflictStepRecord]`, `[trace.rs:AuthorityTraceEmitter]` (`+new/get_traces/clear_traces/emit`), `[trace.rs:EvidenceSink]` — audit trace emission.
- `[signing.rs:build_signing_payload]`, `[signing.rs:verify_fact_signature]`, `[signing.rs:VerificationError]` — fact signature verification; the module is gated by `#[cfg(feature = "signing")]` in `[mod.rs]`.
- `[error.rs:AuthorityErrorCode]`, `[error.rs:AuthorityError]` (`+new/recoverable/with_hint/with_context` and named constructors `missing_config`, `invalid_environment`, `fact_trust_violation`, `source_unavailable`, `invalid_transform`, `specificity_conflict`, `ambiguous_lowering`, `trace_persistence_failure`, `conflicting_specificity_profile`, `pack_hash_mismatch`, `semantics_hash_mismatch`, `invalid_request`) — error surface.
- `[mod.rs]` — re-exports the compiler, environment, error, fact_resolver, pack, policy, resolver, trace, and transform types plus `types::*`.

## Invariants
- `[types.rs:FinalDecision::precedence_rank]` — decision precedence is fixed: `Reject`(5) > `Deny`(4) > `Escalate`(3) > `Allow`(2) > `NotApplicable`(1).
- `[types.rs:UnknownHandlingDefault::validate]` — unknown/absent facts must resolve through the per-modality default, never to an implicit allow.
- `[pack.rs:AuthorityPack::validate_hash]` — a pack must validate its own content hash before use; `[error.rs:pack_hash_mismatch]` is the failure channel.
- `[pack.rs:AuthorityPack::validate_semantics_version]` — a pack may only be used when its semantics version is supported.
- `[resolver.rs:validate_versions]` — resolver version validation must run before resolution.
- `[transform.rs:PurityFlags::is_pure]` — impure transforms are not eligible for deterministic derived-fact computation.
- `[policy.rs:ObligationSpec::validate]` — obligations are validated before they participate in a decision.
- `[signing.rs:verify_fact_signature]` — a signed fact must verify against its signing payload, otherwise it is not trusted.

## Side Effects
- `[trace.rs:AuthorityTraceEmitter::emit]` — accumulates traces into emitter-owned state via interior mutability (`get_traces`/`clear_traces` mutate the same store); `[trace.rs:EvidenceSink]` selects where emitted evidence is written.
- `[environment.rs:AuthorityEnvironment::validate]` — takes `&mut self`; validation mutates the environment's validated state.

## Verification
- Run: `cargo test -p domainforge-core --features cli`

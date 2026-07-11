---
type: CEP Specification
id: CEP-0008
title: Semantic Envelope
description: This document defines the CEP Semantic Envelope.
status: Draft
version: 0.1.0
tags:
- cep
timestamp: '2026-06-26T14:13:47Z'
protocol: Canonical Evaluation Protocol
category: Normative
document_type: Foundational Specification
depends_on:
- CEP-0000
- CEP-0001
- CEP-0002
- CEP-0003
- CEP-0004
- CEP-0005
- CEP-0006
- CEP-0007
supersedes: []
superseded_by: []
conformance_class: null
benchmark_family: null
schema_ref: null
defines:
- Semantic Envelope
- Envelope Identity
- Envelope Boundary
- Envelope Profile
- Envelope Completeness
used_by:
- CEP-0009
- CEP-0010
related:
- CEP-0007
- CEP-0009
---

# CEP-0008 — Semantic Envelope

## Navigation

**Depends on:** [CEP-0000](./CEP-0000-derivation-ladder.md), [CEP-0001](./CEP-0001-ontology.md), and the prior protocol chain through [CEP-0007](./CEP-0007-settlement-model.md)
**Defines:** Semantic Envelope, Envelope Identity, Envelope Boundary, Envelope Profile, Envelope Completeness
**Used by:** [CEP-0009](./CEP-0009-benchmark-model.md), [CEP-0010](./CEP-0010-conformance.md)

## Abstract

This document defines the CEP Semantic Envelope.

A Semantic Envelope is the canonical transport structure for preserving CEP-relevant context across tools, agents, benchmarks, systems, organizations, and transformations.

A Semantic Envelope is not merely a message.

A Semantic Envelope is a portable transformation context.

It carries enough represented structure to preserve constraints, distinctions, relations, representations, states, transformations, observations, questions, claims, evidence, settlement records, authority context, affordance context, capability context, provenance, uncertainty, omissions, identity, and conformance metadata.

The purpose of the Semantic Envelope is to make represented transformations interoperable without collapsing the distinctions required for CEP evaluation.

## 1. Purpose

The purpose of the Semantic Envelope is to prevent context loss during handoff.

CEP evaluations often fail because systems pass raw text, raw JSON, raw logs, model outputs, test results, or task summaries without preserving what those materials mean in relation to state, transformation, question, evidence, and settlement.

A Semantic Envelope exists so that a transformation can move across systems while preserving:

- what was represented
- what state was involved
- what transformation was attempted, claimed, observed, or settled
- what question made the transformation relevant
- what evidence supports or contradicts claims
- what authority constraints applied
- what settlement status was reached
- what affordances changed
- what capability update may be justified

what uncertainty, omission, contradiction, or debt remains.

A CEP-compatible implementation MUST preserve these distinctions when transporting evaluation context.

## 2. Scope

This document defines:
- Semantic Envelope definition
- envelope identity
- envelope boundary
- envelope modes
- envelope required sections
- entity carriage
- reference semantics
- completeness semantics
- provenance
- integrity
- versioning
- merge and split behavior
- transformation handoff
- authority context
- evidence context
- settlement context
- benchmark context
- conformance requirements
- anti-collapse rules

This document does not define:
- final JSON Schema
- final YAML syntax
- encryption standard
- registry protocol
- storage backend
- runner implementation
- benchmark scoring algorithms

Those are defined in later CEP specifications or implementation profiles.

## 3. Core Definition

A Semantic Envelope is a bounded, versioned, provenance-bearing, CEP-compatible container that preserves transformation-relevant representations and their relationships.

A Semantic Envelope MUST be capable of carrying references to:

- constraints
- distinctions
- relations
- representations
- states
- transformations
- observations
- perspectives
- questions
- claims
- evidence
- settlements
- actors
- authority records
- affordances
- capabilities
- metrics
- benchmarks
- provenance
- identity records
- omissions
- uncertainty
- contradictions
- debt records
- conformance metadata

A Semantic Envelope MAY omit entities not relevant to the evaluation context.

When materially relevant entities are omitted, the envelope MUST preserve whether those entities are:

- intentionally_absent
- unknown
- unavailable
- out_of_scope
- redacted
- not_collected
- not_applicable
- compressed
- lost

## 4. Semantic Envelope Is Not Raw Context

CEP SHALL distinguish Semantic Envelope from raw context.

### 4.1 Raw Context

Raw context is unstructured or weakly structured material supplied to a system.

- Examples:
- prompt text
- source file
- log dump
- trace export
- email
- policy document
- user instruction
- model output
- screenshot
- test output
- conversation history.

### 4.2 Semantic Envelope

A Semantic Envelope preserves the evaluative meaning of context.

It identifies which parts of the context function as representation, state, transformation, question, evidence, claim, settlement, authority, capability, affordance, or omission.

### 4.3 Anti-Collapse Rule

A CEP implementation MUST NOT treat raw context as a Semantic Envelope unless the required CEP structure is present or derivable and validated.

## 5. Semantic Envelope Is Not Reality

A Semantic Envelope preserves representations of reality.

It does not contain reality.

It MUST preserve the representation boundary defined in CEP-0002.

### 5.1 Boundary Rule

A Semantic Envelope MUST NOT imply that contained material is complete, objective, current, verified, settled, authorized, or true unless those statuses are explicitly represented and evidenced.

## 6. Semantic Envelope Is Not Settlement

A Semantic Envelope may carry settlement records.

It is not settlement by itself.

Transporting context does not settle a transformation.

A Semantic Envelope may be:
- pre_settlement
- settlement_pending
- settlement_ready
- settled
- partially_settled
- unsettled
- invalid_for_settlement
- unknown

### 6.1 Anti-Collapse Rule

A CEP implementation MUST NOT treat the existence of a Semantic Envelope as evidence that the transformation inside it has settled.

## 7. Envelope Identity

A Semantic Envelope MUST have identity.

Envelope identity allows CEP to distinguish:
- original envelope from revised envelope
- source envelope from derived envelope
- partial envelope from complete envelope
- merged envelope from component envelope
- signed envelope from unsigned envelope
- current envelope from superseded envelope

### 7.1 Required Identity Fields

A Semantic Envelope MUST preserve:
- envelope_id
- cep_version
- envelope_version
- created_at
- created_by
- scope
- envelope_kind
- lineage_refs

### 7.2 Recommended Identity Fields

A Semantic Envelope SHOULD preserve:
- content_hash
- semantic_hash
- signature_refs
- supersedes
- superseded_by
- parent_envelope_refs
- child_envelope_refs
- source_system_refs
- tenant_or_org_ref
- classification
- retention_policy

### 7.3 Identity Failure

If envelope identity is absent, CEP cannot determine whether context was updated, replaced, merged, split, tampered with, summarized, or superseded.

## 8. Envelope Boundary

Every Semantic Envelope has a boundary.

The boundary defines what transformation context is included and what remains omitted, unknown, or out of scope.

### 8.1 Boundary Record

An envelope boundary SHOULD preserve:
- scope
- included_sections
- excluded_sections
- known_omissions
- unknowns
- redactions
- compression_notes
- out_of_scope_entities
- classification
- limitations

### 8.2 Boundary Failure

If envelope boundary is absent, downstream systems may treat missing context as nonexistent.

This causes false authority, false settlement, false capability, and invalid benchmark scoring.

## 9. Envelope Kinds

CEP recognizes the following Semantic Envelope kinds.

- work_request
- context_bundle
- authority_request
- authority_decision
- execution_trace
- evidence_packet
- settlement_packet
- capability_update
- affordance_update
- benchmark_case
- benchmark_result
- conformance_report
- semantic_snapshot
- semantic_diff
- semantic_trace
- composite
- unknown

### 9.1 Work Request Envelope

Carries the question, target transformation, scope, constraints, authority requirements, and expected evidence for proposed work.

### 9.2 Context Bundle Envelope

Carries relevant representations, states, constraints, and prior transformations used to perform or evaluate work.

### 9.3 Authority Request Envelope

Carries actor, proposed transformation, target resources, authority constraints, and decision question.

### 9.4 Authority Decision Envelope

Carries allow, deny, escalate, or unknown decision with evidence and authority rationale.

### 9.5 Execution Trace Envelope

Carries attempted actions, observed transformations, logs, diffs, outputs, and execution evidence.

### 9.6 Evidence Packet Envelope

Carries evidence records bound to claims, questions, transformations, or settlements.

### 9.7 Settlement Packet Envelope

Carries settlement question, evidence sufficiency, consequence records, settlement status, debt, and state update recommendation.

### 9.8 Capability Update Envelope

Carries repeated settlement history, variation context, payment burden delta, affordance delta, and capability claim.

### 9.9 Benchmark Case Envelope

Carries case states, questions, transformations, evidence requirements, answer keys, metrics, and scoring context.

### 9.10 Benchmark Result Envelope

Carries benchmark outputs, scores, evidence traces, conformance status, errors, warnings, and result provenance.

### 9.11 Semantic Snapshot Envelope

Carries a bounded state of a system or organization at a time or version.

### 9.12 Semantic Diff Envelope

Carries represented differences between two or more states, snapshots, envelopes, or transformation histories.

### 9.13 Semantic Trace Envelope

Carries an ordered transformation chain and its related evidence, settlement, authority, and capability updates.

## 10. Envelope Completeness

A Semantic Envelope MUST preserve completeness status.

Completeness is context-specific.

An envelope may be complete for one question and incomplete for another.

### 10.1 Completeness Values

Completeness SHOULD be one of:
- complete_for_context
- partial
- minimal
- unknown
- invalid
- out_of_scope

### 10.2 Section Completeness

Each major envelope section SHOULD preserve its own completeness status.

- Example:
- states: complete_for_context
- evidence: partial
- authority: unknown
- settlement: not_applicable
- capability: out_of_scope

### 10.3 Completeness Failure

If completeness is not represented, downstream systems may assume the envelope contains all required context.

This is invalid.

## 11. Envelope Section Model

A CEP-compatible Semantic Envelope SHOULD support the following top-level sections.

- metadata
- boundary
- references
- constraints
- distinctions
- relations
- representations
- states
- transformations
- observations
- perspectives
- questions
- claims
- evidence
- settlements
- actors
- authority
- affordances
- capabilities
- metrics
- benchmarks
- provenance
- identity
- omissions
- uncertainty
- contradictions
- debt
- integrity
- conformance
- extensions

An implementation MAY store these sections inline, by reference, or through a hybrid structure.

If a section is omitted, the omission status MUST be preserved when material.

## 12. Minimal Envelope Metadata

A Semantic Envelope MUST preserve minimal metadata.

- envelope_id
- cep_version
- envelope_version
- envelope_kind
- created_at
- created_by
- scope
- completeness_status
- provenance_refs
- boundary_record

An envelope lacking metadata is not CEP-valid.

## 13. References

A Semantic Envelope MAY carry entities inline or by reference.

### 13.1 Reference Record

A reference record SHOULD preserve:
- ref_id
- ref_type
- target_uri_or_id
- target_hash
- target_version
- access_method
- availability_status
- integrity_status
- scope
- limitations

### 13.2 Reference Types

Reference types MAY include:
- local_id
- file_path
- uri
- content_hash
- database_id
- trace_id
- commit_hash
- document_id
- artifact_id
- external_record_id
- envelope_id
- semantic_hash
- unknown

### 13.3 Reference Availability

Availability SHOULD be one of:
- available
- unavailable
- redacted
- permission_denied
- expired
- unknown
- not_applicable

### 13.4 Reference Failure

A Semantic Envelope that references external material MUST preserve enough information to retrieve, verify, or mark the reference unavailable.

Dangling references MUST be marked.

## 14. Inline vs Referenced Carriage

A Semantic Envelope MAY carry content inline or by reference.

### 14.1 Inline Carriage

Inline carriage is appropriate when:
- content is small
- content is required for portability
- benchmark cases require self-contained evaluation
- references cannot be trusted to remain available

### 14.2 Referenced Carriage

Referenced carriage is appropriate when:
- content is large
- content is sensitive
- content already exists in durable storage
- content must remain access-controlled
- only hash verification is needed

### 14.3 Hybrid Carriage

Hybrid carriage MAY include a summary inline and source reference externally.

If hybrid carriage is used, the envelope MUST NOT treat the summary as equivalent to the source unless equivalence is declared and evidenced.

## 15. Provenance

A Semantic Envelope MUST preserve provenance.

### 15.1 Envelope Provenance Record

Envelope provenance SHOULD preserve:
- provenance_id
- source_envelope_refs
- source_representation_refs
- source_system_refs
- producer_refs
- production_method
- created_at
- derived_from
- transformation_refs
- tool_refs
- model_refs
- human_refs
- review_status
- verification_status

### 15.2 Provenance Failure

If provenance is absent, CEP cannot distinguish original context from derived context, generated interpretation, normalized envelope, or benchmark result.

An envelope without provenance is not settlement-valid.

## 16. Integrity

A Semantic Envelope SHOULD preserve integrity records.

Integrity is REQUIRED when envelopes are used for authority, settlement, benchmark scoring, audit, or conformance.

### 16.1 Integrity Record

An integrity record SHOULD preserve:
- content_hash
- semantic_hash
- signature_refs
- verification_method
- verification_status
- tamper_status
- checked_at
- checker_ref

### 16.2 Verification Status

Verification status SHOULD be one of:
- verified
- unverified
- failed
- unknown
- not_applicable

### 16.3 Tamper Status

Tamper status SHOULD be one of:
- intact
- modified
- tampered
- unknown
- not_applicable

### 16.4 Integrity Failure

If integrity cannot be established, the envelope MAY still be used for exploration.

It SHOULD NOT be used for high-stakes settlement, authority, or benchmark scoring without qualification.

## 17. Versioning

A Semantic Envelope MUST preserve versioning.

### 17.1 Version Record

A version record SHOULD preserve:
- cep_version
- envelope_version
- schema_version
- profile_version
- extension_versions
- created_at
- updated_at
- supersedes
- superseded_by
- migration_refs
- compatibility_status

### 17.2 Compatibility Status

Compatibility SHOULD be one of:
- compatible
- partially_compatible
- requires_migration
- incompatible
- unknown

### 17.3 Version Failure

If versioning is absent, tools cannot safely interpret envelope structure.

## 18. Profiles

A Semantic Envelope MAY conform to one or more profiles.

A profile defines additional requirements for a domain, benchmark family, tool, or evaluation context.

### 18.1 Profile Examples

- reality_gap_profile
- authority_profile
- settlement_profile
- capability_profile
- benchmark_case_profile
- benchmark_result_profile
- audit_profile
- agent_trace_profile
- organizational_snapshot_profile

### 18.2 Profile Record

A profile record SHOULD preserve:
- profile_id
- profile_version
- required_sections
- required_fields
- validation_rules
- conformance_status

### 18.3 Profile Failure

If a tool assumes a profile but the envelope does not declare it, interpretation may be invalid.

## 19. Constraints Section

The constraints section carries represented constraints relevant to the envelope.

It SHOULD preserve:
- constraint_id
- constraint_type
- description
- scope
- source
- status
- applies_to
- evidence_refs
- confidence

Constraints MAY be carried inline or by reference.

[Constraint](./CEP-0001-ontology.md) status MUST NOT be inferred from absence.

## 20. Distinctions Section

The distinctions section carries preserved differences.

It SHOULD preserve:
- distinction_id
- left_term
- right_term
- basis
- distinction_kind
- scope
- source
- confidence
- evidence_refs

A Semantic Envelope used for Reality Gap, Authority, [Settlement](./CEP-0007-settlement-model.md), Capability, or Affordance evaluation MUST preserve the distinctions required by that evaluation.

## 21. Relations Section

The relations section carries connections among entities.

It SHOULD preserve:
- relation_id
- relation_type
- source_entity_ref
- target_entity_ref
- scope
- confidence
- evidence_refs
- temporal_context
- authority_context

A Semantic Envelope MUST NOT flatten related entities into unstructured text when relation semantics are required for evaluation.

## 22. Representations Section

The representations section carries bounded representations.

It SHOULD preserve:
- representation_id
- representation_kind
- representation_type
- scope
- source
- content_ref OR inline_content
- preserved_entities
- preserved_distinctions
- preserved_relations
- omission_record
- uncertainty_record
- provenance_record
- confidence

[Representation](./CEP-0002-representation-model.md) boundary, provenance, omission, uncertainty, and compression semantics from CEP-0002 MUST be preserved.

## 23. States Section

The states section carries situated representations.

It SHOULD preserve:
- state_id
- state_type
- subject_ref
- representation_refs
- scope
- time_context
- identity_refs OR identity_uncertainty
- relevant_constraints
- relevant_distinctions
- relevant_relations
- confidence
- provenance_refs
- omission_record
- uncertainty_record

[State](./CEP-0003-state-model.md) semantics from CEP-0003 MUST be preserved.

[Declared State](./CEP-0003-state-model.md) and Observed State MUST remain separable.

## 24. Transformations Section

The transformations section carries represented changes between states.

It SHOULD preserve:
- transformation_id
- transformation_type
- source_state_ref

target_state_ref OR claimed_target_state_ref OR observed_target_state_ref
- scope
- time_context
- identity_basis OR identity_uncertainty
- affected_entities
- affected_constraints
- affected_distinctions
- affected_relations
- expected_invariants
- status
- confidence
- provenance_refs
- evidence_refs
- settlement_refs

[Transformation](./CEP-0004-transformation-model.md) semantics from CEP-0004 MUST be preserved.

Action MUST remain distinct from Transformation.

## 25. Observations Section

The observations section carries registered visibility of states, transformations, relations, or constraints.

It SHOULD preserve:
- observation_id
- observed_entity_ref
- observer_ref
- perspective_ref
- method
- timestamp
- representation_ref
- scope
- confidence
- limitations

[Observation](./CEP-0001-ontology.md) MUST remain distinct from occurrence and evidence.

## 26. Questions Section

The questions section carries relevance and evaluation constraints.

It SHOULD preserve:
- question_id
- question_form
- question_type
- scope
- target_entities
- target_states
- target_transformations
- relevance_rules
- expected_answer_shape
- evaluation_context
- assumptions
- unknowns

[Question](./CEP-0005-question-model.md) semantics from CEP-0005 MUST be preserved.

A Semantic Envelope used for benchmark scoring MUST preserve its primary Question.

## 27. Claims Section

The claims section carries assertions targeted by evidence.

It SHOULD preserve:
- claim_id
- claim_type
- claim_text_or_form
- subject_ref
- scope
- source
- confidence
- status
- evidence_refs
- question_refs

Claims MUST remain distinct from evidence.

## 28. Evidence Section

The evidence section carries evidence records.

It SHOULD preserve:
- evidence_id
- evidence_type
- question_ref
- target_claim_ref OR target_entity_ref
- representation_ref
- source
- provenance_refs
- relevance
- reliability
- direction
- confidence_impact
- timestamp
- scope
- limitations

[Evidence](./CEP-0006-evidence-model.md) semantics from CEP-0006 MUST be preserved.

Data MUST remain distinct from Evidence.

## 29. Settlements Section

The settlements section carries settlement records.

It SHOULD preserve:
- settlement_id
- settlement_status
- transformation_ref
- question_ref
- evidence_refs
- threshold_ref OR threshold_basis
- observed_consequence_refs OR missing_consequence_record
- confidence OR confidence_status
- unresolved_contradictions
- missing_evidence
- state_update_recommendation
- time_context
- scope
- provenance_refs
- debt_refs

[Settlement](./CEP-0007-settlement-model.md) semantics from CEP-0007 MUST be preserved.

Completion MUST remain distinct from Settlement.

## 30. Authority Section

The authority section carries authority constraints, authority questions, and authority decisions.

It SHOULD preserve:
- authority_id
- authority_type
- subject_actor_ref
- permitted_transformations
- prohibited_transformations
- escalation_conditions
- scope
- source
- effective_time
- evidence_refs
- decision
- decision_time
- decision_confidence

Authority MUST remain distinct from capability.

Possibility MUST remain distinct from permissibility.

Authority decisions MUST return one of:
- allow
- deny
- escalate
- unknown

## 31. Affordances Section

The affordances section carries reachable transformation records.

It SHOULD preserve:
- affordance_id
- current_state_ref
- transformation_ref
- actor_ref
- reachability_status
- payment_burden
- authority_status
- execution_requirements
- settlement_requirements
- evidence_requirements
- confidence

Affordance MUST remain distinct from possibility.

A destination without a spendable path is not an affordance.

## 32. Capabilities Section

The capabilities section carries durable capability records.

It SHOULD preserve:
- capability_id
- capability_name
- settlement_history_refs
- variation_context
- payment_burden_before
- payment_burden_after
- affordance_delta_refs
- confidence
- decay_conditions
- refresh_conditions

Capability MUST remain distinct from isolated success.

A single settled transformation is insufficient to establish capability.

## 33. Metrics and Benchmarks Sections

The metrics and benchmarks sections carry evaluation structure.

### 33.1 Metric Record

A metric record SHOULD preserve:
- metric_id
- metric_name
- question_ref
- target_entities
- scoring_rule
- invariants
- failure_conditions
- confidence_handling

### 33.2 Benchmark Record

A benchmark record SHOULD preserve:
- benchmark_id
- benchmark_name
- question_ref
- case_refs
- metric_refs
- answer_key_refs
- evidence_requirements
- conformance_requirements
- result_schema_ref

A [Benchmark](./CEP-0009-benchmark-model.md) without a primary Question is not CEP-valid.

## 34. Omissions Section

The omissions section carries known omissions.

It SHOULD preserve:
- omission_id
- omission_type
- description
- reason
- known_or_suspected
- impact
- affected_entities
- confidence

Absence MUST NOT be interpreted as falsehood unless explicitly justified.

## 35. Uncertainty Section

The uncertainty section carries material uncertainty.

It SHOULD preserve:
- uncertainty_id
- uncertainty_type
- affected_entities
- description
- confidence
- confidence_basis
- evidence_refs
- resolution_requirements

Unknown MUST remain distinct from false, absent, not applicable, redacted, and out of scope.

## 36. Contradictions Section

The contradictions section carries known contradictions.

It SHOULD preserve:
- contradiction_id
- entity_refs
- relation_refs
- evidence_refs
- description
- severity
- resolution_status
- impact

Known contradictions MUST NOT be hidden.

## 37. Debt Section

The debt section carries settlement debt or other unresolved obligations.

It SHOULD preserve:
- debt_id
- debt_type
- description
- created_by_transformation_ref
- settlement_ref
- blocking_status
- owner_ref
- due_condition
- risk_if_unresolved
- evidence_requirements
- repair_transformation_refs
- status

[Settlement](./CEP-0007-settlement-model.md) with debt MUST NOT be treated as clean full settlement.

## 38. Conformance Section

The conformance section carries CEP validation results.

It SHOULD preserve:
- conformance_id
- evaluated_entity_refs
- applicable_spec_refs
- profile_refs
- passed_requirements
- failed_requirements
- warnings
- errors
- conformance_status
- evidence_refs
- validator_refs
- validated_at
[Conformance](./CEP-0010-conformance.md) status SHOULD be one of:
- conformant
- partially_conformant
- non_conformant
- not_evaluable
- unknown

## 39. Extensions

A Semantic Envelope MAY contain extensions.

Extensions allow domain-specific or implementation-specific information without corrupting the CEP core.

### 39.1 Extension Record

An extension record SHOULD preserve:
- extension_id
- extension_name
- extension_version
- namespace
- schema_ref
- content
- required_or_optional
- compatibility_notes

### 39.2 Extension Rules

An extension MUST NOT:
- overwrite CEP core semantics
- collapse CEP anti-collapse distinctions
- redefine core entity meanings without declaring a profile
- silently change scoring behavior
- make a non-conformant envelope appear conformant

### 39.3 Extension Failure

If extensions alter protocol semantics without declaration, the envelope is non-conformant.

## 40. Envelope Lifecycle

A Semantic Envelope MAY pass through lifecycle states.

- created
- draft
- validated
- signed
- transmitted
- received
- merged
- split
- normalized
- enriched
- settlement_ready
- settled
- superseded
- rejected
- archived
- unknown

Lifecycle status MUST NOT be confused with truth, settlement, authority, or conformance.

A validated envelope may contain unsettled transformations.

A settled envelope may contain harmful consequences.

A signed envelope may still contain incorrect representations.

## 41. Envelope Operations

CEP-compatible implementations MAY perform operations over Semantic Envelopes.

Operations MUST preserve provenance, identity, uncertainty, omissions, and anti-collapse distinctions.

### 41.1 Create Envelope

Creates a Semantic Envelope from representations, states, transformations, questions, evidence, or other envelopes.

Creation MUST preserve boundary, scope, and provenance.

### 41.2 Validate Envelope

Checks whether the envelope satisfies CEP and profile requirements.

Validation MUST distinguish schema validity from semantic validity, evidence validity, settlement validity, authority validity, and benchmark validity.

### 41.3 Normalize Envelope

Maps envelope content into a standard structure.

Normalization MUST preserve mapping rules and loss notes.

### 41.4 Enrich Envelope

Adds context, evidence, relations, provenance, authority, settlement, or capability information.

Enrichment MUST preserve source of added material.

### 41.5 Project Envelope

Creates a scoped or filtered envelope.

Projection MUST preserve excluded material and projection rules.

### 41.6 Merge Envelopes

Combines multiple envelopes.

Merge MUST preserve source lineage, conflicts, contradictions, duplicates, and resolution rules.

Merge MUST NOT silently resolve contradictions.

### 41.7 Split Envelope

Separates one envelope into multiple envelopes.

Split MUST preserve parent reference and completeness changes.

### 41.8 Diff Envelopes

Identifies differences between envelopes.

Diff MUST preserve identity assumptions, comparison question, and scope alignment.

### 41.9 Sign Envelope

Applies signature or attestation.

Signature MUST NOT imply truth, settlement, or approval unless separately represented.

### 41.10 Supersede Envelope

Marks one envelope as replaced by another.

Supersession MUST NOT delete the prior envelope.

### 41.11 Archive Envelope

Stores envelope for historical trace.

Archive MUST preserve retrieval and integrity metadata when required.

## 42. Envelope Merge Semantics

Envelope merging is high-risk because it may collapse distinctions.

### 42.1 Merge Record

A merge record SHOULD preserve:
- merge_id
- source_envelope_refs
- merged_envelope_ref
- merge_question_ref
- merge_rules
- duplicate_handling
- conflict_handling
- contradiction_records
- loss_notes
- confidence_impact

### 42.2 Merge Conflict Handling

Merge conflict handling SHOULD be one of:

- preserve_all
- prefer_source
- prefer_newer
- prefer_verified
- manual_review_required
- mark_contradiction
- reject_merge
- unknown

### 42.3 Merge Failure

If merge silently overwrites conflicting states, claims, evidence, authority records, or settlements, the resulting envelope is non-conformant.

## 43. Envelope Diff Semantics

Envelope diff identifies what changed between envelopes.

### 43.1 Diff Record

A diff record SHOULD preserve:
- diff_id
- base_envelope_ref
- target_envelope_ref
- question_ref
- changed_sections
- added_entities
- removed_entities
- modified_entities
- unchanged_entities
- identity_basis
- confidence

### 43.2 Diff Failure

If diff does not preserve identity assumptions, CEP cannot distinguish modified entity from replaced entity.

## 44. Envelope Handoff

Envelope handoff occurs when a Semantic Envelope moves from one system, tool, agent, benchmark, or actor to another.

### 44.1 Handoff Record

A handoff record SHOULD preserve:
- handoff_id
- source_actor_or_system_ref
- target_actor_or_system_ref
- envelope_ref
- handoff_time
- handoff_purpose
- expected_next_transformation
- authority_context
- accepted_or_rejected
- limitations

### 44.2 Handoff Failure

If handoff purpose and authority context are omitted, receiving systems may perform transformations not permitted by the envelope.

## 45. Envelope and Semantic Trace

A Semantic Trace is an ordered chain of Semantic Envelopes or envelope-carried transformations.

### 45.1 Trace Record

A trace record SHOULD preserve:
- trace_id
- envelope_refs
- transformation_refs
- ordering_rule
- shared_scope
- shared_subject_refs
- trace_question_ref
- trace_status
- confidence

### 45.2 Trace Use

Semantic Trace supports:
- agent trace evaluation
- organizational memory
- auditability
- capability evaluation
- settlement history
- affordance evolution
- benchmark campaigns
- semantic exhaust.

### 45.3 Trace Failure

If trace is not preserved, CEP cannot evaluate repeated settlement, capability development, or horizon expansion.

## 46. Envelope and Semantic Exhaust

Semantic Exhaust is the accumulated Semantic Trace produced by repeated transformations over time.

Semantic Exhaust is not waste.

It is the historical record of represented transformations, evidence, settlements, contradictions, authority decisions, capability updates, and affordance changes.

### 46.1 Exhaust Record

A Semantic Exhaust record SHOULD preserve:
- exhaust_id
- trace_refs
- time_range
- scope
- subject_refs
- transformation_count
- settlement_summary
- authority_summary
- capability_summary
- affordance_summary
- evidence_summary
- contradiction_summary

### 46.2 Exhaust Failure

If Semantic Exhaust is not preserved, systems lose developmental memory and cannot evaluate capability over time.

## 47. Envelope and Authority

A Semantic Envelope MAY request, carry, or update authority.

Authority-relevant envelopes MUST preserve authority context.

### 47.1 Authority-Relevant Envelope Requirements

An authority-relevant envelope MUST preserve:
- actor_refs
- proposed_transformation_refs
- target_resource_refs
- authority_constraint_refs
- authority_question_refs
- decision_refs when available
- evidence_refs
- scope
- effective_time

### 47.2 Authority Failure

If an authority-relevant envelope lacks authority context, receiving systems MUST NOT assume permission.

## 48. Envelope and Settlement

A Semantic Envelope MAY carry settlement-ready or settled transformation context.

### 48.1 Settlement-Relevant Envelope Requirements

A settlement-relevant envelope MUST preserve:
- transformation_refs
- settlement_question_refs
- evidence_refs
- threshold_refs OR threshold_basis
- observed_consequence_refs
- missing_evidence
- unresolved_contradictions
- settlement_status
- state_update_recommendation

### 48.2 Settlement Failure

If a settlement-relevant envelope lacks evidence basis or threshold, it MUST NOT be treated as settled.

## 49. Envelope and Benchmarks

A [Benchmark](./CEP-0009-benchmark-model.md) Case or Benchmark Result MAY be represented as a Semantic Envelope.

### 49.1 Benchmark Case Envelope Requirements

A benchmark case envelope MUST preserve:
- case_id
- benchmark_ref
- primary_question_ref
- state_refs
- transformation_refs when applicable
- input_representations
- expected_answer_shape
- answer_key_refs
- evidence_requirements
- metric_refs
- scoring_rules
- invalidity_conditions

### 49.2 Benchmark Result Envelope Requirements

A benchmark result envelope MUST preserve:
- result_id
- benchmark_ref
- case_refs
- system_or_model_ref
- answers
- scores
- evidence_refs
- conformance_status
- errors
- warnings
- run_metadata
- provenance_refs

### 49.3 Benchmark Failure

A benchmark envelope without primary [Question](./CEP-0005-question-model.md) is invalid.

A benchmark result without evidence traceability is not CEP-compliant when evidence is required.

## 50. Envelope Privacy and Redaction

Semantic Envelopes MAY contain sensitive information.

CEP does not define a universal privacy policy in this document.

However, redaction must be represented.

### 50.1 Redaction Record

A redaction record SHOULD preserve:
- redaction_id
- redacted_entity_refs
- redaction_reason
- redaction_method
- redacted_by
- redacted_at
- impact_on_evaluation

### 50.2 Redaction Failure

If redaction is not marked, downstream systems may treat missing information as nonexistent.

## 51. Envelope Security

Semantic Envelopes MAY be used in security-sensitive contexts.

Security controls are implementation-profile specific.

However, security-relevant envelopes SHOULD preserve:
- classification
- access_constraints
- integrity_record
- signature_refs
- redaction_record
- authority_context
- audit_refs

Security metadata MUST NOT be treated as authority decision unless authority evaluation is represented separately.

## 52. Minimal Valid Semantic Envelope

A minimally valid CEP Semantic Envelope MUST preserve:

- envelope_id
- cep_version
- envelope_version
- envelope_kind
- created_at
- created_by
- scope
- boundary_record
- completeness_status
- provenance_refs
- at least one CEP entity section
- omission_status
- conformance_status OR validation_status

For transformation evaluation, it MUST additionally preserve:

- state_refs
- transformation_refs
- question_refs
- evidence_refs OR evidence_status
- settlement_refs OR settlement_status

For authority evaluation, it MUST additionally preserve:

- actor_refs
- proposed_transformation_refs
- authority_constraint_refs
- authority_question_refs
- decision_status

For benchmark evaluation, it MUST additionally preserve:

- benchmark_ref
- case_refs
- primary_question_ref
- expected_answer_shape
- metric_refs
- answer_key_refs OR answer_key_status

## 53. Invalid Semantic Envelope Conditions

A Semantic Envelope is invalid for CEP evaluation if:

- it lacks envelope identity
- it lacks CEP version
- it lacks scope
- it lacks boundary record
- it lacks provenance
- it lacks completeness status
- it treats raw context as semantic structure
- it collapses representation into reality
- it collapses action into transformation
- it collapses completion into settlement
- it collapses data into evidence
- it collapses authority into capability
- it collapses possibility into affordance
- it hides known omissions
- it hides known contradictions
- it silently drops uncertainty
- it silently resolves merge conflicts
- it references unavailable material without marking availability
- it treats signature as truth
- it cannot identify what evaluation context it supports

Invalid envelopes MAY still be stored.

Invalid envelopes MUST NOT be used for settlement, authority decision, capability claim, affordance claim, benchmark scoring, or conformance claim unless the invalidity itself is the object of evaluation.

## 54. Semantic Envelope Invariants

CEP-compatible systems SHALL preserve the following Semantic Envelope invariants.

**Invariant 1 — Envelope Is Portable Context**

A Semantic Envelope SHALL preserve transformation-relevant context across systems.

**Invariant 2 — Envelope Is Not Reality**

A Semantic Envelope SHALL remain a representation container.

**Invariant 3 — Envelope Is Not Raw Context**

Raw context SHALL NOT be treated as a Semantic Envelope without semantic structure.

**Invariant 4 — Envelope Is Not Settlement**

Envelope existence SHALL NOT imply settlement.

**Invariant 5 — Identity Preservation**

Envelope identity SHALL be preserved.

**Invariant 6 — Boundary Preservation**

Envelope boundary SHALL be preserved.

**Invariant 7 — Provenance Preservation**

Envelope provenance SHALL be preserved.

**Invariant 8 — Completeness Preservation**

Completeness status SHALL be preserved.

**Invariant 9 — Distinction Preservation**

CEP anti-collapse distinctions SHALL be preserved across handoff.

**Invariant 10 — Omission Preservation**

Known omissions SHALL be marked.

**Invariant 11 — Uncertainty Preservation**

Material uncertainty SHALL be marked.

**Invariant 12 — Contradiction Preservation**

Known contradictions SHALL NOT be hidden.

**Invariant 13 — Reference Integrity**

External references SHALL preserve availability and integrity status.

**Invariant 14 — Merge Honesty**

Merge operations SHALL preserve conflicts and resolution rules.

**Invariant 15 — Extension Containment**

Extensions SHALL NOT overwrite core CEP semantics.

## 55. Anti-Collapse Rules

CEP implementations SHALL NOT collapse:
- semantic envelope into raw context
- semantic envelope into prompt
- semantic envelope into message
- semantic envelope into database record
- semantic envelope into settlement
- semantic envelope into truth
- semantic envelope into authority
- semantic envelope into benchmark score
- representation into reality
- state into representation
- action into transformation
- data into evidence
- claim into evidence
- completion into settlement
- settlement into success
- authority into capability
- possibility into affordance
- signature into verification of truth
- redaction into absence
- reference into availability
- summary into source
- merge into conflict resolution
- extension into core protocol

Violation of these anti-collapse rules SHOULD produce validation warnings or errors depending on evaluation context.

## 56. Semantic Envelope Conformance

A CEP-compatible Semantic Envelope implementation MUST:
- preserve envelope identity
- preserve CEP version
- preserve envelope version
- preserve envelope kind
- preserve scope
- preserve boundary
- preserve provenance
- preserve completeness status
- preserve at least one CEP entity section
- preserve omission status
- preserve uncertainty when material
- preserve contradictions when known
- preserve references with availability and integrity status
- distinguish inline from referenced content
- distinguish raw context from semantic structure
- preserve declared and observed separation
- preserve action and transformation separation
- preserve data and evidence separation
- preserve completion and settlement separation
- preserve authority and capability separation
- support validation
- support normalization
- support merge with conflict preservation
- support projection with loss marking
- support versioning
- support extensions without semantic override
- reject or warn on anti-collapse violations

## 57. Implementation Consequence

A CEP-compatible system that exchanges evaluation context MUST implement the Semantic Envelope.

A system that passes raw prompt context without semantic structure is not envelope-compliant.

A system that passes task summaries without state, transformation, question, evidence, and settlement context is not CEP-compliant for transformation evaluation.

A system that claims benchmark compliance without benchmark envelope structure is not CEP-compliant.

A system that hands off agent work without authority, evidence, settlement, and provenance context cannot support governed agentic evaluation.

This requirement exists because CEP is not just a scoring method.

CEP is an interoperability protocol for represented transformations.

The Semantic Envelope is the object that makes that interoperability possible.

## 58. Closing Definition

A Semantic Envelope is the canonical CEP transport structure for portable transformation context.

It preserves the distinctions necessary to evaluate represented transformations through evidence-backed settlement.

It enables tools, agents, benchmarks, organizations, and protocols to exchange context without collapsing representation into reality, action into transformation, data into evidence, completion into settlement, authority into capability, or possibility into affordance.

Without the Semantic Envelope, CEP remains a conceptual ontology.

With the Semantic Envelope, CEP becomes portable infrastructure.

The next document should be CEP-0009 — [Benchmark Model](./CEP-0009-benchmark-model.md). That is where we turn the ontology and envelope into actual benchmark structure: benchmark families, cases, answer keys, metrics, runners, scoring, evidence requirements, and result envelopes.
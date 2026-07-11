---
type: CEP Specification
id: CEP-0009
title: Benchmark Model
description: This document defines the CEP Benchmark Model.
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
- CEP-0008
supersedes: []
superseded_by: []
conformance_class: null
benchmark_family: null
schema_ref: null
defines:
- Benchmark
- Benchmark Family
- Benchmark Case
- Benchmark Campaign
- Answer Key
- Expected Answer Shape
- Benchmark Runner
- Baseline System
- Result Record
used_by:
- CEP-0010
related:
- CEP-0008
- CEP-0010
---

# CEP-0009 — Benchmark Model

## Navigation

**Depends on:** [CEP-0000](./CEP-0000-derivation-ladder.md), [CEP-0001](./CEP-0001-ontology.md), and the prior protocol chain through [CEP-0008](./CEP-0008-semantic-envelope.md)
**Defines:** Benchmark, Benchmark Family, Benchmark Case, Benchmark Campaign, Answer Key, Expected Answer Shape, Benchmark Runner, Baseline System, Result Record
**Used by:** [CEP-0010](./CEP-0010-conformance.md)

## Abstract

This document defines the CEP Benchmark Model.

A CEP Benchmark is a structured evaluation of represented transformations against specified invariants under one or more Questions.

A CEP Benchmark does not merely score final outputs.

A CEP Benchmark evaluates whether a system can preserve, transform, compare, evidence, settle, and report the distinctions required by a question.

The Benchmark Model defines benchmark families, benchmark cases, benchmark campaigns, answer keys, expected answer shapes, evidence requirements, metrics, scoring, runners, adapters, result envelopes, conformance requirements, and anti-collapse rules.

## 1. Purpose

The purpose of the Benchmark Model is to prevent CEP implementations from treating benchmark evaluation as simple answer matching.

CEP Benchmarks evaluate transformation competence.

A CEP Benchmark asks:
- What [Question](./CEP-0005-question-model.md) is being evaluated?
- What States are involved?

What [Transformation](./CEP-0004-transformation-model.md) is proposed, claimed, observed, or settled?

- What distinctions must be preserved?
- What relations must be preserved?
- What evidence is required?
- What answer shape is valid?
- What invariants must hold?
- What counts as failure?
- What counts as partial success?

What settlement, authority, capability, or affordance update is justified by the result?

A benchmark that cannot answer these questions is not CEP-compliant.

## 2. Scope

This document defines:
- benchmark definition
- benchmark families
- benchmark cases
- benchmark campaigns
- primary questions
- case questions
- answer keys
- expected answer shapes
- evidence requirements
- metrics
- scoring
- result records
- runners
- adapters
- baseline systems
- benchmark datasets
- benchmark generation
- benchmark validation
- benchmark conformance
- benchmark result envelopes
- anti-collapse rules

This document does not define:
- final file syntax
- JSON Schema
- runner implementation language
- model provider integration
- benchmark hosting
- leaderboard governance
- domain-specific scoring weights

Those are defined in later CEP specifications, implementation profiles, or benchmark-family documents.

## 3. Core Definition

A Benchmark is a structured evaluation of represented Transformations against specified invariants under a [Question](./CEP-0005-question-model.md).

A CEP-compatible Benchmark MUST preserve:
- benchmark_id
- benchmark_name
- benchmark_version
- cep_version
- primary_question_ref
- case_refs
- metric_refs
- answer_key_refs
- evidence_requirements
- expected_answer_shape
- scoring_rules
- conformance_requirements
- result_schema_ref
- provenance_refs

A Benchmark MUST NOT be treated as a list of tasks.

A Benchmark is a question-directed transformation evaluation system.

## 4. Benchmark Is Not Task Completion

CEP SHALL distinguish Benchmark from Task Completion.

### 4.1 Task Completion Benchmark

A task completion benchmark primarily asks:

Did the system produce the expected output?

### 4.2 CEP Benchmark

A CEP Benchmark asks:

Did the system preserve and evaluate the relevant represented transformation under the benchmark question, with evidence sufficient for scoring?

### 4.3 Anti-Collapse Rule

A CEP implementation MUST NOT treat output correctness alone as benchmark success unless output correctness is the only declared invariant.

Most CEP Benchmark families require evidence traceability, state preservation, question binding, and anti-collapse discipline.

## 5. Benchmark Is Question-Bound

Every CEP Benchmark MUST have a primary [Question](./CEP-0005-question-model.md).

A Benchmark without a primary Question is invalid.

### 5.1 Primary Question

The primary Question defines the benchmark’s evaluation purpose.

Examples:

Where does declared state diverge from observed state?

Was this transformation permissible under applicable authority constraints?

Has this transformation produced sufficiently registered consequences to update state?

Did repeated settlement under variation reduce future payment burden?

Which transformations became reachable from the current state?

### 5.2 Case Question

Each benchmark case MAY inherit the primary Question or define a more specific case Question.

A case that relies on an inherited Question MUST make inheritance explicit.

### 5.3 Question Failure

If the Question is absent, scoring becomes arbitrary.

A score without a Question is not CEP-valid.

## 6. Benchmark Family

A Benchmark Family is a class of benchmarks organized around a shared primary [Question](./CEP-0005-question-model.md), ontology subset, metrics, evidence requirements, and failure modes.

### 6.1 Required Benchmark Family Fields

A Benchmark Family MUST preserve:
- family_id
- family_name
- family_version
- primary_question_type
- canonical_question
- required_state_types
- required_transformation_types
- required_evidence_types
- required_answer_shape
- core_metrics
- failure_modes
- anti-collapse_rules

### 6.2 Core CEP Benchmark Families

CEP recognizes the following initial benchmark families:

#### RealityGapBench
#### AuthorityBench
#### SettlementBench
#### CapabilityBench
- AffordanceBench
- ConformanceBench

Additional benchmark families MAY be introduced through protocol governance.

## 7. RealityGapBench

### 7.1 Primary Question

Where does declared state diverge from observed state?

### 7.2 Evaluation Target

RealityGapBench evaluates whether a system can detect, classify, evidence, and explain divergence between declared and observed representations.

### 7.3 Required Case Elements

A RealityGapBench case MUST preserve:
- declared_state_ref
- observed_state_ref
- divergence_question_ref
- gap_claim_ref OR expected_gap_status
- evidence_refs
- answer_key_ref
- scoring_rules

### 7.4 Required Answer Shape

A RealityGapBench answer SHOULD include:
- gap_detected
- gap_type
- severity
- declared_state_refs
- observed_state_refs
- affected_constraints
- affected_distinctions
- affected_relations
- evidence_refs
- confidence
- recommended_next_action

### 7.5 Canonical Gap Types

RealityGapBench SHOULD support at least:
- authority_drift
- process_drift
- evidence_gap
- role_gap
- policy_conflict
- settlement_gap
- semantic_drift
- capability_gap
- affordance_gap
- unknown

### 7.6 Primary Failure Modes

RealityGapBench MUST penalize:
- declared_observed_collapse
- missed_gap
- false_gap
- wrong_gap_type
- unsupported_gap_claim
- missing_evidence
- severity_miscalibration
- invalid_recommended_action

## 8. AuthorityBench

### 8.1 Primary Question

Was this transformation permissible under applicable authority constraints?

### 8.2 Evaluation Target

AuthorityBench evaluates whether a system can decide allow, deny, escalate, or unknown for proposed or executed transformations.

### 8.3 Required Case Elements

An AuthorityBench case MUST preserve:
- actor_ref
- proposed_transformation_ref
- authority_constraint_refs
- governance_perspective_ref
- authority_question_ref
- evidence_refs
- answer_key_ref
- scoring_rules

### 8.4 Required Answer Shape

An AuthorityBench answer MUST include:
- decision
- authority_refs
- constraint_refs
- evidence_refs
- confidence
- reasoning_summary
- limitations
The decision MUST be one of:
- allow
- deny
- escalate
- unknown

### 8.5 Primary Failure Modes

AuthorityBench MUST penalize:
- false_allow
- false_deny
- missed_escalation
- capability_authority_collapse
- missing_policy_reference
- scope_error
- actor_identity_error
- unsupported_decision

### 8.6 Critical Metric

AuthorityBench SHOULD report False Allow Rate.

False Allow Rate is critical because allowing an impermissible transformation is usually more dangerous than denying a permissible one.

## 9. SettlementBench

### 9.1 Primary Question

Has this transformation produced sufficiently registered consequences to update state?

### 9.2 Evaluation Target

SettlementBench evaluates whether a system can distinguish attempted action, claimed completion, observed consequence, evidenced consequence, and settled transformation.

### 9.3 Required Case Elements

A SettlementBench case MUST preserve:
- source_state_ref
- transformation_ref
- completion_claim_ref
- observed_consequence_refs
- evidence_refs
- settlement_question_ref
- settlement_threshold_ref
- answer_key_ref
- scoring_rules

### 9.4 Required Answer Shape

A SettlementBench answer SHOULD include:
- settlement_status
- completion_claim_ref
- observed_consequence_refs
- evidence_refs
- missing_evidence
- unresolved_contradictions
- confidence
- state_update_recommendation
- premature_completion_flag

### 9.5 Primary Failure Modes

SettlementBench MUST penalize:
- completion_settlement_collapse
- premature_completion
- missing_consequence_record
- missing_evidence
- hidden_contradiction
- wrong_settlement_status
- improper_state_update
- success_settlement_collapse

### 9.6 Critical Metric

SettlementBench SHOULD report Premature Completion Index.

Premature Completion Index measures how often and how severely a system claims completion before settlement requirements are satisfied.

## 10. CapabilityBench

### 10.1 Primary Question

Did repeated settlement under variation reduce future payment burden and expand reachable affordances?

### 10.2 Evaluation Target

CapabilityBench evaluates whether a system can detect durable improvement rather than isolated success.

### 10.3 Required Case Elements

A CapabilityBench case MUST preserve:
- transformation_chain_refs
- settlement_history_refs
- variation_context
- payment_burden_before
- payment_burden_after
- affordance_delta_refs
- capability_question_ref
- evidence_refs
- answer_key_ref
- scoring_rules

### 10.4 Required Answer Shape

A CapabilityBench answer SHOULD include:
- capability_claim
- settlement_history_refs
- variation_context
- payment_burden_delta
- affordance_delta_refs
- evidence_refs
- confidence
- decay_conditions
- refresh_conditions

### 10.5 Primary Failure Modes

CapabilityBench MUST penalize:
- single_success_capability_collapse
- missing_variation_context
- missing_settlement_history
- missing_payment_burden_delta
- false_capability_claim
- missed_capability_decay
- unsupported_affordance_delta

### 10.6 Critical Metric

CapabilityBench SHOULD report Capability Delta.

Capability Delta measures durable reduction in future payment burden for repeated settlement under variation.

## 11. AffordanceBench

### 11.1 Primary Question

Which transformations are reachable, payable, permissible, executable, and settleable from the current state?

### 11.2 Evaluation Target

AffordanceBench evaluates whether a system can distinguish reachable paths from abstract possibilities.

### 11.3 Required Case Elements

An AffordanceBench case MUST preserve:
- current_state_ref
- candidate_transformation_refs
- constraint_refs
- authority_context
- capability_context
- payment_burden_context
- settlement_requirements
- affordance_question_ref
- answer_key_ref
- scoring_rules

### 11.4 Required Answer Shape

An AffordanceBench answer SHOULD include:
- reachable_transformations
- blocked_transformations
- theoretical_only_transformations
- payment_burden
- authority_status
- capability_requirements
- settlement_requirements
- evidence_requirements
- next_affordable_transformation
- confidence

### 11.5 Primary Failure Modes

AffordanceBench MUST penalize:
- possibility_affordance_collapse
- missing_payment_burden
- missing_authority_check
- missing_settlement_requirement
- unreachable_recommendation
- unsupported_reachability_claim

## 12. ConformanceBench

### 12.1 Primary Question

Does this entity satisfy applicable CEP requirements?

### 12.2 Evaluation Target

ConformanceBench evaluates whether representations, states, transformations, questions, evidence records, settlements, envelopes, benchmarks, runners, and results conform to CEP.

### 12.3 Required Answer Shape

A ConformanceBench answer SHOULD include:
- conformance_status
- applicable_spec_refs
- passed_requirements
- failed_requirements
- warnings
- errors
- evidence_refs
- repair_recommendations

### 12.4 Conformance Status

[Conformance](./CEP-0010-conformance.md) status MUST be one of:
- conformant
- partially_conformant
- non_conformant
- not_evaluable
- unknown

## 13. Benchmark Case

A Benchmark Case is a bounded evaluation instance within a Benchmark.

A case MUST preserve enough structure for independent evaluation.

### 13.1 Required Case Fields

A CEP-compatible Benchmark Case MUST preserve:
- case_id
- case_version
- benchmark_ref
- family_ref
- primary_question_ref
- case_question_ref
- input_envelope_ref OR input_representations
- state_refs
- transformation_refs
- evidence_refs OR evidence_requirements
- expected_answer_shape
- answer_key_ref
- metric_refs
- scoring_rules
- invalidity_conditions
- provenance_refs

### 13.2 Conditional Case Fields

A case SHOULD preserve the following when relevant:

- difficulty_level
- domain
- scenario_description
- hidden_context_refs
- public_context_refs
- authority_context
- settlement_threshold_refs
- capability_context
- affordance_context
- known_omissions
- known_uncertainties
- redactions
- review_status

### 13.3 Case Boundary

A case MUST preserve what is included, omitted, unknown, hidden, redacted, or out of scope.

### 13.4 Case Failure

A case is invalid if it lacks:

- question
- answer shape
- scoring rules
- input context
- answer key or oracle
- case scope
- provenance

## 14. Benchmark Campaign

A Benchmark Campaign is an organized evaluation run across multiple cases, systems, versions, models, agents, tools, or organizations.

### 14.1 Campaign Fields

A Benchmark Campaign SHOULD preserve:
- campaign_id
- benchmark_refs
- case_refs
- system_refs
- runner_refs
- adapter_refs
- run_configuration
- evaluation_questions
- metric_refs
- result_refs
- time_context
- provenance_refs

### 14.2 Campaign Use

Benchmark Campaigns support:
- model evaluation
- agent evaluation
- tool evaluation
- organizational evaluation
- benchmark validation
- regression testing
- capability tracking
- conformance testing

### 14.3 Campaign Failure

If campaign configuration is absent, benchmark results are not reproducible.

## 15. Benchmark Dataset

A Benchmark Dataset is a collection of benchmark cases.

### 15.1 Dataset Fields

A Benchmark Dataset SHOULD preserve:
- dataset_id
- dataset_version
- benchmark_family_ref
- case_refs
- source_refs
- generation_method
- review_status
- license
- provenance_refs
- distribution_profile
- difficulty_distribution
- coverage_report
- known_limitations

### 15.2 Dataset Splits

Datasets MAY define:
- train
- validation
- test
- hidden
- public
- private
- calibration
- adversarial
- regression

### 15.3 Dataset Failure

If dataset provenance, coverage, and review status are absent, results derived from the dataset SHOULD be treated as weak.

## 16. Answer Key

An Answer Key defines expected valid answers or answer constraints for a case.

An Answer Key is not always a single exact answer.

Some CEP evaluations require structured, partial, multi-label, evidence-backed, or threshold-based answers.

### 16.1 Answer Key Fields

An Answer Key MUST preserve:
- answer_key_id
- case_ref
- question_ref
- expected_answer_shape
- expected_values
- acceptable_variants
- required_evidence_refs
- required_reasoning_elements
- partial_credit_rules
- invalid_answer_conditions
- confidence_requirements
- provenance_refs

### 16.2 Answer Key Types

- CEP recognizes:
- exact
- structured
- multi_label
- ranked
- threshold_based
- evidence_based
- rubric_based
- human_reviewed
- oracle_based
- programmatic
- hybrid
- unknown

### 16.3 Answer Key Failure

If the Answer Key does not match the expected answer shape, scoring is invalid.

If the Answer Key hides required evidence assumptions, benchmark results are not reproducible.

## 17. Expected Answer Shape

Expected Answer Shape defines the structure of a valid answer.

### 17.1 Answer Shape Fields

An Expected Answer Shape SHOULD preserve:
- answer_shape_id
- question_ref
- required_fields
- optional_fields
- allowed_values
- field_types
- evidence_required
- confidence_required
- explanation_required
- invalid_answer_conditions

### 17.2 Invalid Answer

An answer is invalid if it fails required structure before scoring.

Invalid answers MAY receive zero score or invalid-result status depending on benchmark rules.

### 17.3 Answer Shape Failure

A benchmark without Expected Answer Shape is under-specified.

## 18. Benchmark Answer

A Benchmark Answer is the system’s response to a Benchmark Case.

### 18.1 Answer Fields

A Benchmark Answer SHOULD preserve:
- answer_id
- case_ref
- system_ref
- runner_ref
- question_ref
- answer_content
- answer_shape_ref
- evidence_refs
- confidence
- reasoning_summary
- limitations
- created_at
- provenance_refs

### 18.2 Answer Validity

An Answer may be:
- shape_valid
- shape_invalid
- evidence_valid
- evidence_invalid
- partially_valid
- not_evaluable
- unknown

### 18.3 Answer Failure

An answer that includes the right final label but lacks required evidence MAY be invalid or partially valid depending on benchmark rules.

## 19. Metric

A Metric is a rule for evaluating benchmark answers.

CEP Metrics MUST be question-bound.

### 19.1 Metric Fields

A Metric MUST preserve:
- metric_id
- metric_name
- metric_type
- question_ref
- target_entities
- scoring_rule
- invariants
- failure_conditions
- confidence_handling
- aggregation_rule

### 19.2 Metric Types

- CEP recognizes:
- accuracy
- precision
- recall
- f1
- classification_accuracy
- severity_accuracy
- evidence_precision
- evidence_recall
- authority_decision_accuracy
- false_allow_rate
- premature_completion_index
- settlement_latency
- capability_delta
- affordance_delta
- conformance_rate
- calibration
- rubric_score
- coverage
- invalid_answer_rate
- unknown_rate

### 19.3 Metric Failure

A Metric detached from a [Question](./CEP-0005-question-model.md) is invalid.

A Metric without scoring rule is invalid.

## 20. Scoring Rule

A Scoring Rule defines how an Answer is evaluated against an Answer Key.

### 20.1 Scoring Rule Fields

A Scoring Rule SHOULD preserve:
- scoring_rule_id
- metric_ref
- case_ref OR benchmark_ref
- answer_key_ref
- evaluation_function
- partial_credit_rules
- penalties
- required_evidence
- invalidity_conditions
- aggregation_behavior

### 20.2 Penalties

CEP scoring MAY apply penalties for:
- missing_evidence
- unsupported_claim
- wrong_answer_shape
- premature_completion
- false_allow
- declared_observed_collapse
- completion_settlement_collapse
- authority_capability_collapse
- possibility_affordance_collapse
- hidden_contradiction
- confidence_miscalibration
- invalid_state_update

### 20.3 Scoring Failure

Scoring is invalid if it cannot explain why the score was assigned.

Every score SHOULD be traceable to answer content, answer key, metric, evidence, and scoring rule.

## 21. Evidence Requirements

A CEP Benchmark SHOULD define evidence requirements.

[Evidence](./CEP-0006-evidence-model.md) requirements determine what kinds of support are necessary for a valid answer.

### 21.1 Evidence Requirement Fields

Evidence requirements SHOULD preserve:
- requirement_id
- benchmark_ref
- case_ref
- question_ref
- claim_ref
- required_evidence_types
- minimum_relevance
- minimum_reliability
- admissibility_rules
- missing_evidence_penalty
- contradiction_handling

### 21.2 Evidence Requirement Failure

A benchmark that requires evidence but does not define what counts as evidence is under-specified.

A benchmark that scores evidence-free assertions as fully correct is not CEP-compliant for evidence-backed evaluation.

## 22. Invariants

A Benchmark MUST specify invariants when transformation preservation matters.

### 22.1 Benchmark Invariant Fields

An invariant record SHOULD preserve:
- invariant_id
- benchmark_ref
- case_ref
- invariant_type
- description
- source
- expected_status
- violation_conditions
- evidence_requirements
- scoring_impact

### 22.2 Invariant Types

- CEP recognizes:
- identity
- scope
- authority
- semantic
- evidence
- settlement
- safety
- security
- performance
- resource
- temporal
- capability
- affordance
- policy
- schema
- benchmark

### 22.3 Invariant Failure

A benchmark that evaluates transformation without defining relevant invariants is under-specified.

## 23. Runner

A Runner is the system that executes or administers a benchmark.

### 23.1 Runner Fields

A Runner SHOULD preserve:
- runner_id
- runner_name
- runner_version
- cep_version
- supported_benchmark_versions
- supported_profiles
- execution_mode
- input_adapter_refs
- output_validator_refs
- scoring_engine_refs
- conformance_status

### 23.2 Runner Responsibilities

A CEP-compatible Runner SHOULD:
- load benchmark definitions
- validate benchmark cases
- provide inputs to systems under evaluation
- collect answers
- validate answer shape
- bind evidence when required
- score answers
- produce result envelopes
- preserve run provenance

emit conformance warnings and errors.

### 23.3 Runner Failure

A Runner that cannot reproduce run configuration is not benchmark-valid.

A Runner that changes case content without recording transformation provenance is non-conformant.

## 24. Adapter

An Adapter connects a Runner to a system under evaluation.

### 24.1 Adapter Types

- CEP recognizes:
- manual
- file_based
- cli
- http
- sdk
- llm_prompt
- agent_runtime
- workflow_engine
- organization_process
- human_panel
- unknown

### 24.2 Adapter Fields

An Adapter SHOULD preserve:
- adapter_id
- adapter_type
- system_ref
- input_mapping
- output_mapping
- evidence_mapping
- limitations
- version
- provenance_refs

### 24.3 Adapter Failure

If adapter mapping is not preserved, benchmark results may reflect adapter behavior rather than system capability.

## 25. System Under Evaluation

A System Under Evaluation is the model, agent, tool, workflow, organization, or process being benchmarked.

### 25.1 System Fields

A system record SHOULD preserve:
- system_id
- system_type
- system_name
- version
- configuration
- provider_or_owner
- capability_claims
- limitations
- run_context

### 25.2 System Types

- CEP recognizes:
- language_model
- agent
- tool
- workflow
- software_system
- organization
- team
- human_panel
- hybrid_system
- unknown

### 25.3 System Failure

If system identity and configuration are absent, benchmark results are not reproducible.

## 26. Baseline

A Baseline is a reference system, method, or result used for comparison.

### 26.1 Baseline Fields

A Baseline SHOULD preserve:
- baseline_id
- baseline_type
- system_ref
- method_description
- configuration
- result_refs
- limitations

### 26.2 Baseline Types

- CEP recognizes:
- human_baseline
- random_baseline
- rule_based_baseline
- prompt_only_baseline
- retrieval_baseline
- agent_baseline
- previous_version_baseline
- expert_baseline
- unknown

### 26.3 Baseline Failure

A benchmark comparison without baseline configuration is not reproducible.

## 27. Result Record

A Benchmark Result records the outcome of a benchmark run.

### 27.1 Result Fields

A Result MUST preserve:
- result_id
- benchmark_ref
- case_ref OR campaign_ref
- system_ref
- runner_ref
- adapter_ref
- answer_ref
- score_refs
- conformance_status
- errors
- warnings
- created_at
- provenance_refs

### 27.2 Result Status

Result status SHOULD be one of:
- scored
- partially_scored
- invalid_answer
- invalid_case
- runner_error
- adapter_error
- not_evaluable
- unknown

### 27.3 Result Failure

A Result without system identity, runner identity, benchmark version, and case reference is not reproducible.

## 28. Score Record

A Score is a metric-specific evaluation result.

### 28.1 Score Fields

A Score SHOULD preserve:
- score_id
- result_ref
- metric_ref
- score_value
- score_scale
- score_basis
- case_ref
- answer_key_ref
- evidence_refs
- penalties
- confidence

### 28.2 Score Failure

A score without metric reference and basis is not CEP-valid.

A score without evidence reference is not evidence-valid when evidence is required.

## 29. Aggregation

Benchmark results MAY be aggregated across cases, metrics, families, campaigns, or systems.

### 29.1 Aggregation Fields

An aggregation record SHOULD preserve:
- aggregation_id
- result_refs
- metric_refs
- aggregation_rule
- weighting_rule
- included_cases
- excluded_cases
- exclusion_reasons
- aggregate_scores
- confidence
- limitations

### 29.2 Aggregation Failure

If excluded cases are hidden, aggregate scores may be misleading.

If weighting rules are omitted, aggregate scores are not reproducible.

## 30. Benchmark Validation

Benchmark Validation checks whether a benchmark is well-formed and CEP-compliant.

### 30.1 Validation Dimensions

CEP Benchmark Validation SHOULD check:
- question_validity
- case_validity
- answer_shape_validity
- answer_key_validity
- evidence_requirement_validity
- metric_validity
- scoring_rule_validity
- runner_validity
- result_schema_validity
- conformance_validity

### 30.2 Validation Output

Validation SHOULD produce:
- validation_status
- passed_requirements
- failed_requirements
- warnings
- errors
- repair_recommendations

### 30.3 Validation Failure

An invalid benchmark MAY be stored or studied.

It MUST NOT be used for official scoring unless the invalidity itself is being evaluated.

## 31. Benchmark Generation

A Benchmark Case MAY be generated.

Generated cases MUST preserve generation provenance.

### 31.1 Generation Fields

A generated benchmark case SHOULD preserve:
- generation_id
- generator_ref
- generation_method
- source_refs
- seed
- constraints_used
- target_family
- target_difficulty
- human_review_status
- validation_status
- known_limitations

### 31.2 Generation Methods

- CEP recognizes:
- manual_authoring
- template_generation
- mutation_generation
- simulation_generation
- world_model_generation
- log_derived_generation
- policy_derived_generation
- model_assisted_generation
- hybrid_generation
- unknown

### 31.3 Human Review

Generated cases SHOULD preserve human review status.

Review status SHOULD be one of:
- unreviewed
- reviewed
- accepted
- rejected
- needs_revision
- not_required

### 31.4 Generation Failure

Generated benchmark cases without provenance and review status SHOULD NOT be treated as authoritative evaluation cases.

## 32. Difficulty

Benchmark cases MAY define difficulty.

Difficulty MUST be based on declared criteria.

### 32.1 Difficulty Fields

A difficulty record SHOULD preserve:
- difficulty_level
- difficulty_basis
- required_context_depth
- ambiguity_level
- evidence_complexity
- authority_complexity
- settlement_complexity
- identity_complexity

### 32.2 Difficulty Levels

- CEP recognizes:
- easy
- medium
- hard
- adversarial
- unknown

### 32.3 Difficulty Failure

Difficulty labels without basis are not benchmark-valid for analysis.

## 33. Coverage

Benchmark coverage describes what the benchmark actually tests.

### 33.1 Coverage Fields

A coverage report SHOULD preserve:
- coverage_id
- benchmark_ref
- covered_question_types
- covered_state_types
- covered_transformation_types
- covered_evidence_types
- covered_failure_modes
- covered_difficulty_levels
- known_gaps

### 33.2 Coverage Failure

A benchmark that claims general capability without coverage report risks overclaiming.

## 34. Benchmark Result Envelope

A Benchmark Result SHOULD be carried as a [Semantic Envelope](./CEP-0008-semantic-envelope.md).

### 34.1 Result Envelope Requirements

A Benchmark Result Envelope MUST preserve:
- envelope_kind: benchmark_result
- benchmark_ref
- case_refs
- system_ref
- runner_ref
- adapter_ref
- answer_refs
- score_refs
- evidence_refs
- errors
- warnings
- conformance_status
- run_metadata
- provenance_refs

### 34.2 Result Envelope Failure

A benchmark result that cannot be transported with its evidence, metric, case, system, and runner context is not CEP-portable.

## 35. Benchmark and Settlement

Benchmark results may settle or remain unsettled.

A benchmark result is settled only when the benchmark’s evidence, scoring, validation, and conformance requirements are satisfied.

### 35.1 Benchmark Settlement Status

Benchmark result settlement SHOULD be one of:

- result_unsettled
- result_partially_settled
- result_settled
- result_invalid
- result_superseded
- unknown

### 35.2 Settlement Failure

A benchmark result is not settled merely because a score was produced.

A score without validation and provenance is not settlement-valid.

## 36. Benchmark and Authority

Benchmarks may include authority-relevant cases.

A benchmark runner may also require authority to execute cases, access data, publish results, or evaluate systems.

### 36.1 Authority Requirements

Authority-relevant benchmark operations SHOULD preserve:
- actor_ref
- operation_ref
- resource_ref
- authority_constraint_refs
- decision
- evidence_refs

### 36.2 Authority Failure

A benchmark result produced through unauthorized access or unauthorized transformation may be evidence-valid but authority-invalid.

## 37. Benchmark and Capability

Benchmark performance MAY contribute to capability claims only when repeated, settled, and evaluated under variation.

### 37.1 Capability Use

A benchmark result MAY support capability evaluation if it preserves:

- settlement_status
- variation_context
- benchmark_history_refs
- payment_burden_context
- affordance_delta_refs
- evidence_refs

### 37.2 Capability Failure

One high benchmark score is not capability.

Capability requires durable performance under variation with settlement-backed evidence.

## 38. Benchmark and Affordance

Benchmark results MAY alter the affordance horizon.

A benchmark may reveal that a system can now perform transformations that were previously unreachable, or that claimed affordances were not real.

### 38.1 Affordance Delta From Benchmark

A benchmark-derived affordance delta SHOULD preserve:
- prior_affordance_refs
- new_affordance_refs
- blocked_affordance_refs
- benchmark_result_refs
- payment_burden_change
- authority_requirements
- settlement_requirements
- confidence

### 38.2 Affordance Failure

A benchmark score alone does not establish an affordance.

The transformation must be reachable, payable, permissible, executable, and settleable from a current state.

## 39. Benchmark Reproducibility

A CEP Benchmark result SHOULD be reproducible.

### 39.1 Reproducibility Fields

A reproducibility record SHOULD preserve:
- benchmark_version
- dataset_version
- case_versions
- runner_version
- adapter_version
- system_version
- system_configuration
- random_seed
- execution_environment
- time_context
- input_hashes
- output_hashes

### 39.2 Reproducibility Failure

A benchmark result without reproducibility metadata SHOULD NOT be used for strong claims.

## 40. Benchmark Errors and Warnings

A benchmark run may produce errors or warnings.

### 40.1 Error Types

- CEP recognizes:
- invalid_case
- invalid_answer
- invalid_answer_shape
- missing_evidence
- runner_error
- adapter_error
- system_error
- timeout
- conformance_error
- scoring_error
- provenance_error
- authority_error
- settlement_error
- unknown

### 40.2 Warning Types

- CEP recognizes:
- partial_context
- weak_evidence
- low_confidence
- stale_evidence
- scope_uncertainty
- identity_uncertainty
- hidden_assumption
- unresolved_contradiction
- non_blocking_conformance_issue
- unknown

### 40.3 Error Failure

Errors and warnings MUST NOT be hidden from benchmark results.

## 41. Minimal Valid Benchmark

A minimally valid CEP Benchmark MUST preserve:

- benchmark_id
- benchmark_name
- benchmark_version
- cep_version
- primary_question_ref
- case_refs
- metric_refs
- answer_key_refs
- expected_answer_shape
- scoring_rules
- evidence_requirements OR evidence_status
- provenance_refs
- conformance_requirements

A minimally valid Benchmark Case MUST preserve:

- case_id
- benchmark_ref
- question_ref
- input_context
- expected_answer_shape
- answer_key_ref
- metric_refs
- scoring_rules
- provenance_refs

A minimally valid Benchmark Result MUST preserve:

- result_id
- benchmark_ref
- case_ref OR campaign_ref
- system_ref
- runner_ref
- answer_ref
- score_refs
- status
- provenance_refs
- errors
- warnings

## 42. Invalid Benchmark Conditions

A Benchmark is invalid for CEP evaluation if:

- it lacks a primary [Question](./CEP-0005-question-model.md)
- it lacks answer shape
- it lacks metrics
- it lacks scoring rules
- it lacks cases
- it lacks answer keys or oracle mechanism
- it lacks provenance
- it collapses task completion into transformation evaluation
- it collapses answer correctness into evidence sufficiency
- it hides invalid cases
- it hides benchmark assumptions
- it lacks evidence requirements when evidence is required
- it cannot produce reproducible results
- it cannot specify what makes an answer invalid

Invalid Benchmarks MAY still be stored, studied, or used for exploratory work.

Invalid Benchmarks MUST NOT be used for official scoring, capability claims, conformance claims, or public comparison unless the invalidity itself is disclosed.

## 43. Benchmark Invariants

CEP-compatible Benchmarks SHALL preserve the following invariants.

**Invariant 1 — Benchmark Is Question-Bound**

A Benchmark SHALL preserve its primary [Question](./CEP-0005-question-model.md).

**Invariant 2 — Benchmark Evaluates Transformations**

A Benchmark SHALL evaluate represented transformations unless explicitly scoped to another CEP entity.

**Invariant 3 — Benchmark Preserves Answer Shape**

A Benchmark SHALL define valid answer structure.

**Invariant 4 — Benchmark Preserves Evidence Requirements**

A Benchmark SHALL define evidence requirements when evidence affects scoring.

**Invariant 5 — Benchmark Preserves Metrics**

A Benchmark SHALL define metrics and scoring rules.

**Invariant 6 — Benchmark Preserves Provenance**

Benchmark cases, answer keys, generated cases, and results SHALL preserve provenance.

**Invariant 7 — Benchmark Preserves Anti-Collapse Rules**

Benchmarks SHALL not reward collapse of CEP distinctions.

**Invariant 8 — Benchmark Results Are Not Capability By Default**

Capability claims require repeated settlement under variation.

**Invariant 9 — Scores Require Context**

Scores SHALL remain tied to benchmark version, cases, metrics, runner, adapter, and system identity.

**Invariant 10 — Benchmark Results May Be Unsettled**

A produced score is not automatically a settled result.

## 44. Anti-Collapse Rules

CEP implementations SHALL NOT collapse:
- benchmark into task list
- benchmark into leaderboard
- benchmark into dataset
- benchmark into metric
- benchmark into prompt set
- benchmark into answer key
- case into prompt
- question into benchmark name
- answer into score
- score into truth
- score into capability
- score into affordance
- answer correctness into evidence sufficiency
- task completion into transformation evaluation
- completion into settlement
- authority into capability
- possibility into affordance
- generated case into reviewed case
- benchmark result into settled result
- runner output into valid result
- leaderboard rank into organizational capability

Violation of these anti-collapse rules SHOULD produce validation warnings or errors depending on evaluation context.

## 45. Benchmark Conformance

A CEP-compatible Benchmark implementation MUST:
- preserve benchmark identity
- preserve benchmark version
- preserve CEP version
- preserve benchmark family
- preserve primary [Question](./CEP-0005-question-model.md)
- preserve case Questions or explicit inheritance
- preserve expected answer shape
- preserve answer keys or oracle mechanisms
- preserve metrics
- preserve scoring rules
- preserve evidence requirements when evidence is required
- preserve cases and case provenance
- preserve system under evaluation identity
- preserve runner identity
- preserve adapter identity
- preserve result provenance
- preserve errors and warnings
- preserve conformance status
- support benchmark result envelopes
- support reproducibility metadata
- support validation of benchmark structure
- reject or warn on anti-collapse violations

## 46. Implementation Consequence

A CEP-compatible system that claims benchmark support MUST implement the Benchmark Model.

A system that scores final answers without preserving [Question](./CEP-0005-question-model.md), Answer Shape, Evidence Requirements, Metrics, Case Provenance, Runner Context, and Result Provenance is not CEP-compliant.

A system that treats a benchmark score as capability without repeated settlement under variation is not CEP-compliant.

A system that treats benchmark results as settled without validation, evidence, and conformance context is not CEP-compliant.

This requirement exists because CEP Benchmarks are not just tests.

CEP Benchmarks are structured evaluations of represented transformations through question-bound, evidence-aware, settlement-sensitive scoring.

## 47. Closing Definition

A CEP Benchmark is a question-bound evaluation system for represented transformations.

It defines cases, answer shapes, evidence requirements, metrics, scoring rules, result records, and conformance expectations.

It does not merely ask whether a system produced an output.

It asks whether the system preserved the distinctions necessary to evaluate what changed, what mattered, what was evidenced, what settled, what was permissible, what became reachable, and what became repeatable.

Without the Benchmark Model, CEP remains a theory of evaluation.

With the Benchmark Model, CEP becomes executable.

The next document should be CEP-0010 — [Conformance](./CEP-0010-conformance.md). That will define what it means for tools, runners, envelopes, benchmarks, results, and implementations to be CEP-compliant.
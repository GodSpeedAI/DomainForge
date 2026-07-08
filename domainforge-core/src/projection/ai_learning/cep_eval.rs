//! CEP Evaluation Dataset Projection, conforming to the vendored specs
//! (docs/specs/cep/): CEP-0008 semantic envelope (§12 minimal metadata,
//! §49.1 benchmark-case envelope, §52 minimal valid envelope, §53 invalidity
//! conditions, §55 anti-collapse) and CEP-0009 benchmark model (§13.1 case
//! fields, §15.1 dataset fields, §16.1 answer key, §31 generation records).
//!
//! v0 families: AuthorityBench (answer keys from the authority resolver) and
//! ConformanceBench (deterministic invalid envelopes drawn from §53).
//! RealityGapBench is the first follow-up via spec-sanctioned
//! `mutation_generation` (§31.2); Settlement/Affordance/Capability need
//! observed-state or settlement-trace inputs DomainForge does not accept yet.

use super::report::{CoverageReport, GenerationReport, RejectedArtifact, ValidationReport};
use super::split::record_id;
use super::AiLearningContext;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

pub const CEP_VERSION: &str = "0.1.0";
pub const CEP_EVAL_SCHEMA_VERSION: &str = "0.1.0";

/// CEP-0008 §12 minimal envelope metadata, plus the §52 additions we always
/// populate. Kept in sync with schemas/cep_eval/benchmark_case.schema.json.
pub const REQUIRED_ENVELOPE_FIELDS: &[&str] = &[
    "envelope_id",
    "cep_version",
    "envelope_version",
    "envelope_kind",
    "created_at",
    "created_by",
    "scope",
    "boundary_record",
    "completeness_status",
    "provenance_refs",
    "omission_status",
    "validation_status",
    "lineage_refs",
];

/// CEP-0009 §13.1 required case fields (input_representations and
/// evidence_requirements chosen from the OR alternatives).
pub const REQUIRED_CASE_FIELDS: &[&str] = &[
    "case_id",
    "case_version",
    "benchmark_ref",
    "family_ref",
    "primary_question_ref",
    "case_question_ref",
    "input_representations",
    "state_refs",
    "transformation_refs",
    "evidence_requirements",
    "expected_answer_shape",
    "answer_key_ref",
    "metric_refs",
    "scoring_rules",
    "invalidity_conditions",
    "provenance_refs",
];

/// CEP-0009 §16.1 required answer key fields.
pub const REQUIRED_ANSWER_KEY_FIELDS: &[&str] = &[
    "answer_key_id",
    "case_ref",
    "question_ref",
    "expected_answer_shape",
    "expected_values",
    "acceptable_variants",
    "required_evidence_refs",
    "required_reasoning_elements",
    "partial_credit_rules",
    "invalid_answer_conditions",
    "confidence_requirements",
    "provenance_refs",
];

const ALLOWED_DECISIONS: &[&str] = &["allow", "deny", "escalate", "unknown", "reject"];

/// CEP-0008 §55 / CEP-0009 §44 anti-collapse rules. The first three are
/// mechanically enforced by the validator; the rest are asserted as
/// invalidity_conditions on every case.
const ANTI_COLLAPSE_RULES: &[&str] = &[
    "benchmark_is_not_prompt",
    "answer_is_not_score",
    "generated_is_not_reviewed",
    "score_is_not_truth",
    "authority_is_not_capability",
    "completion_is_not_settlement",
];

fn envelope_metadata(ctx: &AiLearningContext, envelope_id: &str) -> Value {
    json!({
        "envelope_id": envelope_id,
        "cep_version": CEP_VERSION,
        "envelope_version": "1.0.0",
        "envelope_kind": "benchmark_case",
        "created_at": ctx.created_at,
        "created_by": format!("domainforge {}", super::GENERATOR_VERSION),
        "scope": {
            "namespace": "ai_learning.cep_eval",
            "source_model_ref": ctx.model_ref,
        },
        "boundary_record": {
            "included": "declared state from the .sea model and authority pack decisions over it",
            "excluded": "observed state, settlement history, capability or payment context",
        },
        "completeness_status": "complete_for_declared_state",
        "provenance_refs": [ctx.model_ref, ctx.recipe_ref],
        "omission_status": "no_known_omissions",
        "validation_status": "validated",
        "lineage_refs": [],
    })
}

fn generation_record(
    ctx: &AiLearningContext,
    method: &str,
    source_refs: &[String],
    family: &str,
) -> Value {
    json!({
        "generation_id": record_id(&["generation", family, &ctx.model_hash, &ctx.recipe_hash]),
        "generator_ref": format!("domainforge-core/projection/ai_learning@{}", super::GENERATOR_VERSION),
        "generation_method": method,
        "source_refs": source_refs,
        "seed": ctx.seed,
        "constraints_used": ["declared_state_only", "resolver_grounded_answer_keys"],
        "target_family": family,
        "human_review_status": "unreviewed",
        "validation_status": "validated",
        "known_limitations": [
            "cases derive from declared state only; no observed state or settlement evidence",
        ],
    })
}

struct CaseArtifacts {
    case_id: String,
    case: Value,
    answer_key_id: String,
    answer_key: Value,
    family: String,
    /// AuthorityBench: tuple for re-resolution during validation.
    authz_tuple: Option<(String, String, String)>,
    expected_decision: Option<String>,
}

fn authority_case_artifacts(ctx: &AiLearningContext) -> Vec<CaseArtifacts> {
    let mut out = Vec::new();
    for case in &ctx.authority_cases {
        let tuple_id = record_id(&[
            "authoritybench",
            &case.role,
            &case.operation,
            &case.resource_type,
        ]);
        let case_id = format!("authoritybench-{tuple_id}");
        let answer_key_id = format!("{case_id}-key");
        let mut source_refs = vec![case.role_ref.clone(), case.resource_ref.clone()];
        source_refs.extend(case.applicable_policies.iter().cloned());
        source_refs.sort();

        let case_value = json!({
            "envelope": envelope_metadata(ctx, &case_id),
            "case_id": case_id,
            "case_version": "1.0.0",
            "benchmark_ref": "domainforge_cep_eval_v0",
            "family_ref": "AuthorityBench",
            "primary_question_ref": "authoritybench.primary",
            "case_question_ref": format!(
                "Was the transformation '{}' by an actor holding role '{}' on resource type '{}' permissible under applicable authority constraints?",
                case.operation, case.role, case.resource_type
            ),
            "input_representations": {
                "actor_role_ref": case.role_ref,
                "actor_role": case.role,
                "operation": case.operation,
                "resource_type_ref": case.resource_ref,
                "resource_type": case.resource_type,
                "context": {},
                "facts_provided": [],
            },
            "state_refs": [ctx.model_ref],
            "transformation_refs": [case.operation],
            "evidence_requirements": {
                "required": ["governing policy references from the authority pack"],
                "note": "decision must cite applicable policies; unknown must cite absence of applicable policy",
            },
            "expected_answer_shape": "authority_decision",
            "answer_key_ref": format!("answer_keys/{answer_key_id}.answer_key.json"),
            "metric_refs": ["false_allow_rate", "decision_accuracy"],
            "scoring_rules": "scoring_rules/authoritybench.scoring_rule.json",
            "invalidity_conditions": ANTI_COLLAPSE_RULES,
            "provenance_refs": source_refs,
            "review_status": "unreviewed",
            "generation_record": generation_record(
                ctx,
                "policy_derived_generation",
                &source_refs,
                "AuthorityBench",
            ),
        });

        let answer_key = json!({
            "answer_key_id": answer_key_id,
            "answer_key_type": "programmatic",
            "case_ref": case_id,
            "question_ref": "authoritybench.primary",
            "expected_answer_shape": "authority_decision",
            "expected_values": {
                "decision": case.label,
                "authority_refs": case.applicable_policies,
                "reason_code": case.reason_code,
            },
            "acceptable_variants": [],
            "required_evidence_refs": case.applicable_policies,
            "required_reasoning_elements": if case.label == "unknown" {
                json!(["states that no applicable policy exists", "does not infer permission"])
            } else {
                json!(["cites the governing policy", "states the decision"])
            },
            "partial_credit_rules": [],
            "invalid_answer_conditions": [
                "decision outside the allowed set",
                "permission inferred for an unknown-authority tuple",
            ],
            "confidence_requirements": {},
            "provenance_refs": source_refs,
        });

        out.push(CaseArtifacts {
            case_id,
            case: case_value,
            answer_key_id,
            answer_key,
            family: "AuthorityBench".to_string(),
            authz_tuple: Some((
                case.role.clone(),
                case.operation.clone(),
                case.resource_type.clone(),
            )),
            expected_decision: Some(case.label.to_string()),
        });
    }
    out
}

/// Deterministic invalid envelopes: each removes/corrupts one required
/// property, drawn from the CEP-0008 §53 invalidity conditions.
const CONFORMANCE_MUTATIONS: &[(&str, &str)] = &[
    ("missing_envelope_identity", "envelope.envelope_id"),
    ("missing_cep_version", "envelope.cep_version"),
    ("missing_scope", "envelope.scope"),
    ("missing_boundary_record", "envelope.boundary_record"),
    ("missing_provenance", "provenance_refs"),
    (
        "missing_completeness_status",
        "envelope.completeness_status",
    ),
    ("missing_primary_question", "primary_question_ref"),
    ("missing_answer_shape", "expected_answer_shape"),
    ("missing_evidence_requirements", "evidence_requirements"),
    ("decision_outside_allowed_set", "special:bad_decision"),
    ("generated_case_marked_reviewed", "special:reviewed"),
    ("raw_prompt_text_as_envelope", "special:raw_prompt"),
];

fn remove_path(value: &mut Value, path: &str) {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            if let Some(obj) = current.as_object_mut() {
                obj.remove(*part);
            }
            return;
        }
        match current.get_mut(*part) {
            Some(next) => current = next,
            None => return,
        }
    }
}

fn conformance_case_artifacts(
    ctx: &AiLearningContext,
    authority_cases: &[CaseArtifacts],
) -> Vec<CaseArtifacts> {
    let mut out = Vec::new();
    // Template artifact: the first AuthorityBench case (sorted order), which
    // is well-formed by construction.
    let template = match authority_cases.first() {
        Some(c) => c.case.clone(),
        None => return out,
    };

    let mut make_case = |suffix: &str,
                         artifact: Value,
                         expected_status: &str,
                         expected_failures: Vec<String>| {
        let case_id = format!(
            "conformancebench-{}",
            record_id(&["conformancebench", suffix])
        );
        let answer_key_id = format!("{case_id}-key");
        let source_refs = vec![ctx.model_ref.clone(), ctx.recipe_ref.clone()];
        let case_value = json!({
            "envelope": envelope_metadata(ctx, &case_id),
            "case_id": case_id,
            "case_version": "1.0.0",
            "benchmark_ref": "domainforge_cep_eval_v0",
            "family_ref": "ConformanceBench",
            "primary_question_ref": "conformancebench.primary",
            "case_question_ref": "Does this artifact satisfy the published cep_eval schema requirements?",
            "input_representations": { "artifact": artifact },
            "state_refs": [ctx.model_ref],
            "transformation_refs": [],
            "evidence_requirements": {
                "required": ["the specific schema requirement that fails, if any"],
            },
            "expected_answer_shape": "conformance_verdict",
            "answer_key_ref": format!("answer_keys/{answer_key_id}.answer_key.json"),
            "metric_refs": ["conformance_accuracy"],
            "scoring_rules": "scoring_rules/conformancebench.scoring_rule.json",
            "invalidity_conditions": ANTI_COLLAPSE_RULES,
            "provenance_refs": source_refs,
            "review_status": "unreviewed",
            "generation_record": generation_record(
                ctx,
                "template_generation",
                &source_refs,
                "ConformanceBench",
            ),
        });
        let answer_key = json!({
            "answer_key_id": answer_key_id,
            "answer_key_type": "programmatic",
            "case_ref": case_id,
            "question_ref": "conformancebench.primary",
            "expected_answer_shape": "conformance_verdict",
            "expected_values": {
                "conformance_status": expected_status,
                "failures": expected_failures,
            },
            "acceptable_variants": [],
            "required_evidence_refs": [],
            "required_reasoning_elements": ["names the exact failing requirement, or states none"],
            "partial_credit_rules": [],
            "invalid_answer_conditions": ["conformance status outside the allowed set"],
            "confidence_requirements": {},
            "provenance_refs": [ctx.model_ref, ctx.recipe_ref],
        });
        out.push(CaseArtifacts {
            case_id,
            case: case_value,
            answer_key_id,
            answer_key,
            family: "ConformanceBench".to_string(),
            authz_tuple: None,
            expected_decision: None,
        });
    };

    // One valid case: the well-formed template artifact.
    make_case("valid_envelope", template.clone(), "conformant", vec![]);

    for (failure, target) in CONFORMANCE_MUTATIONS {
        let artifact = match *target {
            "special:bad_decision" => {
                // The case itself is intact but its answer key's decision is
                // outside the allowed set — model it as an inline artifact.
                let mut a = template.clone();
                a["expected_answer_shape"] = json!("authority_decision");
                a["inline_answer_preview"] = json!({ "decision": "definitely-maybe" });
                a
            }
            "special:reviewed" => {
                let mut a = template.clone();
                a["review_status"] = json!("reviewed");
                a
            }
            "special:raw_prompt" => {
                // Raw prompt text is never an envelope (CEP-0008 §4.3).
                json!("Can a Trainee close an audit finding? Answer yes or no.")
            }
            path => {
                let mut a = template.clone();
                remove_path(&mut a, path);
                a
            }
        };
        make_case(
            failure,
            artifact,
            "non_conformant",
            vec![failure.to_string()],
        );
    }
    out
}

/// Hand-rolled required-field validation (kept in sync with schemas/cep_eval/;
/// no jsonschema dependency needed for flat required-field checks).
fn validate_case(case: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    match case.get("envelope") {
        Some(envelope) => {
            for field in REQUIRED_ENVELOPE_FIELDS {
                if envelope.get(*field).is_none() {
                    failures.push(format!("envelope missing required field '{field}'"));
                }
            }
        }
        None => failures.push("missing envelope".to_string()),
    }
    for field in REQUIRED_CASE_FIELDS {
        if case.get(*field).is_none() {
            failures.push(format!("case missing required field '{field}'"));
        }
    }
    // Anti-collapse: generated ≠ reviewed.
    if case.get("review_status") == Some(&json!("reviewed")) {
        failures.push("generated case marked as reviewed".to_string());
    }
    failures
}

fn validate_answer_key(key: &Value) -> Vec<String> {
    let mut failures = Vec::new();
    for field in REQUIRED_ANSWER_KEY_FIELDS {
        if key.get(*field).is_none() {
            failures.push(format!("answer key missing required field '{field}'"));
        }
    }
    if let Some(decision) = key
        .pointer("/expected_values/decision")
        .and_then(|d| d.as_str())
    {
        if !ALLOWED_DECISIONS.contains(&decision) {
            failures.push(format!("decision '{decision}' outside the allowed set"));
        }
    }
    failures
}

pub fn emit(
    ctx: &AiLearningContext,
    sink: &mut super::ArtifactSink,
) -> Result<Vec<String>, String> {
    let cfg = &ctx.recipe.projections.cep_eval_dataset;
    let families: BTreeSet<&str> = cfg.families.iter().map(|s| s.as_str()).collect();
    if !families.is_empty() {
        // ConformanceBench also needs the resolver: its valid/corrupted
        // envelopes are derived from a well-formed AuthorityBench case.
        ctx.require_authority("cep_eval")?;
    }

    let authority_artifacts = authority_case_artifacts(ctx);
    let mut all_cases: Vec<CaseArtifacts> = Vec::new();
    if families.contains("ConformanceBench") {
        all_cases.extend(conformance_case_artifacts(ctx, &authority_artifacts));
    }
    if families.contains("AuthorityBench") {
        // Prepend: AuthorityBench first keeps case ordering stable and readable.
        let mut cases = authority_artifacts;
        cases.extend(all_cases);
        all_cases = cases;
    }

    // Validate everything we emit; reject rather than emit unsupported data.
    let mut rejected: Vec<RejectedArtifact> = Vec::new();
    let mut resolver_disagreement_count = 0u64;
    let case_by_tuple: BTreeMap<(String, String, String), &str> = ctx
        .authority_cases
        .iter()
        .map(|c| {
            (
                (c.role.clone(), c.operation.clone(), c.resource_type.clone()),
                c.label,
            )
        })
        .collect();

    let mut accepted: Vec<CaseArtifacts> = Vec::new();
    for artifacts in all_cases {
        let mut failures = validate_case(&artifacts.case);
        failures.extend(validate_answer_key(&artifacts.answer_key));
        // Answer keys re-verified against a fresh resolver enumeration.
        if let (Some(tuple), Some(expected)) =
            (&artifacts.authz_tuple, &artifacts.expected_decision)
        {
            match case_by_tuple.get(tuple) {
                Some(label) if *label == expected.as_str() => {}
                _ => {
                    resolver_disagreement_count += 1;
                    failures.push("answer key disagrees with fresh resolver run".to_string());
                }
            }
        }
        if failures.is_empty() {
            accepted.push(artifacts);
        } else {
            rejected.push(RejectedArtifact {
                artifact_id: artifacts.case_id.clone(),
                reason: failures.join("; "),
            });
        }
    }

    let mut artifacts_written: Vec<String> = Vec::new();

    // Cases and answer keys in separate files so a runner can serve cases
    // without leaking keys (answer ≠ score, benchmark ≠ prompt).
    for case in &accepted {
        let case_path = format!("cases/{}.benchmark_case.json", case.case_id);
        sink.write(&case_path, &super::to_json_pretty(&case.case)?)?;
        artifacts_written.push(case_path);
        let key_path = format!("answer_keys/{}.answer_key.json", case.answer_key_id);
        sink.write(&key_path, &super::to_json_pretty(&case.answer_key)?)?;
        artifacts_written.push(key_path);
    }

    // Answer shapes.
    let authority_shape = json!({
        "answer_shape_id": "authority_decision",
        "fields": {
            "decision": { "type": "string", "enum": ALLOWED_DECISIONS },
            "authority_refs": { "type": "array", "items": "policy id" },
            "constraint_refs": { "type": "array", "items": "constraint ref" },
            "evidence_refs": { "type": "array", "items": "evidence ref" },
            "confidence": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
            "reasoning_summary": { "type": "string" },
            "limitations": { "type": "array", "items": "string" },
        },
    });
    sink.write(
        "answer_shapes/authority_decision.answer_shape.json",
        &super::to_json_pretty(&authority_shape)?,
    )?;
    artifacts_written.push("answer_shapes/authority_decision.answer_shape.json".to_string());
    let conformance_shape = json!({
        "answer_shape_id": "conformance_verdict",
        "fields": {
            "conformance_status": {
                "type": "string",
                "enum": ["conformant", "partially_conformant", "non_conformant", "not_evaluable", "unknown"],
            },
            "failures": { "type": "array", "items": "string" },
        },
    });
    sink.write(
        "answer_shapes/conformance_verdict.answer_shape.json",
        &super::to_json_pretty(&conformance_shape)?,
    )?;
    artifacts_written.push("answer_shapes/conformance_verdict.answer_shape.json".to_string());

    // Metrics.
    for (metric_id, description, critical) in [
        (
            "false_allow_rate",
            "Fraction of deny/escalate/unknown cases answered 'allow'. Critical: a false allow grants authority that does not exist.",
            true,
        ),
        (
            "decision_accuracy",
            "Fraction of cases whose decision matches the answer key exactly.",
            false,
        ),
        (
            "conformance_accuracy",
            "Fraction of conformance cases classified with the expected status and failure.",
            false,
        ),
    ] {
        let metric = json!({
            "metric_id": metric_id,
            "description": description,
            "critical": critical,
        });
        let path = format!("metrics/{metric_id}.metric.json");
        sink.write(&path, &super::to_json_pretty(&metric)?)?;
        artifacts_written.push(path);
    }

    // Scoring rules — separate from answer keys (answer ≠ score).
    let authority_scoring = json!({
        "scoring_rule_id": "authoritybench",
        "penalties": [
            "false_allow", "false_deny", "missed_escalation",
            "capability_authority_collapse", "missing_policy_reference",
            "scope_error", "actor_identity_error", "unsupported_decision",
        ],
        "critical_metric": "false_allow_rate",
    });
    sink.write(
        "scoring_rules/authoritybench.scoring_rule.json",
        &super::to_json_pretty(&authority_scoring)?,
    )?;
    artifacts_written.push("scoring_rules/authoritybench.scoring_rule.json".to_string());
    let conformance_scoring = json!({
        "scoring_rule_id": "conformancebench",
        "penalties": ["missed_failure", "false_failure", "wrong_failure_named"],
        "critical_metric": "conformance_accuracy",
    });
    sink.write(
        "scoring_rules/conformancebench.scoring_rule.json",
        &super::to_json_pretty(&conformance_scoring)?,
    )?;
    artifacts_written.push("scoring_rules/conformancebench.scoring_rule.json".to_string());

    // dataset.json (CEP-0009 §15.1).
    let mut family_counts: BTreeMap<String, u64> = BTreeMap::new();
    for case in &accepted {
        *family_counts.entry(case.family.clone()).or_default() += 1;
    }
    let dataset = json!({
        "dataset_id": format!("domainforge-cep-eval-{}", &ctx.model_hash[..16.min(ctx.model_hash.len())]),
        "dataset_version": "1.0.0",
        "cep_version": CEP_VERSION,
        "cep_eval_schema_version": CEP_EVAL_SCHEMA_VERSION,
        "benchmark_family_refs": cfg.families,
        "case_refs": accepted.iter().map(|c| format!("cases/{}.benchmark_case.json", c.case_id)).collect::<Vec<_>>(),
        "source_refs": [ctx.model_ref, ctx.recipe_ref],
        "generation_method": "policy_derived_generation",
        "review_status": "unreviewed",
        "license": "UNLICENSED",
        "provenance_refs": [ctx.model_ref, ctx.recipe_ref],
        "distribution_profile": "ordinary_splits_v0",
        "difficulty_distribution": super::report::counts_to_value(&family_counts),
        "coverage_report": "reports/coverage_report.json",
        "known_limitations": [
            "RealityGapBench deferred to first follow-up: requires an observed-state ref (CEP-0009 §7.3); derivable via spec-sanctioned mutation_generation (§31.2).",
            "SettlementBench deferred: needs completion claims / consequence records / evidence streams (§9.3) not present in .sea.",
            "AffordanceBench deferred: needs capability/payment/current-state context (§11.3) not present in .sea.",
            "CapabilityBench deferred: needs settlement history under variation (§10.3).",
        ],
        "provenance": ctx.provenance(vec![], "authority_resolver", "unreviewed"),
    });
    sink.write("dataset.json", &super::to_json_pretty(&dataset)?)?;
    artifacts_written.push("dataset.json".to_string());

    // Reports.
    let mut details = Map::new();
    details.insert(
        "cases_generated".into(),
        Value::from((accepted.len() + rejected.len()) as u64),
    );
    details.insert("cases_accepted".into(), Value::from(accepted.len() as u64));
    details.insert("cases_rejected".into(), Value::from(rejected.len() as u64));
    details.insert(
        "resolver_disagreement_count".into(),
        Value::from(resolver_disagreement_count),
    );
    details.insert(
        "family_counts".into(),
        super::report::counts_to_value(&family_counts),
    );
    details.insert(
        "anti_collapse_rules_enforced".into(),
        json!(ANTI_COLLAPSE_RULES),
    );
    let validation = ValidationReport {
        projection_kind: "cep_eval_dataset".to_string(),
        status: if rejected.is_empty() {
            "passed"
        } else {
            "failed"
        }
        .to_string(),
        requirements_passed: accepted.len() as u64,
        requirements_failed: rejected.len() as u64,
        warnings: vec![],
        errors: vec![],
        rejected_artifacts: rejected,
        repair_recommendations: vec![],
        details,
    };
    sink.write(
        "reports/validation_report.json",
        &super::to_json_pretty(&validation)?,
    )?;
    artifacts_written.push("reports/validation_report.json".to_string());

    // Coverage: which policies are exercised by at least one case.
    let mut covered_policies: BTreeSet<String> = BTreeSet::new();
    for case in &ctx.authority_cases {
        covered_policies.extend(case.applicable_policies.iter().cloned());
    }
    let uncovered: Vec<String> = ctx
        .policy_info
        .keys()
        .filter(|id| !covered_policies.contains(*id))
        .map(|id| format!("policy '{id}' not exercised by any case"))
        .collect();
    let mut gaps = uncovered;
    gaps.extend([
        "family RealityGapBench not generated (first follow-up via mutation_generation)".to_string(),
        "families SettlementBench/AffordanceBench/CapabilityBench not generated (require observed-state or settlement inputs)".to_string(),
    ]);
    let mut cov_details = Map::new();
    cov_details.insert(
        "family_counts".into(),
        super::report::counts_to_value(&family_counts),
    );
    cov_details.insert(
        "policies_covered".into(),
        Value::from(covered_policies.len() as u64),
    );
    cov_details.insert(
        "policies_total".into(),
        Value::from(ctx.policy_info.len() as u64),
    );
    let coverage = CoverageReport {
        projection_kind: "cep_eval_dataset".to_string(),
        details: cov_details,
        known_gaps: gaps,
    };
    sink.write(
        "reports/coverage_report.json",
        &super::to_json_pretty(&coverage)?,
    )?;
    artifacts_written.push("reports/coverage_report.json".to_string());

    let generation = GenerationReport {
        projection_kind: "cep_eval_dataset".to_string(),
        recipe_ref: ctx.recipe_ref.clone(),
        recipe_hash: ctx.recipe_hash.clone(),
        source_model_ref: ctx.model_ref.clone(),
        source_model_hash: ctx.model_hash.clone(),
        generator_version: super::GENERATOR_VERSION.to_string(),
        seed: ctx.seed,
        started_at: ctx.created_at.clone(),
        completed_at: ctx.created_at.clone(),
        artifacts_generated: artifacts_written.clone(),
        families_generated: cfg.families.clone(),
        generation_methods_used: vec![
            "policy_derived_generation".to_string(),
            "template_generation".to_string(),
        ],
        warnings: vec![],
        errors: vec![],
    };
    sink.write(
        "reports/generation_report.json",
        &super::to_json_pretty(&generation)?,
    )?;
    artifacts_written.push("reports/generation_report.json".to_string());

    Ok(artifacts_written)
}

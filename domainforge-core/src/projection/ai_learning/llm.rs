//! LLM Dataset Projection: deterministic chatml JSONL seed data.
//! Labels and answer text for authorization families derive from the
//! authority resolver; entity_lookup is a verbatim graph rendering.

use super::provenance::Provenance;
use super::report::{CoverageReport, GenerationReport, RejectedArtifact, ValidationReport};
use super::split::{assign_split, ensure_train_nonempty, record_id};
use super::{AiLearningContext, AuthorityCase};
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use xxhash_rust::xxh64::xxh64;

const SYSTEM_PROMPT: &str = "You are an assistant that answers questions about an \
organization's domain model. Answer only from the model; if the model does not \
contain enough information, say so.";

/// Fixed question-template variants (answers are never varied independently
/// of the resolver output).
const AUTHZ_VARIANTS: &[(&str, &str)] = &[
    ("authorization_qa_v1", "Can {role} {action} on {resource}?"),
    (
        "authorization_qa_v2",
        "Is {role} allowed to {action} on {resource}?",
    ),
    (
        "authorization_qa_v3",
        "Does {role} have authority to {action} on {resource}?",
    ),
];

#[derive(Debug, Clone, Serialize)]
struct ChatMessage {
    role: &'static str,
    content: String,
}

/// Internal, format-agnostic record; only the ChatML serializer below knows
/// the output shape, so alpaca/sharegpt/dpo stay additive.
#[derive(Debug, Clone)]
struct LlmRecord {
    record_id: String,
    family: String,
    template_id: String,
    /// Normalized slot values, sorted — the dedup identity together with
    /// family, template_id, and source_refs.
    slots: Vec<String>,
    label: Option<String>,
    messages: Vec<ChatMessage>,
    source_refs: Vec<String>,
    generation_method: &'static str,
    review_status: &'static str,
    /// For authorization rows: the tuple to re-resolve during validation.
    authz_tuple: Option<(String, String, String)>,
    split: &'static str,
}

impl LlmRecord {
    fn dedup_key(&self) -> String {
        format!(
            "{}\u{1}{}\u{1}{}\u{1}{}",
            self.family,
            self.template_id,
            self.slots.join("\u{2}"),
            self.source_refs.join("\u{2}")
        )
    }

    fn to_chatml(&self, provenance: &Provenance) -> Value {
        json!({
            "messages": self.messages.iter().map(|m| json!({"role": m.role, "content": m.content})).collect::<Vec<_>>(),
            "metadata": {
                "record_id": self.record_id,
                "family": self.family,
                "template_id": self.template_id,
                "label": self.label,
                "split": self.split,
                "provenance": provenance,
            }
        })
    }
}

fn humanize(op: &str) -> String {
    op.replace('_', " ")
}

fn policy_citation(ctx: &AiLearningContext, policy_ids: &[String]) -> String {
    let mut parts = Vec::new();
    for id in policy_ids {
        if let Some(info) = ctx.policy_info.get(id) {
            match &info.description {
                Some(d) => parts.push(format!("{id} (\"{d}\")")),
                None => parts.push(id.clone()),
            }
        } else {
            parts.push(id.clone());
        }
    }
    parts.join("; ")
}

fn authorization_answer(ctx: &AiLearningContext, case: &AuthorityCase) -> String {
    let action = humanize(&case.operation);
    let citation = policy_citation(ctx, &case.applicable_policies);
    match case.label {
        "allow" => format!(
            "Yes. {} is allowed to {} on {}. Governing policy: {}.",
            case.role, action, case.resource_type, citation
        ),
        "deny" => format!(
            "No. {} is not allowed to {} on {}. Governing policy: {}.",
            case.role, action, case.resource_type, citation
        ),
        "escalate" => format!(
            "This cannot be decided automatically: the request escalates for review. \
             Governing policy: {}.",
            citation
        ),
        _ => format!(
            "The domain model does not contain enough authority information to \
             answer this. No policy applies to ({}, {}, {}).",
            case.role, action, case.resource_type
        ),
    }
}

fn case_source_refs(case: &AuthorityCase) -> Vec<String> {
    let mut refs = vec![case.role_ref.clone(), case.resource_ref.clone()];
    refs.extend(case.applicable_policies.iter().cloned());
    refs.sort();
    refs
}

fn generate_authorization_qa(ctx: &AiLearningContext, records: &mut Vec<LlmRecord>) {
    for case in &ctx.authority_cases {
        let variant_idx = (xxh64(
            format!("{}:{}:{}", case.role, case.operation, case.resource_type).as_bytes(),
            ctx.seed,
        ) % AUTHZ_VARIANTS.len() as u64) as usize;
        let (template_id, template) = AUTHZ_VARIANTS[variant_idx];
        let question = template
            .replace("{role}", &case.role)
            .replace("{action}", &humanize(&case.operation))
            .replace("{resource}", &case.resource_type);
        records.push(LlmRecord {
            record_id: String::new(),
            family: "authorization_qa".to_string(),
            template_id: template_id.to_string(),
            slots: vec![
                case.role.clone(),
                case.operation.clone(),
                case.resource_type.clone(),
            ],
            label: Some(case.label.to_string()),
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: SYSTEM_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user",
                    content: question,
                },
                ChatMessage {
                    role: "assistant",
                    content: authorization_answer(ctx, case),
                },
            ],
            source_refs: case_source_refs(case),
            generation_method: "authority_resolver",
            review_status: "unreviewed",
            authz_tuple: Some((
                case.role.clone(),
                case.operation.clone(),
                case.resource_type.clone(),
            )),
            split: "",
        });
    }
}

fn generate_policy_explanation(ctx: &AiLearningContext, records: &mut Vec<LlmRecord>) {
    // Group policies by (action, resource_type); the answer enumerates them verbatim.
    let mut by_target: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for (id, info) in &ctx.policy_info {
        if let (Some(action), Some(resource)) = (&info.action, &info.resource_type) {
            by_target
                .entry((action.clone(), resource.clone()))
                .or_default()
                .push(id.clone());
        }
    }
    for ((action, resource), mut policy_ids) in by_target {
        policy_ids.sort();
        let question = format!(
            "Which policy governs {} on {}?",
            humanize(&action),
            resource
        );
        let listing = policy_ids
            .iter()
            .map(|id| {
                let info = &ctx.policy_info[id];
                format!(
                    "- {} ({}, priority {}): {}",
                    id,
                    info.modality,
                    info.priority,
                    info.description.as_deref().unwrap_or("no description")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        let answer = format!(
            "The following {} govern {} on {}:\n{}",
            if policy_ids.len() == 1 {
                "policy governs"
            } else {
                "policies"
            },
            humanize(&action),
            resource,
            listing
        );
        // Grammar fix for the single-policy case.
        let answer = answer.replacen("policy governs govern", "policy governs", 1);
        let mut source_refs = policy_ids.clone();
        source_refs.sort();
        records.push(LlmRecord {
            record_id: String::new(),
            family: "policy_explanation".to_string(),
            template_id: "policy_explanation_v1".to_string(),
            slots: vec![action.clone(), resource.clone()],
            label: None,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: SYSTEM_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user",
                    content: question,
                },
                ChatMessage {
                    role: "assistant",
                    content: answer,
                },
            ],
            source_refs,
            generation_method: "template_generation",
            review_status: "unreviewed",
            authz_tuple: None,
            split: "",
        });
    }
}

fn generate_counterexample_detection(ctx: &AiLearningContext, records: &mut Vec<LlmRecord>) {
    for case in &ctx.authority_cases {
        // Only allow/deny cases have a crisp contradiction to present.
        let (claim, correction_head) = match case.label {
            "allow" => (
                format!(
                    "{} is not permitted to {} on {}.",
                    case.role,
                    humanize(&case.operation),
                    case.resource_type
                ),
                "This claim is incorrect: the operation is permitted.",
            ),
            "deny" => (
                format!(
                    "{} is permitted to {} on {}.",
                    case.role,
                    humanize(&case.operation),
                    case.resource_type
                ),
                "This claim is incorrect: the operation is prohibited.",
            ),
            _ => continue,
        };
        let correction = format!(
            "{} The authority resolver decides '{}' for ({}, {}, {}). Governing policy: {}.",
            correction_head,
            case.label,
            case.role,
            humanize(&case.operation),
            case.resource_type,
            policy_citation(ctx, &case.applicable_policies)
        );
        records.push(LlmRecord {
            record_id: String::new(),
            family: "counterexample_detection".to_string(),
            template_id: "counterexample_detection_v1".to_string(),
            slots: vec![
                case.role.clone(),
                case.operation.clone(),
                case.resource_type.clone(),
            ],
            label: Some(case.label.to_string()),
            messages: vec![
                ChatMessage { role: "system", content: SYSTEM_PROMPT.to_string() },
                ChatMessage {
                    role: "user",
                    content: format!(
                        "Review this claim against the domain model and correct it if wrong: \"{claim}\""
                    ),
                },
                ChatMessage { role: "assistant", content: correction },
            ],
            source_refs: case_source_refs(case),
            generation_method: "authority_resolver",
            review_status: "unreviewed",
            authz_tuple: Some((
                case.role.clone(),
                case.operation.clone(),
                case.resource_type.clone(),
            )),
            split: "",
        });
    }
}

fn generate_entity_lookup(ctx: &AiLearningContext, records: &mut Vec<LlmRecord>) {
    let graph = ctx.graph;
    let mut subjects: Vec<(String, String, String, String)> = Vec::new(); // (kind, name, id, facts)

    let mut entities: Vec<_> = graph.all_entities();
    entities.sort_by_key(|e| e.name().to_string());
    for entity in entities {
        let mut facts = vec![format!(
            "{} is an entity in namespace '{}'.",
            entity.name(),
            entity.namespace()
        )];
        let roles = graph.role_names_for_entity(entity.id());
        if !roles.is_empty() {
            let mut roles = roles;
            roles.sort();
            facts.push(format!("Assigned roles: {}.", roles.join(", ")));
        }
        for flow in graph.flows_from(entity.id()) {
            if let Some(to) = graph.get_entity(flow.to_id()) {
                if let Some(resource) = graph.get_resource(flow.resource_id()) {
                    facts.push(format!("Sends '{}' to {}.", resource.name(), to.name()));
                }
            }
        }
        for flow in graph.flows_to(entity.id()) {
            if let Some(from) = graph.get_entity(flow.from_id()) {
                if let Some(resource) = graph.get_resource(flow.resource_id()) {
                    facts.push(format!(
                        "Receives '{}' from {}.",
                        resource.name(),
                        from.name()
                    ));
                }
            }
        }
        subjects.push((
            "entity".to_string(),
            entity.name().to_string(),
            entity.id().to_string(),
            facts.join(" "),
        ));
    }

    let mut roles: Vec<_> = graph.all_roles();
    roles.sort_by_key(|r| r.name().to_string());
    let relations = graph.all_relations();
    for role in roles {
        let mut facts = vec![format!(
            "{} is a role in namespace '{}'.",
            role.name(),
            role.namespace()
        )];
        let mut rel_facts: Vec<String> = Vec::new();
        for relation in &relations {
            if relation.subject_role() == role.id() {
                if let Some(object) = graph.get_role(relation.object_role()) {
                    rel_facts.push(format!(
                        "{} {} {} (relation '{}').",
                        role.name(),
                        relation.predicate(),
                        object.name(),
                        relation.name()
                    ));
                }
            }
            if relation.object_role() == role.id() {
                if let Some(subject) = graph.get_role(relation.subject_role()) {
                    rel_facts.push(format!(
                        "{} {} {} (relation '{}').",
                        subject.name(),
                        relation.predicate(),
                        role.name(),
                        relation.name()
                    ));
                }
            }
        }
        rel_facts.sort();
        facts.extend(rel_facts);
        subjects.push((
            "role".to_string(),
            role.name().to_string(),
            role.id().to_string(),
            facts.join(" "),
        ));
    }

    let mut resources: Vec<_> = graph.all_resources();
    resources.sort_by_key(|r| r.name().to_string());
    for resource in resources {
        subjects.push((
            "resource".to_string(),
            resource.name().to_string(),
            resource.id().to_string(),
            format!(
                "{} is a resource in namespace '{}' (unit: {}).",
                resource.name(),
                resource.namespace(),
                resource.unit_symbol()
            ),
        ));
    }

    for (kind, name, concept_id, facts) in subjects {
        records.push(LlmRecord {
            record_id: String::new(),
            family: "entity_lookup".to_string(),
            template_id: "entity_lookup_v1".to_string(),
            slots: vec![kind.clone(), name.clone()],
            label: None,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: SYSTEM_PROMPT.to_string(),
                },
                ChatMessage {
                    role: "user",
                    content: format!("What is {name}? What roles or relations does it have?"),
                },
                ChatMessage {
                    role: "assistant",
                    content: facts,
                },
            ],
            source_refs: vec![concept_id],
            generation_method: "template_generation",
            // Rule: verbatim graph rendering with no inference needs no review.
            review_status: "not_required",
            authz_tuple: None,
            split: "",
        });
    }
}

pub fn emit(
    ctx: &AiLearningContext,
    sink: &mut super::ArtifactSink,
) -> Result<Vec<String>, String> {
    let cfg = &ctx.recipe.projections.llm_dataset;
    let families: BTreeSet<&str> = cfg.families.iter().map(|s| s.as_str()).collect();
    if families.contains("authorization_qa")
        || families.contains("policy_explanation")
        || families.contains("counterexample_detection")
    {
        ctx.require_authority("llm_dataset authorization families")?;
    }

    let mut records: Vec<LlmRecord> = Vec::new();
    if families.contains("authorization_qa") {
        generate_authorization_qa(ctx, &mut records);
    }
    if families.contains("policy_explanation") {
        generate_policy_explanation(ctx, &mut records);
    }
    if families.contains("counterexample_detection") {
        generate_counterexample_detection(ctx, &mut records);
    }
    if families.contains("entity_lookup") {
        generate_entity_lookup(ctx, &mut records);
    }

    // Assign ids, dedup, validate, split — all deterministic.
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut rejected: Vec<RejectedArtifact> = Vec::new();
    let mut duplicate_exact_count = 0u64;
    let mut unsupported_reference_count = 0u64;
    let mut resolver_disagreement_count = 0u64;

    let known_refs = known_concept_refs(ctx);
    let case_by_tuple: BTreeMap<(String, String, String), &AuthorityCase> = ctx
        .authority_cases
        .iter()
        .map(|c| {
            (
                (c.role.clone(), c.operation.clone(), c.resource_type.clone()),
                c,
            )
        })
        .collect();

    let mut accepted: Vec<LlmRecord> = Vec::new();
    for mut record in records {
        record.record_id = record_id(&[
            &record.family,
            &record.template_id,
            &record.slots.join("\u{2}"),
            &record.source_refs.join("\u{2}"),
        ]);
        let key = record.dedup_key();
        if !seen.insert(key) {
            duplicate_exact_count += 1;
            rejected.push(RejectedArtifact {
                artifact_id: record.record_id.clone(),
                reason: "duplicate_exact".to_string(),
            });
            continue;
        }
        let mut bad_ref = false;
        for r in &record.source_refs {
            if !known_refs.contains(r) {
                bad_ref = true;
            }
        }
        if bad_ref {
            unsupported_reference_count += 1;
            rejected.push(RejectedArtifact {
                artifact_id: record.record_id.clone(),
                reason: "unsupported_reference".to_string(),
            });
            continue;
        }
        // Re-resolution check: the label must agree with a fresh resolver run.
        if let (Some(tuple), Some(label)) = (&record.authz_tuple, &record.label) {
            match case_by_tuple.get(tuple) {
                Some(case) if case.label == label.as_str() => {}
                _ => {
                    resolver_disagreement_count += 1;
                    rejected.push(RejectedArtifact {
                        artifact_id: record.record_id.clone(),
                        reason: "resolver_disagreement".to_string(),
                    });
                    continue;
                }
            }
        }
        record.split = assign_split(
            &record.record_id,
            &ctx.model_hash,
            ctx.seed,
            &ctx.recipe.splits,
        );
        accepted.push(record);
    }

    // Tiny-dataset rule: train must be non-empty.
    let mut split_values: Vec<&'static str> = accepted.iter().map(|r| r.split).collect();
    let mut split_warnings: Vec<String> = Vec::new();
    if let Some(idx) = ensure_train_nonempty(&mut split_values) {
        accepted[idx].split = "train";
        split_warnings.push(
            "train split was empty; reassigned one record to train (tiny-dataset rule)".to_string(),
        );
    }

    // Serialize per split.
    let mut lines: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    lines.insert("train", vec![]);
    lines.insert("validation", vec![]);
    lines.insert("test", vec![]);
    let mut family_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut split_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut covered_refs: BTreeSet<String> = BTreeSet::new();
    for record in &accepted {
        let provenance = ctx.provenance(
            record.source_refs.clone(),
            record.generation_method,
            record.review_status,
        );
        let value = record.to_chatml(&provenance);
        let line = serde_json::to_string(&value).map_err(|e| e.to_string())?;
        lines.get_mut(record.split).unwrap().push(line);
        *family_counts.entry(record.family.clone()).or_default() += 1;
        *split_counts.entry(record.split.to_string()).or_default() += 1;
        covered_refs.extend(record.source_refs.iter().cloned());
    }

    let mut artifacts = Vec::new();
    for (split, rows) in &lines {
        let filename = format!("{split}.jsonl");
        let mut content = rows.join("\n");
        if !content.is_empty() {
            content.push('\n');
        }
        sink.write(&filename, &content)?;
        artifacts.push(filename);
    }

    // Reports.
    let uncovered: Vec<String> = known_refs.difference(&covered_refs).cloned().collect();

    let manifest = json!({
        "dataset_kind": "llm_dataset",
        "format": cfg.format,
        "families": cfg.families,
        "record_count": accepted.len(),
        "splits": super::report::counts_to_value(&split_counts),
        "provenance": ctx.provenance(vec![], "template_generation", "unreviewed"),
    });
    sink.write("dataset_manifest.json", &super::to_json_pretty(&manifest)?)?;
    artifacts.push("dataset_manifest.json".to_string());

    let mut details = Map::new();
    details.insert(
        "rows_generated".into(),
        Value::from(accepted.len() as u64 + rejected.len() as u64),
    );
    details.insert("rows_accepted".into(), Value::from(accepted.len() as u64));
    details.insert("rows_rejected".into(), Value::from(rejected.len() as u64));
    details.insert(
        "duplicate_exact_count".into(),
        Value::from(duplicate_exact_count),
    );
    details.insert(
        "unsupported_reference_count".into(),
        Value::from(unsupported_reference_count),
    );
    details.insert(
        "resolver_disagreement_count".into(),
        Value::from(resolver_disagreement_count),
    );
    details.insert(
        "family_counts".into(),
        super::report::counts_to_value(&family_counts),
    );
    details.insert(
        "split_counts".into(),
        super::report::counts_to_value(&split_counts),
    );
    let validation = ValidationReport {
        projection_kind: "llm_dataset".to_string(),
        status: "passed".to_string(),
        requirements_passed: accepted.len() as u64,
        requirements_failed: 0,
        warnings: split_warnings.clone(),
        errors: vec![],
        rejected_artifacts: rejected,
        repair_recommendations: vec![],
        details,
    };
    sink.write(
        "validation_report.json",
        &super::to_json_pretty(&validation)?,
    )?;
    artifacts.push("validation_report.json".to_string());

    let mut cov_details = Map::new();
    cov_details.insert(
        "source_coverage".into(),
        json!({
            "covered_refs": covered_refs.len(),
            "known_refs": known_refs.len(),
        }),
    );
    cov_details.insert(
        "family_counts".into(),
        super::report::counts_to_value(&family_counts),
    );
    let coverage = CoverageReport {
        projection_kind: "llm_dataset".to_string(),
        details: cov_details,
        known_gaps: uncovered,
    };
    sink.write("coverage_report.json", &super::to_json_pretty(&coverage)?)?;
    artifacts.push("coverage_report.json".to_string());

    let generation = GenerationReport {
        projection_kind: "llm_dataset".to_string(),
        recipe_ref: ctx.recipe_ref.clone(),
        recipe_hash: ctx.recipe_hash.clone(),
        source_model_ref: ctx.model_ref.clone(),
        source_model_hash: ctx.model_hash.clone(),
        generator_version: super::GENERATOR_VERSION.to_string(),
        seed: ctx.seed,
        started_at: ctx.created_at.clone(),
        completed_at: ctx.created_at.clone(),
        artifacts_generated: artifacts.clone(),
        families_generated: cfg.families.clone(),
        generation_methods_used: vec![
            "authority_resolver".to_string(),
            "template_generation".to_string(),
        ],
        warnings: split_warnings,
        errors: vec![],
    };
    sink.write(
        "generation_report.json",
        &super::to_json_pretty(&generation)?,
    )?;
    artifacts.push("generation_report.json".to_string());

    Ok(artifacts)
}

/// Every ref a record may legally cite: ConceptIds of graph concepts plus
/// authority-pack policy ids.
fn known_concept_refs(ctx: &AiLearningContext) -> BTreeSet<String> {
    let mut refs = BTreeSet::new();
    for e in ctx.graph.all_entities() {
        refs.insert(e.id().to_string());
    }
    for r in ctx.graph.all_roles() {
        refs.insert(r.id().to_string());
    }
    for r in ctx.graph.all_resources() {
        refs.insert(r.id().to_string());
    }
    // Flow ids are intentionally excluded: they are fresh UUIDs on every
    // parse, so records must not (and do not) cite them.
    for r in ctx.graph.all_relations() {
        refs.insert(r.id().to_string());
    }
    refs.extend(ctx.policy_info.keys().cloned());
    refs
}

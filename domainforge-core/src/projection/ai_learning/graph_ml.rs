//! Graph ML Dataset Projection: export the semantic graph for downstream
//! graph learning. Negative samples are resolver-grounded, never invented.

use super::report::{CoverageReport, GenerationReport, RejectedArtifact, ValidationReport};
use super::split::assign_split;
use super::AiLearningContext;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use xxhash_rust::xxh64::xxh64;

#[derive(Debug, Clone, Serialize)]
struct Node {
    node_id: String,
    name: String,
    node_type: String,
    namespace: String,
}

#[derive(Debug, Clone, Serialize)]
struct Edge {
    edge_id: String,
    edge_type: String,
    source: String,
    target: String,
    #[serde(skip_serializing_if = "Map::is_empty")]
    attributes: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
struct NegativeSample {
    sample_id: String,
    role_ref: String,
    operation: String,
    resource_ref: String,
    label: String,
    reason: String,
    resolver_reason_code: String,
    applicable_policies: Vec<String>,
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

pub fn emit(
    ctx: &AiLearningContext,
    sink: &mut super::ArtifactSink,
) -> Result<Vec<String>, String> {
    let cfg = &ctx.recipe.projections.graph_ml_dataset;
    let tasks: BTreeSet<&str> = cfg.tasks.iter().map(|s| s.as_str()).collect();
    if tasks.contains("policy_violation_detection") {
        ctx.require_authority("graph_ml policy_violation_detection")?;
    }

    let graph = ctx.graph;
    let mut nodes: Vec<Node> = Vec::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut errors: Vec<String> = Vec::new();

    // Nodes: what the Graph actually exposes, categorical features only.
    for e in graph.all_entities() {
        nodes.push(Node {
            node_id: e.id().to_string(),
            name: e.name().to_string(),
            node_type: "entity".to_string(),
            namespace: e.namespace().to_string(),
        });
    }
    for r in graph.all_roles() {
        nodes.push(Node {
            node_id: r.id().to_string(),
            name: r.name().to_string(),
            node_type: "role".to_string(),
            namespace: r.namespace().to_string(),
        });
    }
    for r in graph.all_resources() {
        nodes.push(Node {
            node_id: r.id().to_string(),
            name: r.name().to_string(),
            node_type: "resource".to_string(),
            namespace: r.namespace().to_string(),
        });
    }
    // Authority-pack policies participate as nodes so applicability is learnable.
    for info in ctx.policy_info.values() {
        nodes.push(Node {
            node_id: info.policy_id.clone(),
            name: info.policy_id.clone(),
            node_type: "policy".to_string(),
            namespace: info.pack_id.clone(),
        });
    }
    nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    // Edges from the actual relation/assignment vocabulary.
    let mut push_edge =
        |edge_type: &str, source: String, target: String, attributes: Map<String, Value>| {
            let edge_id = super::split::record_id(&[edge_type, &source, &target]);
            edges.push(Edge {
                edge_id,
                edge_type: edge_type.to_string(),
                source,
                target,
                attributes,
            });
        };

    for entity in graph.all_entities() {
        if let Some(role_ids) = graph.roles_for_entity(entity.id()) {
            for role_id in role_ids {
                push_edge(
                    "role_assignment",
                    entity.id().to_string(),
                    role_id.to_string(),
                    Map::new(),
                );
            }
        }
    }
    for flow in graph.all_flows() {
        let mut attributes = Map::new();
        attributes.insert(
            "resource".into(),
            Value::String(flow.resource_id().to_string()),
        );
        push_edge(
            "flow",
            flow.from_id().to_string(),
            flow.to_id().to_string(),
            attributes,
        );
    }
    for relation in graph.all_relations() {
        let mut attributes = Map::new();
        attributes.insert(
            "predicate".into(),
            Value::String(relation.predicate().to_string()),
        );
        attributes.insert(
            "relation_name".into(),
            Value::String(relation.name().to_string()),
        );
        push_edge(
            "relation",
            relation.subject_role().to_string(),
            relation.object_role().to_string(),
            attributes,
        );
    }
    for info in ctx.policy_info.values() {
        if let Some(role_name) = &info.actor_role {
            if let Some(role_id) = graph.find_role_by_name(role_name) {
                push_edge(
                    "policy_applies_to_role",
                    info.policy_id.clone(),
                    role_id.to_string(),
                    Map::new(),
                );
            }
        }
        if let Some(resource_name) = &info.resource_type {
            if let Some(resource_id) = graph.find_resource_by_name(resource_name) {
                push_edge(
                    "policy_applies_to_resource",
                    info.policy_id.clone(),
                    resource_id.to_string(),
                    Map::new(),
                );
            }
        }
    }
    edges.sort_by(|a, b| a.edge_id.cmp(&b.edge_id));

    // Validation: every edge endpoint must be a known node.
    let node_ids: BTreeSet<&str> = nodes.iter().map(|n| n.node_id.as_str()).collect();
    let mut rejected: Vec<RejectedArtifact> = Vec::new();
    edges.retain(|edge| {
        let ok = node_ids.contains(edge.source.as_str()) && node_ids.contains(edge.target.as_str());
        if !ok {
            rejected.push(RejectedArtifact {
                artifact_id: edge.edge_id.clone(),
                reason: "edge_references_missing_node".to_string(),
            });
        }
        ok
    });

    // Resolver-grounded samples for policy_violation_detection.
    let mut positives: Vec<NegativeSample> = Vec::new();
    let mut negatives: Vec<NegativeSample> = Vec::new();
    if tasks.contains("policy_violation_detection") {
        for case in &ctx.authority_cases {
            let sample_id = super::split::record_id(&[
                "authz_sample",
                &case.role_ref,
                &case.operation,
                &case.resource_ref,
            ]);
            let sample = |label: &str, reason: &str| NegativeSample {
                sample_id: sample_id.clone(),
                role_ref: case.role_ref.clone(),
                operation: case.operation.clone(),
                resource_ref: case.resource_ref.clone(),
                label: label.to_string(),
                reason: reason.to_string(),
                resolver_reason_code: case.reason_code.clone(),
                applicable_policies: case.applicable_policies.clone(),
            };
            match case.label {
                // Allow is a positive sample and must never be emitted as negative.
                "allow" => positives.push(sample("positive", "allowed_by_policy")),
                "deny" | "reject" => negatives.push(sample("negative", "forbidden_by_policy")),
                "escalate" => negatives.push(sample("negative", "requires_escalation")),
                "unknown" => {
                    // Open-world safety: Unknown is not evidence of prohibition.
                    if cfg.treat_unknown_as_negative {
                        negatives.push(sample("negative", "unknown_authority"));
                    }
                }
                other => errors.push(format!("unhandled decision label '{other}'")),
            }
        }
        // Hard error path: a negative sample whose resolver decision is Allow
        // is a generator bug.
        let allow_tuples: BTreeSet<String> = ctx
            .authority_cases
            .iter()
            .filter(|c| c.label == "allow")
            .map(|c| format!("{}|{}|{}", c.role_ref, c.operation, c.resource_ref))
            .collect();
        for neg in &negatives {
            let key = format!("{}|{}|{}", neg.role_ref, neg.operation, neg.resource_ref);
            if allow_tuples.contains(&key) {
                return Err(format!(
                    "generator bug: negative sample {} has resolver decision Allow",
                    neg.sample_id
                ));
            }
        }
        // Deterministic cap: keep the lowest stable-hash negatives.
        let cap = (positives.len() as u64).saturating_mul(cfg.max_negative_ratio as u64) as usize;
        if negatives.len() > cap {
            negatives.sort_by_key(|n| {
                xxh64(
                    format!("{}\u{1}{}\u{1}{}", n.sample_id, ctx.model_hash, ctx.seed).as_bytes(),
                    ctx.seed,
                )
            });
            negatives.truncate(cap);
            negatives.sort_by(|a, b| a.sample_id.cmp(&b.sample_id));
        }
    }

    // Split masks over nodes (node_classification) — disjoint by construction.
    let mut masks: BTreeMap<&str, Vec<String>> = BTreeMap::new();
    masks.insert("train", vec![]);
    masks.insert("validation", vec![]);
    masks.insert("test", vec![]);
    for node in &nodes {
        let split = assign_split(&node.node_id, &ctx.model_hash, ctx.seed, &ctx.recipe.splits);
        masks.get_mut(split).unwrap().push(node.node_id.clone());
    }
    let mut warnings: Vec<String> = Vec::new();
    if masks["train"].is_empty() && !nodes.is_empty() {
        let moved = nodes[0].node_id.clone();
        for list in masks.values_mut() {
            list.retain(|id| id != &moved);
        }
        masks.get_mut("train").unwrap().push(moved);
        warnings.push(
            "train mask was empty; reassigned one node to train (tiny-dataset rule)".to_string(),
        );
    }

    // Labels per task.
    let mut labels = Map::new();
    if tasks.contains("node_classification") {
        let mut node_labels = Map::new();
        for node in &nodes {
            node_labels.insert(node.node_id.clone(), Value::String(node.node_type.clone()));
        }
        labels.insert("node_classification".into(), Value::Object(node_labels));
    }
    if tasks.contains("link_prediction") {
        labels.insert(
            "link_prediction".into(),
            json!({
                "positive_edges": edges.iter().map(|e| e.edge_id.clone()).collect::<Vec<_>>(),
                "negative_samples_ref": "negative_samples.json",
            }),
        );
    }
    if tasks.contains("policy_violation_detection") {
        labels.insert(
            "policy_violation_detection".into(),
            json!({
                "positive_count": positives.len(),
                "negative_count": negatives.len(),
                "samples_ref": "negative_samples.json",
            }),
        );
    }

    let mut artifacts = Vec::new();

    let graph_json = json!({
        "nodes": nodes,
        "edges": edges,
        "features": {
            "node": ["node_type", "namespace"],
            "edge": ["edge_type"],
            "note": "categorical only; no invented numeric features in v0",
        },
        "provenance": ctx.provenance(vec![], "graph_export", "unreviewed"),
    });
    sink.write("graph.json", &super::to_json_pretty(&graph_json)?)?;
    artifacts.push("graph.json".to_string());

    let mut nodes_csv = String::from("node_id,name,node_type,namespace\n");
    for n in &nodes {
        nodes_csv.push_str(&format!(
            "{},{},{},{}\n",
            csv_escape(&n.node_id),
            csv_escape(&n.name),
            csv_escape(&n.node_type),
            csv_escape(&n.namespace)
        ));
    }
    sink.write("nodes.csv", &nodes_csv)?;
    artifacts.push("nodes.csv".to_string());

    let mut edges_csv = String::from("edge_id,edge_type,source,target\n");
    for e in &edges {
        edges_csv.push_str(&format!(
            "{},{},{},{}\n",
            csv_escape(&e.edge_id),
            csv_escape(&e.edge_type),
            csv_escape(&e.source),
            csv_escape(&e.target)
        ));
    }
    sink.write("edges.csv", &edges_csv)?;
    artifacts.push("edges.csv".to_string());

    sink.write(
        "labels.json",
        &super::to_json_pretty(&Value::Object(labels))?,
    )?;
    artifacts.push("labels.json".to_string());

    sink.write("masks.json", &super::to_json_pretty(&json!(masks))?)?;
    artifacts.push("masks.json".to_string());

    let neg_json = json!({
        "positives": positives,
        "negatives": negatives,
        "policy": {
            "unknown_excluded_by_default": !cfg.treat_unknown_as_negative,
            "max_negative_ratio": cfg.max_negative_ratio,
        },
        "provenance": ctx.provenance(vec![], "authority_resolver", "unreviewed"),
    });
    sink.write("negative_samples.json", &super::to_json_pretty(&neg_json)?)?;
    artifacts.push("negative_samples.json".to_string());

    // Reports.
    let mut node_type_counts: BTreeMap<String, u64> = BTreeMap::new();
    for n in &nodes {
        *node_type_counts.entry(n.node_type.clone()).or_default() += 1;
    }
    let mut edge_type_counts: BTreeMap<String, u64> = BTreeMap::new();
    for e in &edges {
        *edge_type_counts.entry(e.edge_type.clone()).or_default() += 1;
    }
    let mut reason_counts: BTreeMap<String, u64> = BTreeMap::new();
    for n in &negatives {
        *reason_counts.entry(n.reason.clone()).or_default() += 1;
    }
    let connected: BTreeSet<&str> = edges
        .iter()
        .flat_map(|e| [e.source.as_str(), e.target.as_str()])
        .collect();
    let orphans: Vec<String> = nodes
        .iter()
        .filter(|n| !connected.contains(n.node_id.as_str()))
        .map(|n| n.node_id.clone())
        .collect();

    let manifest = json!({
        "dataset_kind": "graph_ml_dataset",
        "tasks": cfg.tasks,
        "node_count": nodes.len(),
        "edge_count": edges.len(),
        "provenance": ctx.provenance(vec![], "graph_export", "unreviewed"),
    });
    sink.write("dataset_manifest.json", &super::to_json_pretty(&manifest)?)?;
    artifacts.push("dataset_manifest.json".to_string());

    let mut details = Map::new();
    details.insert("node_count".into(), Value::from(nodes.len() as u64));
    details.insert("edge_count".into(), Value::from(edges.len() as u64));
    details.insert("rejected_edges".into(), Value::from(rejected.len() as u64));
    let status = if errors.is_empty() {
        "passed"
    } else {
        "failed"
    };
    let validation = ValidationReport {
        projection_kind: "graph_ml_dataset".to_string(),
        status: status.to_string(),
        requirements_passed: (nodes.len() + edges.len()) as u64,
        requirements_failed: rejected.len() as u64,
        warnings: warnings.clone(),
        errors: errors.clone(),
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
        "node_type_distribution".into(),
        super::report::counts_to_value(&node_type_counts),
    );
    cov_details.insert(
        "edge_type_distribution".into(),
        super::report::counts_to_value(&edge_type_counts),
    );
    cov_details.insert(
        "negative_sample_reasons".into(),
        super::report::counts_to_value(&reason_counts),
    );
    cov_details.insert(
        "negative_sample_count".into(),
        Value::from(negatives.len() as u64),
    );
    cov_details.insert(
        "positive_sample_count".into(),
        Value::from(positives.len() as u64),
    );
    cov_details.insert("orphan_nodes".into(), json!(orphans.clone()));
    let coverage = CoverageReport {
        projection_kind: "graph_ml_dataset".to_string(),
        details: cov_details,
        known_gaps: orphans,
    };
    sink.write("coverage_report.json", &super::to_json_pretty(&coverage)?)?;
    artifacts.push("coverage_report.json".to_string());

    let generation = GenerationReport {
        projection_kind: "graph_ml_dataset".to_string(),
        recipe_ref: ctx.recipe_ref.clone(),
        recipe_hash: ctx.recipe_hash.clone(),
        source_model_ref: ctx.model_ref.clone(),
        source_model_hash: ctx.model_hash.clone(),
        generator_version: super::GENERATOR_VERSION.to_string(),
        seed: ctx.seed,
        started_at: ctx.created_at.clone(),
        completed_at: ctx.created_at.clone(),
        artifacts_generated: artifacts.clone(),
        families_generated: cfg.tasks.clone(),
        generation_methods_used: vec!["graph_export".to_string(), "authority_resolver".to_string()],
        warnings,
        errors,
    };
    sink.write(
        "generation_report.json",
        &super::to_json_pretty(&generation)?,
    )?;
    artifacts.push("generation_report.json".to_string());

    Ok(artifacts)
}

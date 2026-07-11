//! AI Learning Projections: compile `.sea` models into machine-learnable and
//! machine-evaluable artifacts (LLM JSONL, graph ML exports, CEP eval cases).
//!
//! Design rule: no generated artifact may invent domain facts. Authorization
//! labels and answer keys are computed by the authority resolver over an
//! AuthorityEnvironmentConfig supplied alongside the model; everything else is
//! a verbatim rendering of the parsed Graph.

pub mod cep_eval;
pub mod graph_ml;
pub mod llm;
pub mod provenance;
pub mod recipe;
pub mod report;
pub mod split;

use crate::authority::{
    ActorContext, AuthorityEnvironment, AuthorityRequest, FinalDecision, ResourceRef,
};
use crate::graph::Graph;
use crate::semantic_pack::canonical_json::hash_canonical_json;
use recipe::Recipe;
use std::collections::{BTreeMap, BTreeSet};

pub const GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

/// One resolver-grounded authorization case: a (role, operation, resource
/// type) tuple plus the resolver's decision over it with empty facts.
#[derive(Debug, Clone)]
pub struct AuthorityCase {
    pub role: String,
    pub role_ref: String,
    pub operation: String,
    pub resource_type: String,
    pub resource_ref: String,
    pub decision: FinalDecision,
    /// "allow" | "deny" | "escalate" | "unknown" | "reject"
    pub label: &'static str,
    pub reason_code: String,
    pub candidate_policies: Vec<String>,
    pub applicable_policies: Vec<String>,
}

/// Static description of one authority-pack policy, for explanations and refs.
#[derive(Debug, Clone)]
pub struct PolicyInfo {
    pub policy_id: String,
    pub pack_id: String,
    pub modality: String,
    pub priority: i32,
    pub action: Option<String>,
    pub actor_role: Option<String>,
    pub resource_type: Option<String>,
    pub description: Option<String>,
}

pub struct AiLearningContext<'a> {
    pub graph: &'a Graph,
    pub model_ref: String,
    pub model_hash: String,
    pub recipe: Recipe,
    pub recipe_ref: String,
    pub recipe_hash: String,
    pub seed: u64,
    pub created_at: String,
    pub authority_config_ref: Option<String>,
    pub authority: Option<AuthorityEnvironment>,
    pub authority_cases: Vec<AuthorityCase>,
    pub policy_info: BTreeMap<String, PolicyInfo>,
}

/// The resolver has no "Unknown" final decision; a tuple with no applicable
/// policy resolves NotApplicable, which for dataset purposes means the model
/// does not contain enough authority information — labeled "unknown".
pub fn decision_label(decision: FinalDecision) -> &'static str {
    match decision {
        FinalDecision::Allow => "allow",
        FinalDecision::Deny => "deny",
        FinalDecision::Escalate => "escalate",
        FinalDecision::NotApplicable => "unknown",
        FinalDecision::Reject => "reject",
    }
}

impl<'a> AiLearningContext<'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn build(
        graph: &'a Graph,
        model_ref: String,
        recipe: Recipe,
        recipe_ref: String,
        seed_override: Option<u64>,
        created_at_override: Option<String>,
        authority: Option<AuthorityEnvironment>,
        authority_config_ref: Option<String>,
    ) -> Result<Self, String> {
        let model_hash = hash_canonical_json(&stable_graph_value(graph));
        let recipe_value = serde_json::to_value(&recipe)
            .map_err(|e| format!("failed to serialize recipe for hashing: {e}"))?;
        let recipe_hash = hash_canonical_json(&recipe_value);
        let seed = seed_override.unwrap_or(recipe.seed);
        let created_at = created_at_override.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        let (authority_cases, policy_info) = match &authority {
            Some(env) => (
                enumerate_authority_cases(graph, env)?,
                collect_policy_info(env),
            ),
            None => (Vec::new(), BTreeMap::new()),
        };

        Ok(Self {
            graph,
            model_ref,
            model_hash,
            recipe,
            recipe_ref,
            recipe_hash,
            seed,
            created_at,
            authority_config_ref,
            authority,
            authority_cases,
            policy_info,
        })
    }

    pub fn require_authority(&self, needed_for: &str) -> Result<(), String> {
        if self.authority.is_none() {
            return Err(format!(
                "{needed_for} requires resolver-grounded labels: provide an \
                 authority environment via --authority-config or the recipe's \
                 'authority_config' field"
            ));
        }
        Ok(())
    }

    pub fn provenance(
        &self,
        source_refs: Vec<String>,
        generation_method: &str,
        review_status: &str,
    ) -> provenance::Provenance {
        provenance::Provenance {
            source_model_ref: self.model_ref.clone(),
            source_model_hash: self.model_hash.clone(),
            source_refs,
            generation_method: generation_method.to_string(),
            generator_version: GENERATOR_VERSION.to_string(),
            recipe_ref: self.recipe_ref.clone(),
            recipe_hash: self.recipe_hash.clone(),
            seed: self.seed,
            created_at: self.created_at.clone(),
            validation_status: "validated".to_string(),
            review_status: review_status.to_string(),
        }
    }
}

/// A run-stable canonical rendering of the graph for source_model_hash.
/// Raw Graph serialization is NOT run-stable (flow ids are fresh UUIDs each
/// parse), so the hash is built from semantic content via accessors: sorted
/// concepts with deterministic ConceptIds, flows keyed by their endpoints.
/// Collections not itemized here (patterns, concept changes, instances)
/// contribute their counts so gross changes still alter the hash.
fn stable_graph_value(graph: &Graph) -> serde_json::Value {
    use serde_json::json;
    let mut entities: Vec<_> = graph
        .all_entities()
        .into_iter()
        .map(|e| json!({"id": e.id().to_string(), "name": e.name(), "namespace": e.namespace()}))
        .collect();
    let mut roles: Vec<_> = graph
        .all_roles()
        .into_iter()
        .map(|r| json!({"id": r.id().to_string(), "name": r.name(), "namespace": r.namespace()}))
        .collect();
    let mut resources: Vec<_> = graph
        .all_resources()
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id().to_string(), "name": r.name(),
                "namespace": r.namespace(), "unit": r.unit_symbol(),
            })
        })
        .collect();
    let mut relations: Vec<_> = graph
        .all_relations()
        .into_iter()
        .map(|r| {
            json!({
                "id": r.id().to_string(), "name": r.name(),
                "subject": r.subject_role().to_string(),
                "predicate": r.predicate(),
                "object": r.object_role().to_string(),
            })
        })
        .collect();
    let mut flows: Vec<_> = graph
        .all_flows()
        .into_iter()
        .map(|f| {
            json!({
                "resource": f.resource_id().to_string(),
                "from": f.from_id().to_string(),
                "to": f.to_id().to_string(),
                "quantity": f.quantity().to_string(),
            })
        })
        .collect();
    let mut policies: Vec<_> = graph
        .all_policies()
        .into_iter()
        .map(|p| json!({"id": p.id.to_string(), "name": p.name, "namespace": p.namespace}))
        .collect();
    let mut assignments: Vec<_> = graph
        .all_entities()
        .into_iter()
        .flat_map(|e| {
            graph
                .roles_for_entity(e.id())
                .map(|roles| {
                    roles
                        .iter()
                        .map(|r| json!({"entity": e.id().to_string(), "role": r.to_string()}))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
        })
        .collect();
    let sort_key = |v: &serde_json::Value| serde_json::to_string(v).unwrap_or_default();
    for list in [
        &mut entities,
        &mut roles,
        &mut resources,
        &mut relations,
        &mut flows,
        &mut policies,
        &mut assignments,
    ] {
        list.sort_by_key(&sort_key);
    }
    json!({
        "entities": entities,
        "roles": roles,
        "resources": resources,
        "relations": relations,
        "flows": flows,
        "policies": policies,
        "role_assignments": assignments,
        "other_counts": {
            "patterns": graph.pattern_count(),
            "instances": graph.instance_count(),
            "entity_instances": graph.entity_instance_count(),
        },
    })
}

/// Enumerate (role, operation, resource type) tuples from the graph and the
/// pack action vocabulary, resolving each with empty facts. Sorted for
/// deterministic output.
fn enumerate_authority_cases(
    graph: &Graph,
    env: &AuthorityEnvironment,
) -> Result<Vec<AuthorityCase>, String> {
    let mut roles: Vec<(&str, String)> = graph
        .all_roles()
        .into_iter()
        .map(|r| (r.name(), r.id().to_string()))
        .collect();
    roles.sort();

    let mut resources: Vec<(&str, String)> = graph
        .all_resources()
        .into_iter()
        .map(|r| (r.name(), r.id().to_string()))
        .collect();
    resources.sort();

    let mut operations: BTreeSet<String> = BTreeSet::new();
    for pack in env.packs() {
        for policy in &pack.policies {
            if let Some(serde_json::Value::String(action)) =
                policy.applies_to.predicates.get("action")
            {
                operations.insert(action.clone());
            }
        }
    }

    let mut cases = Vec::new();
    for (role, role_ref) in &roles {
        for operation in &operations {
            for (resource, resource_ref) in &resources {
                let request = AuthorityRequest {
                    request_id: format!("ai-learning:{role}:{operation}:{resource}"),
                    actor: ActorContext {
                        id: format!("{role}-representative"),
                        role: Some(role.to_string()),
                        groups: vec![],
                        service_account: None,
                        agent_identity: None,
                    },
                    operation: operation.clone(),
                    resource: ResourceRef {
                        id: None,
                        type_: Some(resource.to_string()),
                        extra: Default::default(),
                    },
                    context: serde_json::json!({}),
                    requested_at: chrono::Utc::now(),
                    correlation_id: None,
                    risk_class: None,
                    metadata: Default::default(),
                };
                let (trace, decision) = env.evaluate(&request, &[]).map_err(|e| {
                    format!("authority evaluation failed for {role}/{operation}/{resource}: {e}")
                })?;
                cases.push(AuthorityCase {
                    role: role.to_string(),
                    role_ref: role_ref.clone(),
                    operation: operation.clone(),
                    resource_type: resource.to_string(),
                    resource_ref: resource_ref.clone(),
                    decision: decision.final_decision,
                    label: decision_label(decision.final_decision),
                    reason_code: decision.reason_code,
                    candidate_policies: trace.candidate_policies,
                    applicable_policies: trace.applicable_policies,
                });
            }
        }
    }
    Ok(cases)
}

fn collect_policy_info(env: &AuthorityEnvironment) -> BTreeMap<String, PolicyInfo> {
    let mut info = BTreeMap::new();
    for pack in env.packs() {
        for policy in &pack.policies {
            let get = |key: &str| -> Option<String> {
                match policy.applies_to.predicates.get(key) {
                    Some(serde_json::Value::String(s)) => Some(s.clone()),
                    _ => None,
                }
            };
            info.insert(
                policy.policy_id.clone(),
                PolicyInfo {
                    policy_id: policy.policy_id.clone(),
                    pack_id: pack.id.clone(),
                    modality: policy.modality.to_string(),
                    priority: policy.priority,
                    action: get("action"),
                    actor_role: get("actor.role"),
                    resource_type: get("resource.type"),
                    description: policy.description.clone(),
                },
            );
        }
    }
    info
}

/// `ArtifactSink` now lives in the shared projection kernel; re-exported here
/// so existing ai-learning call sites keep the `ai_learning::ArtifactSink`
/// path.
pub use crate::projection::sink::ArtifactSink;

/// Run every enabled projection from JSON inputs and return the artifacts as
/// a relative-path → content map. This is the binding surface: string in,
/// strings out, no filesystem — the layout matches `--format ai-learning`.
pub fn project_ai_learning_in_memory(
    graph: &Graph,
    recipe_json: Option<&str>,
    authority_config_json: Option<&str>,
    model_ref: &str,
    seed: Option<u64>,
    created_at: Option<String>,
) -> Result<BTreeMap<String, String>, String> {
    let recipe = match recipe_json {
        Some(json) => Recipe::from_json(json)?,
        None => Recipe::default(),
    };
    let authority = match authority_config_json {
        Some(json) => {
            let config: crate::authority::AuthorityEnvironmentConfig =
                serde_json::from_str(json).map_err(|e| format!("invalid authority config: {e}"))?;
            let mut env = AuthorityEnvironment::new(config)
                .map_err(|e| format!("failed to create authority environment: {e}"))?;
            env.validate()
                .map_err(|e| format!("authority environment validation failed: {e}"))?;
            Some(env)
        }
        None => None,
    };
    let ctx = AiLearningContext::build(
        graph,
        model_ref.to_string(),
        recipe,
        "<inline>".to_string(),
        seed,
        created_at,
        authority,
        authority_config_json.map(|_| "<inline>".to_string()),
    )?;

    let mut map = BTreeMap::new();
    if ctx.recipe.projections.llm_dataset.enabled {
        let mut sink = ArtifactSink::Memory {
            prefix: "llm_dataset/".to_string(),
            map: &mut map,
        };
        llm::emit(&ctx, &mut sink)?;
    }
    if ctx.recipe.projections.graph_ml_dataset.enabled {
        let mut sink = ArtifactSink::Memory {
            prefix: "graph_dataset/".to_string(),
            map: &mut map,
        };
        graph_ml::emit(&ctx, &mut sink)?;
    }
    if ctx.recipe.projections.cep_eval_dataset.enabled {
        let mut sink = ArtifactSink::Memory {
            prefix: "cep_eval/".to_string(),
            map: &mut map,
        };
        cep_eval::emit(&ctx, &mut sink)?;
    }
    Ok(map)
}

pub(crate) fn to_json_pretty<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string_pretty(value)
        .map(|s| s + "\n")
        .map_err(|e| format!("serialization failed: {e}"))
}

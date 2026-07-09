//! Cedar authority projection: a SEA model becomes a Cedar authorization
//! schema + policy set, so the architecture's authority structure (who may
//! move which resource) is consumable by a Cedar policy engine.
//!
//! Mapping:
//! - model namespace → Cedar namespace
//! - `Entity` + `Resource` declarations → `entityTypes` (each `{}`, no shape —
//!   the SEA model does not declare entity attributes)
//! - each `Flow` (`<Resource> from <From> to <To>`) → an `Action`
//!   `Issue<Resource>` whose `appliesTo.principalTypes = [From]` and
//!   `resourceTypes = [Resource]`. Multiple flows issuing the same resource
//!   merge their principals. Cedar's request model is principal+resource only,
//!   so the `to` endpoint is recorded in the action's `to` annotation rather
//!   than in `appliesTo`.
//!
//! Output is two files: `schema.cedarschema.json` (entity types + actions,
//! sorted for byte-identical runs) and `policies.cedar` (one `permit` per
//! action — the baseline authority grant that the flow exists in the model).
//!
//! [Cedar]: https://docs.cedarpolicy.com/schema/json-schema.html

use crate::graph::Graph;
use crate::projection::sink::ArtifactSink;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Relative paths of the emitted artifacts.
pub const SCHEMA_FILE: &str = "schema.cedarschema.json";
pub const POLICIES_FILE: &str = "policies.cedar";

/// Emit the Cedar schema + policies into `sink`; returns emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let ns = model_namespace(graph);

    let schema = build_schema(graph, &ns);
    let policies = build_policies(graph, &ns, model_ref, &created_at);

    sink.write(SCHEMA_FILE, &schema)?;
    sink.write(POLICIES_FILE, &policies)?;
    Ok(vec![SCHEMA_FILE.to_string(), POLICIES_FILE.to_string()])
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_cedar_in_memory(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
) -> Result<BTreeMap<String, String>, String> {
    let mut map = BTreeMap::new();
    let mut sink = ArtifactSink::Memory {
        prefix: String::new(),
        map: &mut map,
    };
    emit(graph, model_ref, created_at, &mut sink)?;
    Ok(map)
}

/// Build the Cedar JSON schema (entityTypes + actions under the namespace).
fn build_schema(graph: &Graph, ns: &str) -> String {
    let entity_name: BTreeMap<String, String> = graph
        .all_entities()
        .iter()
        .map(|e| (e.id().to_string(), e.name().to_string()))
        .collect();
    let resource_name: BTreeMap<String, String> = graph
        .all_resources()
        .iter()
        .map(|r| (r.id().to_string(), r.name().to_string()))
        .collect();

    // entityTypes: union of entities + resources (both are Cedar entity types).
    let mut type_names: Vec<String> = Vec::new();
    for e in graph.all_entities() {
        type_names.push(e.name().to_string());
    }
    for r in graph.all_resources() {
        type_names.push(r.name().to_string());
    }
    type_names.sort();
    type_names.dedup();
    // serde_json::Map is BTreeMap-backed → sorted key emission.
    let mut entity_types: Map<String, Value> = Map::new();
    for t in &type_names {
        entity_types.insert(t.clone(), json!({}));
    }

    // actions: one `Issue<Resource>` per resource, merging principals + targets
    // across all flows issuing it.
    let mut by_resource: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for f in graph.all_flows() {
        let (Some(from), Some(to), Some(resource)) = (
            entity_name.get(&f.from_id().to_string()),
            entity_name.get(&f.to_id().to_string()),
            resource_name.get(&f.resource_id().to_string()),
        ) else {
            continue;
        };
        let entry = by_resource
            .entry(resource.clone())
            .or_insert_with(|| (BTreeSet::new(), BTreeSet::new()));
        entry.0.insert(from.clone());
        entry.1.insert(to.clone());
    }
    let mut actions: Map<String, Value> = Map::new();
    for (resource, (principals, targets)) in &by_resource {
        let mut principal_types: Vec<Value> = principals.iter().map(|p| json!(p)).collect();
        principal_types.sort_by_key(|v| v.as_str().unwrap_or("").to_string());
        let mut annotations: Map<String, Value> = Map::new();
        annotations.insert(
            "to".to_string(),
            json!(targets.iter().cloned().collect::<Vec<_>>().join(",")),
        );
        actions.insert(
            format!("Issue{resource}"),
            json!({
                "appliesTo": {
                    "principalTypes": principal_types,
                    "resourceTypes": [ resource ]
                },
                "annotations": annotations,
            }),
        );
    }

    let mut ns_map: Map<String, Value> = Map::new();
    ns_map.insert("entityTypes".to_string(), Value::Object(entity_types));
    ns_map.insert("actions".to_string(), Value::Object(actions));

    let mut root: Map<String, Value> = Map::new();
    root.insert(ns.to_string(), Value::Object(ns_map));

    format!(
        "// Cedar schema projected by DomainForge. JSON schema format\n\
         // (see https://docs.cedarpolicy.com/schema/json-schema.html).\n{}\n",
        serde_json::to_string_pretty(&Value::Object(root)).expect("schema serializes")
    )
}

/// Build `policies.cedar`: one `permit` per action (baseline authority grant).
fn build_policies(graph: &Graph, ns: &str, model_ref: &str, created_at: &str) -> String {
    let resource_name: BTreeMap<String, String> = graph
        .all_resources()
        .iter()
        .map(|r| (r.id().to_string(), r.name().to_string()))
        .collect();
    let mut resources: Vec<String> = graph
        .all_flows()
        .iter()
        .filter_map(|f| resource_name.get(&f.resource_id().to_string()).cloned())
        .collect();
    resources.sort();
    resources.dedup();

    let mut s = String::new();
    s.push_str(&format!(
        "// Cedar policies projected by DomainForge from {model_ref} at {created_at}.\n\
         // One baseline `permit` per Action: the authority grant that the flow\n\
         // exists in the `{ns}` architecture. Tighten with `when`/`unless`\n\
         // clauses derived from SEA `policy` expressions as needed.\n\n"
    ));
    for resource in &resources {
        s.push_str(&format!(
            "permit (\n  principal,\n  action == {ns}::Action::\"Issue{resource}\",\n  resource\n);\n\n"
        ));
    }
    s
}

/// Derive a single namespace: first entity's namespace (sorted by name), else
/// first resource's, else `"default"`.
fn model_namespace(graph: &Graph) -> String {
    let mut ents = graph.all_entities();
    ents.sort_by_key(|e| e.name().to_string());
    if let Some(e) = ents.first() {
        return e.namespace().to_string();
    }
    let mut res = graph.all_resources();
    res.sort_by_key(|r| r.name().to_string());
    if let Some(r) = res.first() {
        return r.namespace().to_string();
    }
    "default".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_to_graph;

    const FIXED_TS: &str = "2026-07-02T00:00:00+00:00";

    const SOURCE: &str = r#"
@namespace "procurement"

Entity "Buyer" in procurement
Entity "Supplier" in procurement
Resource "PurchaseOrder" units in procurement

Flow "PurchaseOrder" from "Buyer" to "Supplier" quantity 1
"#;

    fn project(source: &str) -> BTreeMap<String, String> {
        let graph = parse_to_graph(source).expect("fixture parses");
        project_cedar_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    fn schema_doc(files: &BTreeMap<String, String>) -> Value {
        let body = &files[SCHEMA_FILE];
        let lines: Vec<&str> = body.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.trim_start().starts_with('{'))
            .unwrap();
        serde_json::from_str(&lines[start..].join("\n")).expect("schema parses as JSON")
    }

    #[test]
    fn emits_two_artifacts() {
        let files = project(SOURCE);
        assert_eq!(
            files.keys().collect::<Vec<_>>(),
            vec!["policies.cedar", "schema.cedarschema.json"]
        );
    }

    #[test]
    fn schema_declares_entity_types_and_one_action_per_resource() {
        let files = project(SOURCE);
        let d = schema_doc(&files);
        let ns_obj = d["procurement"].as_object().expect("namespace present");
        let entity_types = ns_obj["entityTypes"]
            .as_object()
            .expect("entityTypes object");
        assert!(entity_types.contains_key("Buyer"));
        assert!(entity_types.contains_key("Supplier"));
        assert!(entity_types.contains_key("PurchaseOrder"));
        let actions = ns_obj["actions"].as_object().expect("actions object");
        let action = &actions["IssuePurchaseOrder"];
        assert_eq!(action["appliesTo"]["principalTypes"][0], "Buyer");
        assert_eq!(action["appliesTo"]["resourceTypes"][0], "PurchaseOrder");
        assert_eq!(action["annotations"]["to"], "Supplier");
    }

    #[test]
    fn policies_has_one_permit_per_action() {
        let files = project(SOURCE);
        let p = &files[POLICIES_FILE];
        // Count actual statements `permit (`, not the word "permit" in the header.
        assert_eq!(p.matches("permit (").count(), 1);
        assert!(p.contains("action == procurement::Action::\"IssuePurchaseOrder\""));
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_empty_types_and_actions() {
        let files = project("@namespace \"empty\"\n");
        let d = schema_doc(&files);
        let ns_obj = d["default"].as_object().expect("default namespace");
        assert!(ns_obj["entityTypes"].as_object().unwrap().is_empty());
        assert!(ns_obj["actions"].as_object().unwrap().is_empty());
        let p = &files[POLICIES_FILE];
        assert_eq!(p.matches("permit (").count(), 0);
    }
}

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
//! # Authority scope (H6)
//! The `policies.cedar` file is a **permissive baseline**: one `permit` per
//! Action, scoped to the flow's source entity type and resource type
//! (`principal is <From>, resource is <Resource>`). It does not project SEA
//! `policy` expressions as Cedar `forbid`/`when` clauses — that is future
//! work. A Cedar engine loaded with this baseline authorizes the model's
//! declared flows, not its full obligation set; tighten with policy-derived
//! clauses as needed. The module doc and `docs/projection-families.md` say so.
//!
//! Output is two files: `schema.cedarschema.json` (entity types + actions,
//! sorted for byte-identical runs — strict JSON, no comment headers) and
//! `policies.cedar` (one scoped `permit` per action).
//!
//! [Cedar]: https://docs.cedarpolicy.com/schema/json-schema.html

use crate::graph::Graph;
use crate::projection::flows::{collect_flows, model_namespace};
use crate::projection::ids::{ident, NameRegistrar};
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
    let ns = model_namespace(graph)?;

    let schema = build_schema(graph, &ns)?;
    let policies = build_policies(graph, &ns, model_ref, &created_at)?;

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
///
/// H5: the schema is **strict JSON** (no `//` comment header) — Cedar's JSON
/// schema parser rejects comments. Provenance lives in `policies.cedar`
/// comments instead.
fn build_schema(graph: &Graph, ns: &str) -> Result<String, String> {
    let mut reg = NameRegistrar::new();
    let ns_ident = reg.register("ident", ns);

    // entityTypes: union of entities + resources (both are Cedar entity types),
    // sanitized to valid Cedar identifiers (M2: hostile names).
    let mut type_names: Vec<String> = Vec::new();
    for e in graph.all_entities() {
        type_names.push(reg.register("ident", e.name()));
    }
    for r in graph.all_resources() {
        type_names.push(reg.register("ident", r.name()));
    }
    type_names.sort();
    type_names.dedup();
    // serde_json::Map is BTreeMap-backed → sorted key emission.
    let mut entity_types: Map<String, Value> = Map::new();
    for t in &type_names {
        entity_types.insert(t.clone(), json!({}));
    }

    let flows = collect_flows(graph)?;

    // actions: one `Issue<Resource>` per resource, merging principals + targets
    // across all flows issuing it. Resource names are sanitized (M2).
    let mut by_resource: BTreeMap<String, (BTreeSet<String>, BTreeSet<String>)> = BTreeMap::new();
    for f in &flows {
        let resource_id = reg.register("ident", &f.resource);
        let from_id = reg.register("ident", &f.from);
        let to_id = reg.register("ident", &f.to);
        let entry = by_resource
            .entry(resource_id)
            .or_insert_with(|| (BTreeSet::new(), BTreeSet::new()));
        entry.0.insert(from_id);
        entry.1.insert(to_id);
    }
    let mut actions: Map<String, Value> = Map::new();
    for (resource, (principals, targets)) in &by_resource {
        // principals is a BTreeSet → already sorted; L7: no redundant re-sort.
        let principal_types: Vec<Value> = principals.iter().map(|p| json!(p)).collect();
        let mut annotations: Map<String, Value> = Map::new();
        annotations.insert(
            "to".to_string(),
            json!(targets.iter().cloned().collect::<Vec<_>>().join(",")),
        );
        let action_name = format!("Issue{resource}");
        actions.insert(
            action_name,
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
    root.insert(ns_ident, Value::Object(ns_map));

    // H5: strict JSON — NO comment header. The schema serializes directly.
    Ok(serde_json::to_string_pretty(&Value::Object(root)).expect("schema serializes"))
}

/// Build `policies.cedar`: one scoped `permit` per action (H6: baseline
/// authority grant scoped to the flow's source entity type + resource type).
fn build_policies(
    graph: &Graph,
    ns: &str,
    model_ref: &str,
    created_at: &str,
) -> Result<String, String> {
    let mut reg = NameRegistrar::new();
    let ns_ident = reg.register("ident", ns);

    // Collect (resource, from) pairs from flows — each becomes a scoped permit.
    // The flow resolver (M4) errors on dangling references.
    let flows = collect_flows(graph)?;
    let mut pairs: Vec<(String, String)> = flows
        .iter()
        .map(|f| {
            (
                reg.register("ident", &f.resource),
                reg.register("ident", &f.from),
            )
        })
        .collect();
    pairs.sort();
    pairs.dedup();

    let mut s = String::new();
    // Provenance + scope note live here (Cedar policy files allow `//` comments).
    s.push_str(&format!(
        "// Cedar policies projected by DomainForge from {model_ref} at {created_at}.\n\
         // PERMISSIVE BASELINE: one scoped `permit` per Action — the authority grant\n\
         // that the flow exists in the `{ns_ident}` architecture. Each permit is scoped\n\
         // to the flow's source entity type (`principal is <From>`) and resource type\n\
         // (`resource is <Resource>`). This does NOT project SEA `policy` expressions;\n\
         // tighten with `when`/`unless` clauses derived from SEA `policy` obligations\n\
         // (e.g. require_approval) as needed.\n\n"
    ));
    for (resource, from) in &pairs {
        // H6 fix: scope each permit to the flow's principal + resource types,
        // not an unconstrained `principal, resource`.
        s.push_str(&format!(
            "permit (\n  principal is {ns_ident}::{from},\n  action == {ns_ident}::Action::\"Issue{resource}\",\n  resource is {ns_ident}::{resource}\n);\n\n"
        ));
    }
    Ok(s)
}

/// Re-export the kernel ident sanitizer for tests + downstream.
pub fn cedar_ident(name: &str) -> String {
    ident(name)
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
        // H5: schema is strict JSON now — parse directly, no comment stripping.
        serde_json::from_str(body).expect("schema parses as JSON")
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
    fn schema_is_strict_json_no_comment_header() {
        let files = project(SOURCE);
        let body = &files[SCHEMA_FILE];
        // H5: the first character must be `{` (strict JSON), not a `//` comment.
        let trimmed = body.trim_start();
        assert!(
            trimmed.starts_with('{'),
            "schema must be strict JSON (no comment header); first chars: {:?}",
            &body[..body.len().min(40)]
        );
        // The whole file must parse as JSON without comment-stripping.
        let _: Value = serde_json::from_str(body).expect("strict JSON parse");
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
    fn policies_has_one_scoped_permit_per_action() {
        let files = project(SOURCE);
        let p = &files[POLICIES_FILE];
        assert_eq!(p.matches("permit (").count(), 1);
        // H6: the permit must scope principal and resource by type.
        assert!(
            p.contains("principal is procurement::Buyer"),
            "permit must scope principal to source entity type (H6): {p}"
        );
        assert!(
            p.contains("resource is procurement::PurchaseOrder"),
            "permit must scope resource to resource type (H6): {p}"
        );
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

    /// M2 regression: hostile names must not corrupt the Cedar schema or
    /// policy file. An entity named `a"b` must produce a sanitized type name
    /// (no raw quote) and must not break the Action EID string.
    #[test]
    fn hostile_names_do_not_corrupt_output() {
        let hostile = r#"
@namespace "h"
Entity "a\"b" in h
Resource "R x" units in h
Flow "R x" from "a\"b" to "a\"b" quantity 1
"#;
        let files = project(hostile);
        // Schema must still parse as strict JSON.
        let _doc = schema_doc(&files);
        // Policy file must not contain a raw quote escaping issue.
        let p = &files[POLICIES_FILE];
        assert!(p.contains("permit ("), "must have a permit");
        // No unescaped raw `"` from the entity name inside identifier positions.
        // The sanitized entity id replaces non-alnum with _.
        assert!(
            p.contains("principal is h::a_b"),
            "hostile entity name must be sanitized: {p}"
        );
    }
}

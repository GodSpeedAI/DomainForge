//! Shared flow resolution + namespace derivation for projection families.
//!
//! Every projection family that consumes SEA flows needs to resolve flow
//! endpoint/resource ids to display names. This module provides one blessed
//! helper so the resolution policy — including the dangling-reference policy
//! (M4) — is uniform across all eight targets. ADR-011 §2 requires this: no
//! family re-implements flow resolution.
//!
//! # Dangling-reference policy
//! The graph should never contain a flow referencing an unknown entity or
//! resource, but if one does, [`collect_flows`] returns an **error** (loud)
//! rather than silently dropping the flow. CloudEvents always errored this
//! way; the other seven targets previously `continue`-d, emitting a smaller
//! architecture than the model declares. Loud is correct — a partially
//! projected authority/verification artifact must not look complete.

use crate::graph::Graph;
use std::collections::BTreeMap;

/// A flow with display names resolved for its resource and endpoints.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedFlow {
    pub resource: String,
    pub from: String,
    pub to: String,
    pub quantity: String,
    pub namespace: String,
    /// Flow annotations (e.g. `@cqrs { "kind": "command" }`) copied from the
    /// graph flow's attribute map. `BTreeMap` gives deterministic ordering.
    pub annotations: BTreeMap<String, serde_json::Value>,
}

/// F3: structured errors for the two failure modes this module can produce.
/// Previously both were a bare `String`, so every one of the eight callers
/// could only match on substring content if it wanted to distinguish them.
/// `From<FlowError> for String` is implemented below so every existing
/// `collect_flows(graph)?` / `model_namespace(graph)?` call site (all
/// currently inside functions returning `Result<_, String>`) keeps
/// compiling unchanged.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FlowError {
    /// A flow references an entity or resource id that doesn't exist in the
    /// graph (M4's loud dangling-reference policy).
    DanglingReference { flow_id: String },
    /// The graph declares more than one namespace across its entities and
    /// resources (M5).
    MultiNamespace { namespaces: Vec<String> },
}

impl std::fmt::Display for FlowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DanglingReference { flow_id } => write!(
                f,
                "flow {flow_id} references an unknown entity or resource (dangling reference)"
            ),
            Self::MultiNamespace { namespaces } => write!(
                f,
                "model spans multiple namespaces ({}); the projection targets require a single \
                 namespace. Namespaces found: {}",
                namespaces.len(),
                namespaces.join(", ")
            ),
        }
    }
}

impl std::error::Error for FlowError {}

impl From<FlowError> for String {
    fn from(e: FlowError) -> String {
        e.to_string()
    }
}

/// Resolve every flow in the graph to display names, sorted by
/// (resource, from, to) for deterministic output. Returns an error if any
/// flow references an unknown entity or resource (loud dangling-reference
/// policy — M4).
pub fn collect_flows(graph: &Graph) -> Result<Vec<ResolvedFlow>, FlowError> {
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

    let mut flows: Vec<ResolvedFlow> = Vec::new();
    for f in graph.all_flows() {
        let (Some(from), Some(to), Some(resource)) = (
            entity_name.get(&f.from_id().to_string()),
            entity_name.get(&f.to_id().to_string()),
            resource_name.get(&f.resource_id().to_string()),
        ) else {
            return Err(FlowError::DanglingReference {
                flow_id: f.id().to_string(),
            });
        };
        flows.push(ResolvedFlow {
            resource: resource.clone(),
            from: from.clone(),
            to: to.clone(),
            quantity: f.quantity().to_string(),
            namespace: f.namespace().to_string(),
            annotations: f
                .attributes()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        });
    }
    flows.sort_by(|a, b| {
        a.resource
            .cmp(&b.resource)
            .then(a.from.cmp(&b.from))
            .then(a.to.cmp(&b.to))
            .then(a.quantity.cmp(&b.quantity))
    });
    Ok(flows)
}

/// Derive the model's namespace. Returns an error if the model spans more
/// than one namespace (multi-namespace models previously collapsed silently
/// to the alphabetically-first entity's namespace — M5). The namespace is
/// derived from all entities and resources; if none declare a namespace,
/// `"default"` is returned.
pub fn model_namespace(graph: &Graph) -> Result<String, FlowError> {
    let mut namespaces: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for e in graph.all_entities() {
        namespaces.insert(e.namespace().to_string());
    }
    for r in graph.all_resources() {
        namespaces.insert(r.namespace().to_string());
    }
    match namespaces.len() {
        0 => Ok("default".to_string()),
        1 => Ok(namespaces.iter().next().unwrap().clone()),
        _ => Err(FlowError::MultiNamespace {
            namespaces: namespaces.into_iter().collect(),
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_to_graph;

    #[test]
    fn collect_flows_resolves_and_sorts() {
        let src = r#"
@namespace "proc"
Entity "A" in proc
Entity "B" in proc
Resource "R" units in proc
Flow "R" from "B" to "A" quantity 2
Flow "R" from "A" to "B" quantity 1
"#;
        let g = parse_to_graph(src).unwrap();
        let flows = collect_flows(&g).unwrap();
        assert_eq!(flows.len(), 2);
        // Sorted by (resource, from, to): A→B before B→A.
        assert_eq!(flows[0].from, "A");
        assert_eq!(flows[1].from, "B");
    }

    #[test]
    fn collect_flows_preserves_flow_annotations() {
        // `@cqrs { "kind": "command" }` on a flow must survive from AST into
        // the graph and out through `ResolvedFlow::annotations` (Task 1).
        let src = r#"
@namespace "proc"
Entity "A" in proc
Entity "B" in proc
Resource "R" units in proc
Flow "R" @cqrs { "kind": "command" } from "A" to "B" quantity 1
"#;
        let g = parse_to_graph(src).unwrap();
        let flows = collect_flows(&g).unwrap();
        assert_eq!(flows.len(), 1);
        assert_eq!(flows[0].annotations["cqrs"]["kind"], "command");
    }

    #[test]
    fn collect_flows_errors_on_dangling_reference() {
        // A flow referencing a non-existent resource is a graph integrity
        // error — we surface it loudly instead of silently dropping (M4).
        // `Graph::add_flow` refuses to construct one (and `remove_entity`/
        // `remove_resource` refuse to leave one dangling), so the only way
        // to produce this state is to bypass those constructors: `Graph`
        // derives `Deserialize` directly on its raw field set, so a
        // hand-edited JSON round-trip can build the otherwise-unreachable
        // state this function must still handle loudly rather than
        // silently (F1 — the previous version of this test asserted the
        // empty case instead of the actual dangling-reference path).
        let src = r#"
@namespace "proc"
Entity "A" in proc
Entity "B" in proc
Resource "R" units in proc
Flow "R" from "A" to "B" quantity 1
"#;
        let g = parse_to_graph(src).unwrap();
        let mut value = serde_json::to_value(&g).unwrap();
        let flow = value["flows"]
            .as_object_mut()
            .unwrap()
            .values_mut()
            .next()
            .unwrap();
        flow["resource_id"] = serde_json::json!(uuid::Uuid::new_v4().to_string());
        let dangling: Graph = serde_json::from_value(value).unwrap();

        let err = collect_flows(&dangling).unwrap_err();
        assert!(matches!(err, FlowError::DanglingReference { .. }));
        assert!(err.to_string().contains("dangling reference"), "{err}");
    }

    #[test]
    fn model_namespace_single() {
        let src = r#"
@namespace "procurement"
Entity "Buyer" in procurement
Resource "PO" units in procurement
"#;
        let g = parse_to_graph(src).unwrap();
        assert_eq!(model_namespace(&g).unwrap(), "procurement");
    }

    #[test]
    fn model_namespace_errors_on_multiple() {
        let src = r#"
@namespace "a"
Entity "A" in a
Entity "B" in b
"#;
        let g = parse_to_graph(src).unwrap();
        let err = model_namespace(&g).unwrap_err();
        assert!(matches!(err, FlowError::MultiNamespace { .. }));
        assert!(err.to_string().contains("multiple namespaces"), "{err}");
    }

    #[test]
    fn model_namespace_default_when_empty() {
        let g = parse_to_graph("@namespace \"empty\"\n").unwrap();
        assert_eq!(model_namespace(&g).unwrap(), "default");
    }
}

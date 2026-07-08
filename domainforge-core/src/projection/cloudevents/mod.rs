//! CloudEvents 1.0 projection: each SEA `Flow` — a resource moving from one
//! entity to another — becomes a [CloudEvents](https://cloudevents.io) 1.0
//! envelope, so an architecture model can be consumed directly by event-driven
//! runtimes (brokers, sinks, async frameworks).
//!
//! Output is a single `events.jsonl`: one CloudEvent per line, sorted by its
//! deterministic `id`, so the projection is byte-identical run-to-run for a fixed
//! `created_at`. Every `id` is minted through [`crate::projection::ids`] from the
//! flow's stable content (source entity, resource, target entity, quantity), so
//! random per-parse flow UUIDs never reach the output.

use crate::graph::Graph;
use crate::projection::ids::element_id;
use crate::projection::sink::ArtifactSink;
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The single emitted artifact's relative path.
pub const OUTPUT_FILE: &str = "events.jsonl";

/// CloudEvents spec version targeted by this projection.
pub const SPEC_VERSION: &str = "1.0";

/// Emit the CloudEvents stream into `sink`; returns the emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let events = build_events(graph, &created_at)?;

    let mut body = String::new();
    body.push_str(&format!(
        "# CloudEvents 1.0 stream projected by DomainForge from {model_ref} at {created_at}.\n"
    ));
    for ev in &events {
        body.push_str(
            &serde_json::to_string(ev)
                .map_err(|e| format!("failed to serialize CloudEvent: {e}"))?,
        );
        body.push('\n');
    }
    sink.write(OUTPUT_FILE, &body)?;
    Ok(vec![OUTPUT_FILE.to_string()])
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_cloudevents_in_memory(
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

/// Build the CloudEvent envelopes, sorted by deterministic id.
fn build_events(graph: &Graph, created_at: &str) -> Result<Vec<Value>, String> {
    // Resolve entity/resource display names by id.
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

    let mut events: Vec<(String, Value)> = Vec::new();
    for f in graph.all_flows() {
        let (Some(from), Some(to), Some(resource)) = (
            entity_name.get(&f.from_id().to_string()),
            entity_name.get(&f.to_id().to_string()),
            resource_name.get(&f.resource_id().to_string()),
        ) else {
            return Err(format!(
                "flow {} references an unknown entity or resource",
                f.id()
            ));
        };
        let ns = f.namespace();
        let qty = f.quantity().to_string();
        let id = element_id(
            "cloudevents",
            &[from.as_str(), resource.as_str(), to.as_str(), qty.as_str()],
        );
        let source = format!("/{}/{}", slug(ns), slug(from));
        let etype = format!("{}.{}.issued", slug(ns), slug(resource));
        let ev = json!({
            "specversion": SPEC_VERSION,
            "id": id.clone(),
            "source": source,
            "type": etype,
            "subject": to.as_str(),
            "time": created_at,
            "datacontenttype": "application/json",
            "data": {
                "resource": resource.as_str(),
                "quantity": qty,
                "from": from.as_str(),
                "to": to.as_str(),
            }
        });
        events.push((id, ev));
    }
    events.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(events.into_iter().map(|(_, v)| v).collect())
}

/// Lowercase ASCII slug for URI path segments / event-type fragments: keep
/// alphanumerics, replace everything else with `_`. Empty input yields `_`.
fn slug(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
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
        project_cloudevents_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    #[test]
    fn emits_single_jsonl_artifact() {
        let files = project(SOURCE);
        assert_eq!(files.keys().collect::<Vec<_>>(), vec!["events.jsonl"]);
    }

    #[test]
    fn each_line_is_a_valid_cloudevent() {
        let files = project(SOURCE);
        let lines: Vec<&str> = files["events.jsonl"]
            .lines()
            .filter(|l| !l.starts_with('#'))
            .collect();
        assert_eq!(lines.len(), 1, "one flow -> one event");
        let ev: Value = serde_json::from_str(lines[0]).expect("event parses as JSON");
        assert_eq!(ev["specversion"].as_str(), Some("1.0"));
        assert_eq!(
            ev["type"].as_str(),
            Some("procurement.purchaseorder.issued")
        );
        assert_eq!(ev["source"].as_str(), Some("/procurement/buyer"));
        assert_eq!(ev["subject"].as_str(), Some("Supplier"));
        assert_eq!(ev["time"].as_str(), Some(FIXED_TS));
        assert_eq!(ev["data"]["resource"].as_str(), Some("PurchaseOrder"));
        assert_eq!(ev["data"]["quantity"].as_str(), Some("1"));
        assert_eq!(ev["id"].as_str().unwrap().len(), 16);
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_header_only() {
        let files = project("@namespace \"empty\"\n");
        let body = &files["events.jsonl"];
        assert_eq!(body.lines().filter(|l| !l.starts_with('#')).count(), 0);
    }
}

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
use crate::projection::flows::{collect_flows, model_namespace};
use crate::projection::ids::{element_id, slug};
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
    _model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let events = build_events(graph, &created_at)?;

    let mut body = String::new();
    // H5: events.jsonl is JSON Lines — every line MUST be valid JSON. No `#`
    // comment header. Provenance is carried in a sibling README or via the
    // CloudEvents extension attribute below; it must not be line 1.
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
    let ns = model_namespace(graph)?;
    let flows = collect_flows(graph)?; // M4: loud dangling-ref policy

    let mut events: Vec<(String, Value)> = Vec::new();
    for f in &flows {
        let qty = &f.quantity;
        let id = element_id(
            "cloudevents",
            &[
                f.from.as_str(),
                f.resource.as_str(),
                f.to.as_str(),
                qty.as_str(),
            ],
        );
        let source = format!("/{}/{}", slug(&ns), slug(&f.from));
        let etype = format!("{}.{}.issued", slug(&ns), slug(&f.resource));
        let ev = json!({
            "specversion": SPEC_VERSION,
            "id": id.clone(),
            "source": source,
            "type": etype,
            "subject": f.to.as_str(),
            "time": created_at,
            "datacontenttype": "application/json",
            // Provenance extension (H5: replaces the old comment header).
            "domainforgemodelref": "DomainForge",
            "data": {
                "resource": f.resource.as_str(),
                "quantity": qty,
                "from": f.from.as_str(),
                "to": f.to.as_str(),
            }
        });
        events.push((id, ev));
    }
    events.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(events.into_iter().map(|(_, v)| v).collect())
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
        // H5: every line must be valid JSON — no comment lines to skip.
        let lines: Vec<&str> = files["events.jsonl"]
            .lines()
            .filter(|l| !l.is_empty())
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
    fn empty_model_emits_empty_file() {
        let files = project("@namespace \"empty\"\n");
        let body = &files["events.jsonl"];
        // H5: no comment header; empty model = empty (or whitespace-only) file.
        assert_eq!(body.lines().filter(|l| !l.is_empty()).count(), 0);
    }

    /// H5 + M2 regression: events.jsonl with a name containing spaces — the
    /// event `type` and `source` use slug'd fragments, and the whole file
    /// must be strict JSONL (no comment header).
    #[test]
    fn hostile_names_produce_valid_strict_jsonl() {
        let hostile = r#"
@namespace "h"
Entity "Order Line" in h
Entity "Supplier" in h
Resource "R" units in h
Flow "R" from "Order Line" to "Supplier" quantity 1
"#;
        let files = project(hostile);
        let body = &files["events.jsonl"];
        // Every line must parse as JSON (H5: strict JSONL).
        for line in body.lines().filter(|l| !l.is_empty()) {
            let ev: Value = serde_json::from_str(line).expect("strict JSONL line parses");
            // M2: source/type slug the namespace + entity name (no raw spaces).
            assert!(
                !ev["source"].as_str().unwrap().contains(' '),
                "source must not contain raw spaces (M2)"
            );
            assert!(
                !ev["type"].as_str().unwrap().contains(' '),
                "type must not contain raw spaces (M2)"
            );
        }
    }
}

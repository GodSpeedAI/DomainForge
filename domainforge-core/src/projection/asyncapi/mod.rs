//! AsyncAPI 2.6 projection: each SEA `Flow` becomes an AsyncAPI channel and
//! message, so an architecture model doubles as the contract for an
//! event-driven API. This is the natural companion to the CloudEvents
//! projection — AsyncAPI describes the channels/messages, CloudEvents defines
//! the per-message envelope.
//!
//! Output is a single `asyncapi.json` (an AsyncAPI document is valid as JSON or
//! YAML; JSON keeps this projection dependency-free). Channels are keyed by
//! `<ns>/<resource>/<from>/issued` and serialized through `serde_json`'s
//! sorted `Map`, so output is byte-identical run-to-run for a fixed model.

use crate::graph::Graph;
use crate::projection::sink::ArtifactSink;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// The single emitted artifact's relative path.
pub const OUTPUT_FILE: &str = "asyncapi.json";

/// AsyncAPI document version targeted by this projection.
pub const ASYNCAPI_VERSION: &str = "2.6.0";

/// Emit the AsyncAPI document into `sink`; returns the emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let spec = build_spec(graph, model_ref, &created_at);
    let mut body = format!(
        "// AsyncAPI {ASYNCAPI_VERSION} document projected by DomainForge from {model_ref} at {created_at}.\n"
    );
    body.push_str(
        &serde_json::to_string_pretty(&spec)
            .map_err(|e| format!("failed to serialize AsyncAPI document: {e}"))?,
    );
    body.push('\n');
    sink.write(OUTPUT_FILE, &body)?;
    Ok(vec![OUTPUT_FILE.to_string()])
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_asyncapi_in_memory(
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

/// Build the AsyncAPI document value.
fn build_spec(graph: &Graph, model_ref: &str, created_at: &str) -> Value {
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

    // serde_json::Map is BTreeMap-backed (no `preserve_order` feature), so
    // channel keys are emitted in sorted order regardless of flow iteration.
    let mut channels: Map<String, Value> = Map::new();
    for f in graph.all_flows() {
        let (Some(from), Some(to), Some(resource)) = (
            entity_name.get(&f.from_id().to_string()),
            entity_name.get(&f.to_id().to_string()),
            resource_name.get(&f.resource_id().to_string()),
        ) else {
            continue;
        };
        let ns = f.namespace();
        let channel_key = format!("{}/{}/{}/issued", slug(ns), slug(resource), slug(from));
        let channel = json!({
            "description": format!("{resource} flow from {from} to {to}"),
            "publish": {
                "message": {
                    "contentType": "application/json",
                    "name": format!("{resource}Issued"),
                    "title": format!("{resource} issued by {from}"),
                    "payload": {
                        "type": "object",
                        "properties": {
                            "resource": {"type": "string"},
                            "quantity": {"type": "string"},
                            "from": {"type": "string"},
                            "to": {"type": "string"}
                        },
                        "required": ["resource", "from", "to"]
                    }
                }
            }
        });
        channels.insert(channel_key, channel);
    }

    json!({
        "asyncapi": ASYNCAPI_VERSION,
        "info": {
            "title": model_ref,
            "version": "1.0.0",
            "x-generated-by": "DomainForge",
            "x-created-at": created_at
        },
        "channels": channels
    })
}

/// Lowercase ASCII slug for channel path segments: keep alphanumerics, replace
/// everything else with `_`. Empty input yields `_`.
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
        project_asyncapi_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    #[test]
    fn emits_single_asyncapi_artifact() {
        let files = project(SOURCE);
        assert_eq!(files.keys().collect::<Vec<_>>(), vec!["asyncapi.json"]);
    }

    #[test]
    fn document_is_a_valid_asyncapi_with_one_channel() {
        let files = project(SOURCE);
        let body = &files["asyncapi.json"];
        let lines: Vec<&str> = body.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.trim_start().starts_with('{'))
            .unwrap();
        let doc: Value = serde_json::from_str(&lines[start..].join("\n")).expect("parses as JSON");
        assert_eq!(doc["asyncapi"], "2.6.0");
        assert_eq!(doc["info"]["version"], "1.0.0");
        let channels = doc["channels"].as_object().expect("channels is an object");
        assert_eq!(channels.len(), 1, "one flow -> one channel");
        let key = channels.keys().next().unwrap();
        assert!(key.ends_with("/purchaseorder/buyer/issued"));
        let msg = &doc["channels"][key.as_str()]["publish"]["message"];
        assert_eq!(msg["contentType"], "application/json");
        assert_eq!(msg["payload"]["properties"]["resource"]["type"], "string");
        assert!(msg["payload"]["required"].as_array().is_some());
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_empty_channels() {
        let files = project("@namespace \"empty\"\n");
        let body = &files["asyncapi.json"];
        let lines: Vec<&str> = body.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.trim_start().starts_with('{'))
            .unwrap();
        let doc: Value = serde_json::from_str(&lines[start..].join("\n")).expect("parses");
        assert!(doc["channels"].as_object().unwrap().is_empty());
        assert_eq!(doc["asyncapi"], "2.6.0");
    }
}

//! AsyncAPI 3.0 projection: a SEA model becomes an [AsyncAPI] 3.0 document
//! (`asyncapi.yaml`) describing the event-driven API contract — channels,
//! operations (producers = `send`, consumers = `receive`), messages, and
//! payload schemas — so an architecture model doubles as the contract a
//! broker, client, and documentation generator all consume.
//!
//! This is the natural companion to the CloudEvents projection: AsyncAPI
//! standardizes the *channels/messages/operations* contract (who publishes
//! where, who subscribes); CloudEvents standardizes the per-message
//! *envelope*. Neither is a runtime broker — both are interoperability
//! contracts. See `docs/cloudevents-asyncapi-projections.md`.
//!
//! # Concept mapping (SEA → AsyncAPI)
//! | SEA concept | AsyncAPI role | Note |
//! |---|---|---|
//! | `Entity` | producer / consumer (operation `send`/`receive`) | "Actor" in the spec maps to Entity. |
//! | `Flow` | one event → one channel + send/receive operations | "Event" in the spec maps to Flow. |
//! | `Resource` | message payload schema | "Capability"/"Evidence" have no SEA primitive; where they would appear they are represented by `Resource`. |
//! | `Policy` | (documented, not structural) | policies don't map to AsyncAPI structures; recorded in `info.description`. |
//!
//! Output is a single `asyncapi.yaml`. Keys are serialized through sorted
//! maps, so output is byte-identical run-to-run for a fixed model. The
//! emitted document validates against the vendored official AsyncAPI 3.0.0
//! schema (`schemas/asyncapi/3.0.0.json`); see
//! `tests/asyncapi_spec_validation_tests.rs`.
//!
//! [AsyncAPI]: https://www.asyncapi.com/

use crate::graph::Graph;
use crate::projection::flows::{collect_flows, model_namespace};
use crate::projection::ids::{slug, NameRegistrar};
use crate::projection::sink::ArtifactSink;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// The single emitted artifact's relative path.
pub const OUTPUT_FILE: &str = "asyncapi.yaml";

/// AsyncAPI document version targeted by this projection.
pub const ASYNCAPI_VERSION: &str = "3.0.0";

/// Emit the AsyncAPI document into `sink`; returns the emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let spec = build_spec(graph, model_ref, &created_at)?;
    let yaml = serde_yaml::to_string(&spec)
        .map_err(|e| format!("failed to serialize AsyncAPI document: {e}"))?;
    // YAML has no leading-comment affordance in serde_yaml; prepend a header.
    let body = format!(
        "# AsyncAPI {ASYNCAPI_VERSION} document projected by DomainForge from {model_ref} at {created_at}.\n\
         # Concept mapping: Entity -> producer/consumer, Flow -> channel+event, Resource -> payload schema.\n{yaml}"
    );
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

/// Build the AsyncAPI 3.0 document value.
fn build_spec(graph: &Graph, model_ref: &str, created_at: &str) -> Result<Value, String> {
    let ns = model_namespace(graph)?; // M5: errors on multi-namespace
    let flows = collect_flows(graph)?; // M4: loud dangling-ref policy

    // Collision-safe key registrar (M1): channel/operation/message/schema keys
    // that collide after case-folding get a hash suffix instead of silently
    // overwriting each other in Map::insert.
    let mut reg = NameRegistrar::new();

    // Group flows by (resource, from) -> set of tos. Each group = one channel.
    let mut groups: BTreeMap<(String, String), BTreeMap<String, ()>> = BTreeMap::new();
    for f in &flows {
        groups
            .entry((f.resource.clone(), f.from.clone()))
            .or_default()
            .insert(f.to.clone(), ());
    }

    // serde_json::Map is BTreeMap-backed → channels/operations/components keys
    // emit in sorted order regardless of insertion order.
    let mut channels: Map<String, Value> = Map::new();
    let mut operations: Map<String, Value> = Map::new();
    let mut messages: Map<String, Value> = Map::new();
    let mut schemas: Map<String, Value> = Map::new();

    for ((resource, from), tos) in &groups {
        // M1: register keys through the collision-safe registrar.
        let chan_key = reg.register("slug", &format!("{}{}Issued", lc(resource), pc(from)));
        let msg_key = reg.register("slug", &format!("{chan_key}Message"));
        let schema_key = reg.register("slug", &format!("{}Payload", lc(resource)));
        let address = format!("{}/{}/{}/issued", slug(&ns), slug(resource), slug(from));

        channels.insert(
            chan_key.clone(),
            json!({
                "address": address,
                "messages": { msg_key.clone(): { "$ref": format!("#/components/messages/{msg_key}") } }
            }),
        );

        // Producer operation: `from` sends.
        let send_op = reg.register("slug", &format!("emit{chan_key}"));
        operations.insert(
            send_op,
            json!({
                "action": "send",
                "channel": { "$ref": format!("#/channels/{chan_key}") },
                "title": format!("{from} emits {resource}"),
                "messages": [ { "$ref": format!("#/components/messages/{msg_key}") } ]
            }),
        );
        // Consumer operation(s): each `to` receives.
        for to in tos.keys() {
            let recv_op = reg.register("slug", &format!("receive{chan_key}By{}", pc(to)));
            operations.insert(
                recv_op,
                json!({
                    "action": "receive",
                    "channel": { "$ref": format!("#/channels/{chan_key}") },
                    "title": format!("{to} consumes {resource}"),
                    "messages": [ { "$ref": format!("#/components/messages/{msg_key}") } ]
                }),
            );
        }

        messages.insert(
            msg_key,
            json!({
                "contentType": "application/json",
                "name": format!("{resource}Issued"),
                "title": format!("{resource} issued by {from}"),
                "payload": { "$ref": format!("#/components/schemas/{schema_key}") }
            }),
        );
        schemas.insert(
            schema_key,
            json!({
                "type": "object",
                "properties": {
                    "resource": { "type": "string" },
                    "from": { "type": "string" },
                    "to": { "type": "string" },
                    "quantity": { "type": "string" }
                },
                "required": ["resource", "from", "to"]
            }),
        );
    }

    let policy_summary = summarize_policies(graph);
    let mut description = format!(
        "Projected by DomainForge from {model_ref} at {created_at}. Channels/operations derived from SEA flows."
    );
    if !policy_summary.is_empty() {
        description.push_str(&format!(" Governing policies: {policy_summary}."));
    }

    let mut components: Map<String, Value> = Map::new();
    components.insert("messages".to_string(), Value::Object(messages));
    components.insert("schemas".to_string(), Value::Object(schemas));

    // L7: info.title is the namespace, not the input file path.
    Ok(json!({
        "asyncapi": ASYNCAPI_VERSION,
        "info": {
            "title": ns,
            "version": "1.0.0",
            "description": description
        },
        "defaultContentType": "application/json",
        "channels": channels,
        "operations": operations,
        "components": components
    }))
}

/// One-line summary of policy names for the description (policies don't map
/// structurally to AsyncAPI; recorded as documentation only).
fn summarize_policies(graph: &Graph) -> String {
    let mut names: Vec<String> = graph
        .all_policies()
        .iter()
        .map(|p| p.name.clone())
        .collect();
    names.sort();
    names.join(", ")
}

/// PascalCase for operation/message key suffixes (Buyer -> Buyer).
fn pc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(if up { c.to_ascii_uppercase() } else { c });
            up = false;
        } else {
            up = true;
        }
    }
    if out.is_empty() {
        out.push('_');
    }
    out
}

/// LowerCamelCase for message/schema keys (PurchaseOrder -> purchaseorder).
fn lc(s: &str) -> String {
    slug(s)
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

    fn doc(files: &BTreeMap<String, String>) -> Value {
        let body = &files[OUTPUT_FILE];
        serde_yaml::from_str(body).expect("output parses as YAML")
    }

    #[test]
    fn emits_single_yaml_artifact() {
        let files = project(SOURCE);
        assert_eq!(files.keys().collect::<Vec<_>>(), vec!["asyncapi.yaml"]);
    }

    #[test]
    fn document_targets_asyncapi_3_0() {
        let d = doc(&project(SOURCE));
        assert_eq!(d["asyncapi"], "3.0.0");
        assert_eq!(d["defaultContentType"], "application/json");
        assert_eq!(d["info"]["version"], "1.0.0");
    }

    #[test]
    fn has_channel_message_operation_and_schema_per_flow() {
        let d = doc(&project(SOURCE));
        let channels = d["channels"].as_object().expect("channels object");
        assert_eq!(channels.len(), 1, "one channel per (resource,from)");
        let ops = d["operations"].as_object().expect("operations object");
        // 1 send (producer) + 1 receive (consumer) for the single flow.
        let sends = ops.values().filter(|o| o["action"] == "send").count();
        let recvs = ops.values().filter(|o| o["action"] == "receive").count();
        assert_eq!(sends, 1, "one producer (send) operation");
        assert_eq!(recvs, 1, "one consumer (receive) operation");
        assert!(!d["components"]["messages"].as_object().unwrap().is_empty());
        assert!(!d["components"]["schemas"].as_object().unwrap().is_empty());
    }

    #[test]
    fn operations_reference_channels_and_messages() {
        let d = doc(&project(SOURCE));
        let ops = d["operations"].as_object().unwrap();
        for (_name, op) in ops {
            let chan_ref = op["channel"]["$ref"].as_str().expect("op has channel $ref");
            assert!(
                chan_ref.starts_with("#/channels/"),
                "bad channel ref: {chan_ref}"
            );
            let msg_ref = op["messages"][0]["$ref"]
                .as_str()
                .expect("op has message $ref");
            assert!(
                msg_ref.starts_with("#/components/messages/"),
                "bad message ref: {msg_ref}"
            );
        }
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_empty_channels_and_operations() {
        let d = doc(&project("@namespace \"empty\"\n"));
        assert!(d["channels"].as_object().unwrap().is_empty());
        assert!(d["operations"].as_object().unwrap().is_empty());
        assert_eq!(d["asyncapi"], "3.0.0");
    }
}

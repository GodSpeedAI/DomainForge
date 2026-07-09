//! Devbox activation projection: a SEA model becomes a [`devbox.json`] that
//! pre-loads the domain manifest (namespace, entities, resources, flows) as
//! environment variables and prints a banner on shell entry, so a developer
//! running `devbox shell` lands in a reproducible environment that already
//! knows what architecture it is hosting.
//!
//! `packages` is intentionally empty: the SEA model declares *what* exists in
//! a domain, not the language/tooling needed to build it (that is
//! stack-specific and inventing nix package names from `Buyer`/`Supplier`
//! would make the file fail to install). Developers add the packages their
//! stack needs on top of this manifest.
//!
//! Output is a single `devbox.json` (JSONC; Devbox parses the leading `//`
//! comment). Serialization goes through `serde_json`'s sorted `Map`, so the
//! output is byte-identical run-to-run for a fixed model.
//!
//! [`devbox.json`]: https://www.jetify.com/devbox/docs/configuration/

use crate::graph::Graph;
use crate::projection::sink::ArtifactSink;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// The single emitted artifact's relative path.
pub const OUTPUT_FILE: &str = "devbox.json";

/// Emit the Devbox config into `sink`; returns the emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let spec = build_spec(graph, model_ref, &created_at);
    let mut body = format!(
        "// devbox.json projected by DomainForge from {model_ref} at {created_at}.\n\
         // Activation projection: a reproducible shell pre-loaded with the domain\n\
         // manifest as env vars (namespace, entities, resources, flows) plus a\n\
         // banner on entry. `packages` is intentionally empty — the SEA model\n\
         // declares what exists, not the language/tooling to build it; add the\n\
         // nix packages your stack needs.\n"
    );
    body.push_str(
        &serde_json::to_string_pretty(&spec)
            .map_err(|e| format!("failed to serialize devbox document: {e}"))?,
    );
    body.push('\n');
    sink.write(OUTPUT_FILE, &body)?;
    Ok(vec![OUTPUT_FILE.to_string()])
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_devbox_in_memory(
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

/// Build the devbox.json value.
fn build_spec(graph: &Graph, model_ref: &str, _created_at: &str) -> Value {
    // ponytail: model `@version` is parsed into the AST but not carried onto
    // Graph; surfacing it is a cross-cutting graph change, out of scope here.
    let ns = model_namespace(graph);

    let mut entities: Vec<String> = graph
        .all_entities()
        .iter()
        .map(|e| e.name().to_string())
        .collect();
    entities.sort();
    let mut resources: Vec<String> = graph
        .all_resources()
        .iter()
        .map(|r| r.name().to_string())
        .collect();
    resources.sort();

    // Resolve display names by id for flow endpoints.
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
    let mut flows: Vec<String> = Vec::new();
    for f in graph.all_flows() {
        let (Some(from), Some(to), Some(resource)) = (
            entity_name.get(&f.from_id().to_string()),
            entity_name.get(&f.to_id().to_string()),
            resource_name.get(&f.resource_id().to_string()),
        ) else {
            continue;
        };
        flows.push(format!("{resource}:{from}->{to}"));
    }
    flows.sort();

    // serde_json::Map is BTreeMap-backed (no preserve_order), so env keys are
    // emitted in sorted order regardless of insertion order.
    let mut env: Map<String, Value> = Map::new();
    env.insert("DOMAINFORGE_MODEL".to_string(), json!(model_ref));
    env.insert("DOMAINFORGE_NAMESPACE".to_string(), json!(ns));
    env.insert(
        "DOMAINFORGE_ENTITIES".to_string(),
        json!(entities.join(",")),
    );
    env.insert(
        "DOMAINFORGE_RESOURCES".to_string(),
        json!(resources.join(",")),
    );
    env.insert("DOMAINFORGE_FLOWS".to_string(), json!(flows.join(",")));

    let init_hook = json!([
        format!("echo \"DomainForge: {ns} namespace\""),
        "echo \"Entities: $DOMAINFORGE_ENTITIES\"".to_string(),
        "echo \"Resources: $DOMAINFORGE_RESOURCES\"".to_string(),
    ]);

    json!({
        "packages": [],
        "env": env,
        "shell": {
            "init_hook": init_hook,
            "scripts": {
                "domainforge-context": "echo \"Namespace: $DOMAINFORGE_NAMESPACE\"; echo \"Model: $DOMAINFORGE_MODEL\"; echo \"Entities: $DOMAINFORGE_ENTITIES\"; echo \"Resources: $DOMAINFORGE_RESOURCES\"; echo \"Flows: $DOMAINFORGE_FLOWS\""
            }
        }
    })
}

/// Derive a single namespace for the manifest: the namespace of the
/// alphabetically-first entity, else first resource, else `"default"`.
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
        project_devbox_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    fn doc(files: &BTreeMap<String, String>) -> Value {
        let body = &files["devbox.json"];
        let lines: Vec<&str> = body.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.trim_start().starts_with('{'))
            .unwrap();
        serde_json::from_str(&lines[start..].join("\n")).expect("parses as JSON")
    }

    #[test]
    fn emits_single_devbox_artifact() {
        let files = project(SOURCE);
        assert_eq!(files.keys().collect::<Vec<_>>(), vec!["devbox.json"]);
    }

    #[test]
    fn document_is_a_valid_devbox_manifest() {
        let files = project(SOURCE);
        let d = doc(&files);
        assert_eq!(d["packages"].as_array().unwrap(), &Vec::<Value>::new());
        let env = d["env"].as_object().expect("env is an object");
        assert_eq!(env["DOMAINFORGE_NAMESPACE"], "procurement");
        assert_eq!(env["DOMAINFORGE_MODEL"], "test.sea");
        assert_eq!(env["DOMAINFORGE_ENTITIES"], "Buyer,Supplier");
        assert_eq!(env["DOMAINFORGE_RESOURCES"], "PurchaseOrder");
        assert_eq!(env["DOMAINFORGE_FLOWS"], "PurchaseOrder:Buyer->Supplier");
        let hook = d["shell"]["init_hook"]
            .as_array()
            .expect("init_hook is an array");
        assert!(hook.len() >= 3);
        let script = d["shell"]["scripts"]["domainforge-context"]
            .as_str()
            .expect("context script present");
        assert!(script.contains("$DOMAINFORGE_NAMESPACE"));
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_default_namespace_and_empty_lists() {
        let files = project("@namespace \"empty\"\n");
        let d = doc(&files);
        assert_eq!(d["env"]["DOMAINFORGE_NAMESPACE"], "default");
        assert_eq!(d["env"]["DOMAINFORGE_ENTITIES"], "");
        assert_eq!(d["env"]["DOMAINFORGE_RESOURCES"], "");
        assert_eq!(d["env"]["DOMAINFORGE_FLOWS"], "");
        assert_eq!(d["packages"].as_array().unwrap().len(), 0);
    }
}

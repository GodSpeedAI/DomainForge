//! Dagger activation projection: a SEA model becomes a minimal Dagger module
//! — a [`dagger.json`] manifest plus a single-file Python module (`main.py` +
//! `pyproject.toml`) — so `dagger call` exposes one function per `Flow`,
//! letting CI activate each step of the architecture reproducibly.
//!
//! Each SEA `Flow` (a resource moving from one entity to another) becomes a
//! `@dagger.function` named `run_<resource>_<from>_<to>` returning a summary
//! string. The module is named after the model's namespace. This is the
//! smallest real Dagger module: `dagger develop --sdk=python` regenerates the
//! SDK layer on top of these three files.
//!
//! Output is sorted (flows ordered by resource/from/to; manifest keys via
//! serde_json's sorted `Map`), so it is byte-identical run-to-run for a fixed
//! model.
//!
//! [`dagger.json`]: https://docs.dagger.io/reference/dagger.schema.json

use crate::graph::Graph;
use crate::projection::sink::ArtifactSink;
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

/// Relative paths of the emitted artifacts.
pub const MANIFEST: &str = "dagger.json";
pub const MODULE_SRC: &str = "main.py";
pub const PYPROJECT: &str = "pyproject.toml";

/// Dagger engine version the projected module targets (format the gate
/// validates; bump when re-pinning against a newer schema).
pub const DAGGER_ENGINE_VERSION: &str = "v0.21.4";

/// Emit the Dagger module into `sink`; returns the emitted relative paths.
pub fn emit(
    graph: &Graph,
    model_ref: &str,
    created_at: Option<String>,
    sink: &mut ArtifactSink,
) -> Result<Vec<String>, String> {
    let created_at = created_at.unwrap_or_else(|| chrono::Utc::now().to_rfc3339());
    let ns = model_namespace(graph);
    let class_name = pascal_case(&ns);

    let manifest = build_manifest(&ns);
    let src = build_module_src(graph, &ns, &class_name, model_ref, &created_at)?;
    let project = build_pyproject(&ns);

    sink.write(MANIFEST, &manifest)?;
    sink.write(MODULE_SRC, &src)?;
    sink.write(PYPROJECT, &project)?;
    Ok(vec![
        MANIFEST.to_string(),
        MODULE_SRC.to_string(),
        PYPROJECT.to_string(),
    ])
}

/// Binding surface: string in, path→content map out, no filesystem.
pub fn project_dagger_in_memory(
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

/// Build the `dagger.json` manifest (schema: name + engineVersion required).
fn build_manifest(ns: &str) -> String {
    // serde_json::Map is BTreeMap-backed, so keys emit in sorted order.
    let mut m: Map<String, Value> = Map::new();
    m.insert(
        "$schema".to_string(),
        json!("https://docs.dagger.io/reference/dagger.schema.json"),
    );
    m.insert("name".to_string(), json!(ns));
    m.insert("sdk".to_string(), json!("python"));
    m.insert("source".to_string(), json!("."));
    m.insert("engineVersion".to_string(), json!(DAGGER_ENGINE_VERSION));
    format!(
        "// dagger.json projected by DomainForge. Dagger module manifest\n\
         // (schema: https://docs.dagger.io/reference/dagger.schema.json).\n{}\n",
        serde_json::to_string_pretty(&Value::Object(m)).expect("manifest serializes")
    )
}

/// Build the single-file Python module source.
fn build_module_src(
    graph: &Graph,
    ns: &str,
    class_name: &str,
    model_ref: &str,
    created_at: &str,
) -> Result<String, String> {
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

    let mut flows: Vec<(String, String, String, String)> = Vec::new();
    for f in graph.all_flows() {
        let (Some(from), Some(to), Some(resource)) = (
            entity_name.get(&f.from_id().to_string()),
            entity_name.get(&f.to_id().to_string()),
            resource_name.get(&f.resource_id().to_string()),
        ) else {
            continue;
        };
        flows.push((
            resource.clone(),
            from.clone(),
            to.clone(),
            f.quantity().to_string(),
        ));
    }
    flows.sort();

    let mut s = String::new();
    s.push_str(&format!(
        "\"\"\"Dagger module projected by DomainForge from {model_ref} at {created_at}.\n\
         \nEach SEA Flow becomes a callable Dagger function that activates that\n\
         step of the `{ns}` architecture. Regenerate the SDK layer with\n\
         `dagger develop --sdk=python`.\n\"\"\"\n\n\
         import dagger\n\n\n\
         @dagger.object_type\n\
         class {class_name}:\n"
    ));
    if flows.is_empty() {
        s.push_str("    pass\n");
    }
    for (resource, from, to, qty) in &flows {
        let fname = format!("run_{}_{}_{}", slug(resource), slug(from), slug(to));
        s.push_str(&format!(
            "\n    @dagger.function\n    def {fname}(self) -> str:\n        \"\"\"Activate flow: {resource} from {from} to {to} (quantity {qty}).\"\"\"\n        return \"{resource}: {from} -> {to} (x{qty})\"\n"
        ));
    }
    Ok(s)
}

/// Build the minimal single-file `pyproject.toml` (per the Dagger Python SDK
/// single-file layout in the module-init docs).
fn build_pyproject(ns: &str) -> String {
    format!(
        "# pyproject.toml projected by DomainForge. Minimal single-file Dagger\n\
         # Python module build config.\n\
         [build-system]\n\
         requires = [\"hatchling>=0.15.0\"]\n\
         build-backend = \"hatchling.build\"\n\n\
         [tool.hatch.build.targets.wheel]\n\
         packages = [\"main.py\"]\n\n\
         [project]\n\
         name = \"{ns}\"\n\
         version = \"0.0.0\"\n\
         requires-python = \">=3.10\"\n\
         dependencies = [\"dagger-io\"]\n"
    )
}

/// Derive a single namespace for the module: the namespace of the
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

/// Lowercase ASCII slug: keep alphanumerics, replace everything else with `_`.
/// Empty input yields `_`.
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

/// PascalCase a namespace into a Python class name: split on non-alphanumerics,
/// capitalize each word, concatenate. Non-alphabetic-leading words get an `_`
/// prefix to keep the identifier valid.
fn pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            if capitalize_next {
                out.push(c.to_ascii_uppercase());
                capitalize_next = false;
            } else {
                out.push(c);
            }
        } else {
            capitalize_next = true;
        }
    }
    if out.is_empty() {
        return "_".to_string();
    }
    // ponytail: leading digit is illegal as a Python identifier start; prefix `_`.
    if out.chars().next().map_or(true, |c| c.is_ascii_digit()) {
        out.insert(0, '_');
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
        project_dagger_in_memory(&graph, "test.sea", Some(FIXED_TS.to_string()))
            .expect("projection succeeds")
    }

    #[test]
    fn emits_three_artifacts() {
        let files = project(SOURCE);
        assert_eq!(
            files.keys().collect::<Vec<_>>(),
            vec!["dagger.json", "main.py", "pyproject.toml"]
        );
    }

    #[test]
    fn manifest_is_schema_valid_and_namespaced() {
        let files = project(SOURCE);
        let body = &files["dagger.json"];
        let lines: Vec<&str> = body.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.trim_start().starts_with('{'))
            .unwrap();
        let doc: Value =
            serde_json::from_str(&lines[start..].join("\n")).expect("manifest parses as JSON");
        assert_eq!(doc["name"], "procurement");
        assert_eq!(doc["sdk"], "python");
        assert_eq!(doc["source"], ".");
        assert!(doc["engineVersion"].as_str().unwrap().starts_with('v'));
        assert_eq!(
            doc["$schema"],
            "https://docs.dagger.io/reference/dagger.schema.json"
        );
    }

    #[test]
    fn module_src_has_one_function_per_flow() {
        let files = project(SOURCE);
        let src = &files["main.py"];
        assert!(
            src.contains("class Procurement:"),
            "class not PascalCase of namespace"
        );
        assert!(src.contains("@dagger.object_type"));
        // One function decorator per flow (1 flow here).
        assert_eq!(src.matches("@dagger.function").count(), 1);
        assert!(src.contains("def run_purchaseorder_buyer_supplier"));
        assert!(src.contains("\"PurchaseOrder: Buyer -> Supplier (x1)\""));
    }

    #[test]
    fn output_is_deterministic() {
        assert_eq!(project(SOURCE), project(SOURCE));
    }

    #[test]
    fn empty_model_emits_pass_module_and_default_namespace() {
        let files = project("@namespace \"empty\"\n");
        let body = &files["dagger.json"];
        let lines: Vec<&str> = body.lines().collect();
        let start = lines
            .iter()
            .position(|l| l.trim_start().starts_with('{'))
            .unwrap();
        let doc: Value = serde_json::from_str(&lines[start..].join("\n")).expect("manifest parses");
        assert_eq!(doc["name"], "default");
        let src = &files["main.py"];
        assert!(
            src.contains("class Default:"),
            "class name derived from fallback namespace"
        );
        assert_eq!(src.matches("@dagger.function").count(), 0);
        assert!(src.contains("    pass"));
    }
}

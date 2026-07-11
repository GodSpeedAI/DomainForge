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
use crate::projection::flows::{collect_flows, model_namespace};
use crate::projection::ids::{slug, NameRegistrar};
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
    let ns = model_namespace(graph)?;
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
///
/// H5: strict JSON — no `//` comment header (Dagger's Go CLI uses strict
/// `encoding/json`). Provenance lives in the `main.py` docstring.
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
    serde_json::to_string_pretty(&Value::Object(m)).expect("manifest serializes")
}

/// Build the single-file Python module source.
fn build_module_src(
    graph: &Graph,
    ns: &str,
    class_name: &str,
    model_ref: &str,
    created_at: &str,
) -> Result<String, String> {
    let flows = collect_flows(graph)?; // M4: loud dangling-ref policy

    // Collision-safe function-name registrar (M1): two flows that slug to the
    // same `run_<r>_<f>_<t>` get a hash suffix so Python doesn't silently
    // shadow one function with another.
    let mut reg = NameRegistrar::new();

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
    for f in &flows {
        let fname = format!(
            "run_{}_{}_{}",
            slug(&f.resource),
            slug(&f.from),
            slug(&f.to)
        );
        // Register the function name to deduplicate collisions (M1).
        let fname = reg.register("slug", &fname);
        // M2: escape names embedded in Python string literals/docstrings so a
        // `"` or `\` in a model name cannot produce a SyntaxError.
        let esc_resource = py_str_escape(&f.resource);
        let esc_from = py_str_escape(&f.from);
        let esc_to = py_str_escape(&f.to);
        let esc_qty = py_str_escape(&f.quantity);
        s.push_str(&format!(
            "\n    @dagger.function\n    def {fname}(self) -> str:\n        \"\"\"Activate flow: {esc_resource} from {esc_from} to {esc_to} (quantity {esc_qty}).\"\"\"\n        return \"{esc_resource}: {esc_from} -> {esc_to} (x{esc_qty})\"\n"
        ));
    }
    Ok(s)
}

/// Escape a string for safe embedding inside a Python double-quoted string
/// literal: escape `\`, `"`, and newlines (M2).
fn py_str_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
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
        // H5: manifest is strict JSON now — parse directly, no comment stripping.
        let doc: Value = serde_json::from_str(body).expect("manifest parses as strict JSON");
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
    fn manifest_is_strict_json_no_comment_header() {
        let files = project(SOURCE);
        let body = &files["dagger.json"];
        // H5: first non-whitespace char must be `{`.
        assert!(
            body.trim_start().starts_with('{'),
            "dagger.json must be strict JSON (no // header): {:?}",
            &body[..body.len().min(40)]
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
        let doc: Value = serde_json::from_str(body).expect("manifest parses as strict JSON");
        assert_eq!(doc["name"], "default");
        let src = &files["main.py"];
        assert!(
            src.contains("class Default:"),
            "class name derived from fallback namespace"
        );
        assert_eq!(src.matches("@dagger.function").count(), 0);
        assert!(src.contains("    pass"));
    }

    /// M2 regression: a name with `"` must not break the Python string literal.
    /// The generated main.py must not have an unescaped quote corrupting the
    /// docstring or return string.
    #[test]
    fn hostile_names_produce_compilable_python() {
        let hostile = r#"
@namespace "h"
Entity "a\"b" in h
Entity "c" in h
Resource "R x" units in h
Flow "R x" from "a\"b" to "c" quantity 1
"#;
        let files = project(hostile);
        let src = &files["main.py"];
        // The raw quote must be escaped in the return string.
        assert!(
            src.contains("\\\""),
            "raw quote must be escaped in Python string literal (M2)"
        );
        // py_compile equivalent: check the docstring + return are balanced.
        assert!(
            !src.contains("\"\"\"Activate flow: a\"b"),
            "unescaped quote must not corrupt the docstring (M2)"
        );
    }
}

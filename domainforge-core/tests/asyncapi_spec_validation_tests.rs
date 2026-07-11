//! Spec-validation gate (requirement 5): the projected `asyncapi.yaml`
//! validates against the official, vendored AsyncAPI 3.0.0 JSON Schema
//! (`schemas/asyncapi/3.0.0.json`). Run by the
//! `scripts/verify/projection-targets/asyncapi.sh` gate.

use domainforge_core::parser::parse_to_graph;
use domainforge_core::projection::asyncapi::project_asyncapi_in_memory;
use jsonschema::JSONSchema;
use std::fs;

const FIXED_TS: &str = "2026-07-02T00:00:00+00:00";

fn load_schema() -> serde_json::Value {
    let path = format!(
        "{}/../schemas/asyncapi/3.0.0.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read vendored AsyncAPI schema at {path}: {e}"));
    serde_json::from_str(&text).expect("vendored schema is valid JSON")
}

fn validate_against_schema(doc: &serde_json::Value) {
    let schema = load_schema();
    let compiled = JSONSchema::compile(&schema).expect("official AsyncAPI 3.0 schema compiles");
    let errs: Vec<String> = match compiled.validate(doc) {
        Ok(()) => Vec::new(),
        Err(it) => it.map(|e| format!("  - {e}")).collect(),
    };
    if !errs.is_empty() {
        panic!(
            "projected AsyncAPI document failed validation against the official 3.0 schema:\n{}",
            errs.join("\n")
        );
    }
}

#[test]
fn minimal_model_validates_against_asyncapi_spec() {
    let src = r#"
@namespace "procurement"
Entity "Buyer" in procurement
Entity "Supplier" in procurement
Resource "PurchaseOrder" units in procurement
Flow "PurchaseOrder" from "Buyer" to "Supplier" quantity 1
"#;
    let graph = parse_to_graph(src).expect("parses");
    let files =
        project_asyncapi_in_memory(&graph, "test.sea", Some(FIXED_TS.into())).expect("projects");
    let doc_yaml = &files["asyncapi.yaml"];
    let doc: serde_json::Value = serde_yaml::from_str(doc_yaml).expect("yaml parses");
    validate_against_schema(&doc);
}

#[test]
fn projection_cell_fixture_validates_against_asyncapi_spec() {
    let fixture = format!(
        "{}/../fixtures/projection_cell/basic/model.sea",
        env!("CARGO_MANIFEST_DIR")
    );
    let src = fs::read_to_string(&fixture)
        .unwrap_or_else(|e| panic!("failed to read projection-cell fixture: {e}"));
    let graph = parse_to_graph(&src).expect("fixture parses");
    let files = project_asyncapi_in_memory(
        &graph,
        "projection_cell/basic/model.sea",
        Some(FIXED_TS.into()),
    )
    .expect("projects");
    let doc_yaml = &files["asyncapi.yaml"];
    let doc: serde_json::Value = serde_yaml::from_str(doc_yaml).expect("yaml parses");
    // Fixture has 2 flows -> 2 channels, 4 operations, 2 messages, 2 schemas.
    assert_eq!(doc["channels"].as_object().unwrap().len(), 2);
    assert_eq!(doc["operations"].as_object().unwrap().len(), 4);
    validate_against_schema(&doc);
}

#[test]
fn empty_model_still_validates() {
    let graph = parse_to_graph("@namespace \"empty\"\n").expect("parses");
    let files =
        project_asyncapi_in_memory(&graph, "empty.sea", Some(FIXED_TS.into())).expect("projects");
    let doc: serde_json::Value =
        serde_yaml::from_str(&files["asyncapi.yaml"]).expect("yaml parses");
    validate_against_schema(&doc);
}

use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

#[test]
fn registry_schema_validates_sample() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let schema_path = manifest
        .join("..")
        .join("schemas")
        .join("sea-registry.schema.json");
    let schema = fs::read_to_string(&schema_path).expect("schema must exist");
    let schema_json: Value = serde_json::from_str(&schema).expect("valid JSON schema");
    let compiled = JSONSchema::options()
        .with_draft(Draft::Draft7)
        .compile(&schema_json)
        .expect("schema compiles");

    let sample_toml = r#"
version = 1
default_namespace = "default"

[[namespaces]]
namespace = "logistics"
patterns = ["domains/logistics/**/*.sea"]
"#;
    let value: toml::Value = toml::from_str(sample_toml).expect("valid toml");
    let json_val = serde_json::to_value(value).expect("serde conversion");

    let result = compiled.validate(&json_val);
    assert!(result.is_ok(), "schema must validate sample registry");
}

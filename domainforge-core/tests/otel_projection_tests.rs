#![cfg(feature = "cli")]

//! Integration tests for the OpenTelemetry SemConv projection
//! (fixtures/otel/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/otel/basic/domain/model.sea")
}

fn project_from(model: &Path, out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("otel-semconv")
        .arg("--created-at")
        .arg(FIXED_TS)
        .arg(model)
        .arg(out);
    cmd.assert()
}

fn project(out: &Path) -> assert_cmd::assert::Assert {
    project_from(&fixture_model(), out)
}

/// Attribute definition ids from the YAML registry (lines `- id: "..."` that are
/// not a group id / span id).
fn yaml_attribute_ids(yaml: &str) -> BTreeSet<String> {
    yaml.lines()
        .filter_map(|l| l.trim().strip_prefix("- id: "))
        .map(|q| q.trim_matches('"').to_string())
        .filter(|k| !k.starts_with("registry.") && !k.starts_with("span."))
        .collect()
}

/// Constant values (the attribute keys) from a `attributes.rs`/`.py`/`.ts` file.
fn const_values(src: &str) -> BTreeSet<String> {
    src.lines()
        .filter_map(|l| {
            l.split("= \"")
                .nth(1)
                .or_else(|| l.split(": str = \"").nth(1))
        })
        .map(|q| {
            q.trim_end_matches(';')
                .trim_end_matches('"')
                .trim_matches('"')
                .to_string()
        })
        .filter(|s| s.contains('.'))
        .collect()
}

#[test]
fn emits_registry_and_constants() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let files = read_tree(tmp.path());
    assert_eq!(
        files.keys().collect::<Vec<_>>(),
        vec![
            "constants/attributes.py",
            "constants/attributes.rs",
            "constants/attributes.ts",
            "registry/telemetry.yaml",
        ]
    );
    let yaml = &files["registry/telemetry.yaml"];
    // Correlation keystone present on the resource group.
    assert!(yaml.contains("domainforge.model.hash"));
    assert!(yaml.contains("domainforge.element.id"));
    // Domain attributes are under the model namespace.
    assert!(yaml.contains("demo.entity."));
    assert!(yaml.contains("demo.flow."));
    assert!(yaml.contains("demo.policy."));
    // Span-kind suggestion for a flow step.
    assert!(yaml.contains("type: span"));
    assert!(yaml.contains("span_kind: internal"));
    // No OTel-reserved namespace leaked into an attribute id.
    for reserved in ["service.", "otel.", "telemetry.", "http.", "rpc."] {
        assert!(
            !yaml.contains(&format!("- id: \"{reserved}")),
            "reserved namespace {reserved} leaked into the registry"
        );
    }
}

/// The single-producer check: the YAML registry and ALL THREE constant files
/// name exactly the same attribute set.
#[test]
fn registry_and_constants_agree_on_attribute_set() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let files = read_tree(tmp.path());

    let yaml_ids = yaml_attribute_ids(&files["registry/telemetry.yaml"]);
    assert!(!yaml_ids.is_empty(), "no attribute ids parsed from YAML");

    let rs = const_values(&files["constants/attributes.rs"]);
    let py = const_values(&files["constants/attributes.py"]);
    let ts = const_values(&files["constants/attributes.ts"]);

    assert_eq!(yaml_ids, rs, "YAML vs attributes.rs disagree");
    assert_eq!(yaml_ids, py, "YAML vs attributes.py disagree");
    assert_eq!(yaml_ids, ts, "YAML vs attributes.ts disagree");
    assert!(yaml_ids.contains("domainforge.model.hash"));
}

#[test]
fn output_is_byte_deterministic() {
    assert_byte_deterministic(|out| {
        project(out).success();
    });
}

/// Renaming an entity changes exactly the attribute keys derived from it — the
/// deterministic-identity teeth-check for this family.
#[test]
fn renaming_an_entity_changes_only_its_attributes() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let before = yaml_attribute_ids(&read_tree(tmp.path())["registry/telemetry.yaml"]);
    assert!(before.iter().any(|k| k.contains("purchaseorder")));
    assert!(!before.iter().any(|k| k.contains("purchaserequest")));
}

#[test]
fn output_must_be_a_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("not_a_dir");
    std::fs::write(&file, "occupied").expect("write");
    project(&file).failure();
}

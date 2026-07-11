#![cfg(feature = "cli")]

//! Integration tests for the RDF/OWL projection
//! (fixtures/rdf/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::path::{Path, PathBuf};

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/rdf/basic/domain/model.sea")
}

fn project_from(model: &Path, out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("rdf")
        .arg("--created-at")
        .arg(FIXED_TS)
        .arg(model)
        .arg(out);
    cmd.assert()
}

fn project(out: &Path) -> assert_cmd::assert::Assert {
    project_from(&fixture_model(), out)
}

#[test]
fn emits_the_three_artifact_manifest() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let files = read_tree(tmp.path());
    for expected in ["model.ttl", "model.jsonld", "ontology.owl.ttl"] {
        assert!(files.contains_key(expected), "missing {expected}");
    }
    assert!(files["model.ttl"].contains("sea:Warehouse rdf:type sea:Entity"));
    assert!(files["ontology.owl.ttl"].contains("sea:from a owl:ObjectProperty"));
    // JSON-LD must be parseable and carry the graph + context.
    let doc: serde_json::Value =
        serde_json::from_str(&files["model.jsonld"]).expect("model.jsonld is valid JSON");
    assert!(doc["@context"]["sea"].is_string());
    assert!(doc["@graph"].is_array());
}

#[test]
fn output_is_byte_deterministic() {
    assert_byte_deterministic(|out| {
        project(out).success();
    });
}

#[test]
fn output_must_be_a_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("not_a_dir");
    std::fs::write(&file, "occupied").expect("write");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("rdf")
        .arg(fixture_model())
        .arg(&file);
    cmd.assert().failure();
}

#[test]
fn recipe_flag_is_rejected() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let recipe = tmp.path().join("recipe.json");
    std::fs::write(&recipe, "{}").expect("write");
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("rdf")
        .arg("--recipe")
        .arg(&recipe)
        .arg(fixture_model())
        .arg(tmp.path().join("out"));
    cmd.assert().failure();
}

/// Teeth-check for deterministic identity: renaming exactly one entity changes
/// exactly the IRIs derived from that entity — every triple that mentions an
/// *untouched* concept is byte-identical across both projections, while the
/// renamed entity's IRIs disappear and the new name's appear.
#[test]
fn renaming_one_entity_changes_only_its_iris() {
    let tmp_base = tempfile::tempdir().expect("tempdir");
    project(tmp_base.path()).success();
    let base = read_tree(tmp_base.path());

    // A copy of the fixture with "Warehouse" renamed to "Depot".
    let renamed_src = std::fs::read_to_string(fixture_model())
        .expect("read fixture")
        .replace("Warehouse", "Depot");
    let tmp_model = tempfile::tempdir().expect("tempdir");
    let model_path = tmp_model.path().join("model.sea");
    std::fs::write(&model_path, renamed_src).expect("write renamed model");
    let tmp_out = tempfile::tempdir().expect("tempdir");
    project_from(&model_path, tmp_out.path()).success();
    let renamed = read_tree(tmp_out.path());

    assert_eq!(
        base.keys().collect::<Vec<_>>(),
        renamed.keys().collect::<Vec<_>>()
    );

    // Every concept NOT derived from Warehouse keeps byte-identical lines: the
    // set of lines mentioning it is unchanged across the rename.
    let untouched = [
        "Factory",
        "Operator",
        "Supervisor",
        "CameraUnits",
        "Oversight",
        "supervises",
    ];
    for (path, base_content) in &base {
        let new_content = &renamed[path];
        // Flow/pattern nodes are anonymous nodes whose deterministic id derives
        // from ALL their referents, so a flow that points at the renamed entity
        // legitimately changes — exclude those lines from the untouched check.
        let named = |l: &&str| !l.contains("sea:flow_") && !l.contains("sea:pattern_");
        for name in untouched {
            let base_lines: Vec<&str> = base_content
                .lines()
                .filter(|l| l.contains(name))
                .filter(named)
                .collect();
            let new_lines: Vec<&str> = new_content
                .lines()
                .filter(|l| l.contains(name))
                .filter(named)
                .collect();
            assert_eq!(
                base_lines, new_lines,
                "{path}: lines for untouched concept `{name}` changed under an unrelated rename"
            );
        }
    }

    // The renamed entity's IRIs moved wholesale from Warehouse to Depot.
    assert!(base["model.ttl"].contains("sea:Warehouse rdf:type sea:Entity"));
    assert!(!renamed["model.ttl"].contains("sea:Warehouse"));
    assert!(renamed["model.ttl"].contains("sea:Depot rdf:type sea:Entity"));
    assert!(renamed["ontology.owl.ttl"].contains("sea:Depot a owl:NamedIndividual, sea:Entity"));
}

#![cfg(feature = "cli")]

//! Integration tests for the ArchiMate 3.0 projection
//! (fixtures/archimate/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::path::{Path, PathBuf};

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/archimate/basic/domain/model.sea")
}

fn project_from(model: &Path, out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("archimate")
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
fn emits_a_single_model_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let files = read_tree(tmp.path());
    assert_eq!(files.keys().collect::<Vec<_>>(), vec!["model.xml"]);
    let xml = &files["model.xml"];
    assert!(xml.contains("<model"));
    // Roles become business roles.
    assert!(xml.contains("xsi:type=\"BusinessRole\"") && xml.contains(">Approver<"));
    // Entities and resources become business objects.
    assert!(xml.contains("xsi:type=\"BusinessObject\""));
    for name in [">PurchaseOrder<", ">Invoice<", ">Budget<"] {
        assert!(xml.contains(name), "missing object {name}");
    }
    // Flows become business processes, accessing their endpoints.
    assert!(xml.contains("xsi:type=\"BusinessProcess\""));
    assert!(xml.contains("xsi:type=\"Access\""));
    // Chained flows trigger one another.
    assert!(xml.contains("xsi:type=\"Triggering\""));
    // Policies become requirements associated with the objects they reference.
    assert!(xml.contains("xsi:type=\"Requirement\""));
    assert!(xml.contains("xsi:type=\"Association\""));
    // One auto-generated business-layer view listing element refs.
    assert!(xml.contains("<views"));
    assert!(xml.contains("xsi:type=\"Diagram\""));
    assert!(xml.contains("elementRef="));
}

#[test]
fn output_is_byte_deterministic() {
    assert_byte_deterministic(|out| {
        project(out).success();
    });
}

// Only spec-legal relations reach the output: the emitted relationship xsi:type
// values are all in the ArchiMate matrix subset the projection uses. (The
// by-construction rejection of illegal relations is unit-tested in the module.)
#[test]
fn every_emitted_relation_type_is_known() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let xml = read_tree(tmp.path())["model.xml"].clone();
    // No Assignment between two objects, etc. — a coarse sanity check that the
    // renderer emitted only relationship elements (never a bare illegal tag).
    let legal = [
        "Assignment",
        "Access",
        "Triggering",
        "Flow",
        "Association",
        "Realization",
        "Serving",
        "Composition",
        "Aggregation",
        "Specialization",
        "Influence",
    ];
    // Every `<relationship ... xsi:type="X"` X must be one of the legal names.
    // Split on "<relationship " (with the trailing space) so the "<relationships>"
    // container tag itself is never mistaken for a relationship element.
    for chunk in xml.split("<relationship ").skip(1) {
        let ty = chunk
            .split("xsi:type=\"")
            .nth(1)
            .and_then(|s| s.split('"').next())
            .expect("relationship has xsi:type");
        assert!(legal.contains(&ty), "unexpected relationship type {ty}");
    }
}

#[test]
fn output_must_be_a_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("not_a_dir");
    std::fs::write(&file, "occupied").expect("write");
    project(&file).failure();
}

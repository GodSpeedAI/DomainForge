#![cfg(feature = "cli")]

//! Integration tests for the BPMN 2.0 projection
//! (fixtures/bpmn/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::path::{Path, PathBuf};

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/bpmn/basic/domain/model.sea")
}

fn project_from(model: &Path, out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("bpmn")
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
fn emits_a_single_bpmn_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let files = read_tree(tmp.path());
    assert_eq!(files.keys().collect::<Vec<_>>(), vec!["model.bpmn"]);
    let bpmn = &files["model.bpmn"];
    assert!(bpmn.contains("<definitions"));
    assert!(bpmn.contains("isExecutable=\"false\""));
    // Branching produces both a diverging and a converging parallel gateway.
    assert!(bpmn.contains("gatewayDirection=\"Diverging\""));
    assert!(bpmn.contains("gatewayDirection=\"Converging\""));
    // Roles become swimlanes; resources become data objects.
    assert!(bpmn.contains("<lane") && bpmn.contains("name=\"Operator\""));
    assert!(bpmn.contains("<dataObject") && bpmn.contains("name=\"CameraUnits\""));
    // Every entity in a flow becomes a task.
    for name in ["Warehouse", "LineA", "LineB", "Assembly"] {
        assert!(
            bpmn.contains(&format!("name=\"{name}\"")),
            "missing task {name}"
        );
    }
}

#[test]
fn output_is_byte_deterministic() {
    assert_byte_deterministic(|out| {
        project(out).success();
    });
}

#[test]
fn renaming_an_entity_changes_only_its_derived_ids() {
    // Deterministic-identity teeth-check: renaming one entity changes exactly the
    // ids derived from it (its task) and leaves an unrelated entity's task id
    // byte-identical. Same input *path* both times, so process/definitions ids
    // are held constant and only content-derived ids can move.
    let dir = tempfile::tempdir().expect("tempdir");
    let model = dir.path().join("model.sea");
    let base = r#"@namespace "demo"
Entity "Warehouse" in demo
Entity "Assembly" in demo
Resource "R" units in demo
Flow "R" from "Warehouse" to "Assembly" quantity 1
"#;
    std::fs::write(&model, base).expect("write");
    let out_a = dir.path().join("a");
    project_from(&model, &out_a).success();
    let before = read_tree(&out_a)["model.bpmn"].clone();

    // Extract the two task ids by their names.
    let warehouse_before = task_id_for(&before, "Warehouse");
    let assembly_before = task_id_for(&before, "Assembly");

    std::fs::write(&model, base.replace("Assembly", "FinalAssembly")).expect("rewrite");
    let out_b = dir.path().join("b");
    project_from(&model, &out_b).success();
    let after = read_tree(&out_b)["model.bpmn"].clone();

    // Warehouse is untouched → identical task id; the renamed entity's id moves.
    assert_eq!(warehouse_before, task_id_for(&after, "Warehouse"));
    assert_ne!(assembly_before, task_id_for(&after, "FinalAssembly"));
}

/// Pull the `id` of the `<task ... name="NAME">` element out of BPMN XML.
fn task_id_for(bpmn: &str, name: &str) -> String {
    let needle = format!("name=\"{name}\"");
    let line = bpmn
        .lines()
        .find(|l| l.contains("<task ") && l.contains(&needle))
        .unwrap_or_else(|| panic!("no task named {name}"));
    let start = line.find("id=\"").expect("id attr") + 4;
    let end = line[start..].find('"').expect("id close") + start;
    line[start..end].to_string()
}

#[test]
fn output_must_be_a_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("not_a_dir");
    std::fs::write(&file, "occupied").expect("write");
    project(&file).failure();
}

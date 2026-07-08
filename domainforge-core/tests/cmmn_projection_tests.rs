#![cfg(feature = "cli")]

//! Integration tests for the CMMN 1.1 projection
//! (fixtures/cmmn/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::path::{Path, PathBuf};

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/cmmn/basic/domain/model.sea")
}

fn project_from(model: &Path, out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("cmmn")
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
fn emits_a_single_cmmn_file() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let files = read_tree(tmp.path());
    assert_eq!(files.keys().collect::<Vec<_>>(), vec!["model.cmmn"]);
    let cmmn = &files["model.cmmn"];
    assert!(cmmn.contains("<definitions"));
    assert!(cmmn.contains("<case "));
    assert!(cmmn.contains("<casePlanModel"));
    // Resources become case file items.
    assert!(cmmn.contains("<caseFileItem") && cmmn.contains("name=\"Budget\""));
    // Roles become case roles.
    assert!(cmmn.contains("<caseRoles") && cmmn.contains("name=\"Approver\""));
    // Entities become human tasks.
    for name in ["PurchaseOrder", "Invoice"] {
        assert!(
            cmmn.contains(&format!("name=\"{name}\"")),
            "missing task {name}"
        );
    }
    // Policies become milestones gated by sentries.
    assert!(cmmn.contains("<milestone"));
    assert!(cmmn.contains("<sentry"));
    assert!(cmmn.contains("<entryCriterion"));
}

#[test]
fn output_is_byte_deterministic() {
    assert_byte_deterministic(|out| {
        project(out).success();
    });
}

// Policy→sentry teeth-check at the fixture level: a fixture *with* a policy
// condition produces a sentry, and the same fixture *without* the policy
// produces none.
#[test]
fn removing_a_policy_removes_its_sentry() {
    let dir = tempfile::tempdir().expect("tempdir");

    let with_policy = r#"@namespace "demo"
Resource "Budget" units in demo
Policy positive_budget as: Budget.quantity > 0
"#;
    let model_a = dir.path().join("with.sea");
    std::fs::write(&model_a, with_policy).expect("write");
    let out_a = dir.path().join("a");
    project_from(&model_a, &out_a).success();
    let a = read_tree(&out_a)["model.cmmn"].clone();
    assert!(a.contains("<sentry"), "a policy must produce a sentry");
    assert!(
        a.contains("<milestone"),
        "a policy must produce a milestone"
    );
    assert!(
        a.contains("Budget.quantity &gt; 0"),
        "condition must be lowered"
    );

    let without_policy = r#"@namespace "demo"
Resource "Budget" units in demo
"#;
    let model_b = dir.path().join("without.sea");
    std::fs::write(&model_b, without_policy).expect("write");
    let out_b = dir.path().join("b");
    project_from(&model_b, &out_b).success();
    let b = read_tree(&out_b)["model.cmmn"].clone();
    assert!(
        !b.contains("<sentry"),
        "removing the policy must remove the sentry (teeth-check)"
    );
    assert!(!b.contains("<milestone"));
}

#[test]
fn output_must_be_a_directory() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let file = tmp.path().join("not_a_dir");
    std::fs::write(&file, "occupied").expect("write");
    project(&file).failure();
}

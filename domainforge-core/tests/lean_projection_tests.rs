#![cfg(feature = "cli")]

//! Integration tests for the Lean 4 projection
//! (fixtures/lean/basic is the proving fixture).

mod common;

use assert_cmd::Command;
use common::projection_harness::{assert_byte_deterministic, read_tree, FIXED_TS};
use std::path::{Path, PathBuf};

fn fixture_model() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../fixtures/lean/basic/domain/model.sea")
}

fn project(out: &Path) -> assert_cmd::assert::Assert {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"));
    cmd.arg("project")
        .arg("--format")
        .arg("lean")
        .arg("--created-at")
        .arg(FIXED_TS)
        .arg(fixture_model())
        .arg(out);
    cmd.assert()
}

#[test]
fn emits_complete_lake_package() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let files = read_tree(tmp.path());
    for expected in [
        "lean-toolchain",
        "lakefile.toml",
        "README.md",
        "DomainForge.lean",
        "DomainForge/Types.lean",
        "DomainForge/Model.lean",
        "DomainForge/Policies.lean",
        "Obligations.lean",
        "Obligations/Stubs.lean",
    ] {
        assert!(files.contains_key(expected), "missing {expected}");
    }
    assert!(files["lakefile.toml"].contains("defaultTargets = [\"DomainForge\"]"));
}

#[test]
fn checked_layer_is_sorry_free_and_obligations_carry_the_stubs() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let files = read_tree(tmp.path());

    for (path, content) in &files {
        if path.starts_with("DomainForge") {
            assert!(!content.contains("sorry"), "{path} must be sorry-free");
        }
    }
    let stubs = &files["Obligations/Stubs.lean"];
    assert!(stubs.contains("theorem obligation_named_entities : True := by sorry"));

    let policies = &files["DomainForge/Policies.lean"];
    assert!(
        policies.contains("theorem policy_positive_flow_holds : policy_positive_flow := by decide")
    );
    assert!(policies.contains("theorem policy_overload_violated : ¬ policy_overload := by decide"));
    assert!(policies.contains("theorem checked_policies_consistent"));
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
        .arg("lean")
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
        .arg("lean")
        .arg("--recipe")
        .arg(&recipe)
        .arg(fixture_model())
        .arg(tmp.path().join("out"));
    cmd.assert().failure();
}

/// End-to-end proof check: requires a Lean toolchain (`lake` + `elan`) on PATH.
/// Run explicitly with `cargo test --features cli -- --ignored lake_build`.
#[test]
#[ignore = "requires a Lean 4 toolchain on PATH"]
fn lake_build_checks_the_generated_package() {
    let tmp = tempfile::tempdir().expect("tempdir");
    project(tmp.path()).success();
    let output = std::process::Command::new("lake")
        .arg("build")
        .current_dir(tmp.path())
        .output()
        .expect("lake on PATH");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "lake build failed:\n{log}");
    assert!(
        !log.contains("uses 'sorry'"),
        "default lib has sorry:\n{log}"
    );
}

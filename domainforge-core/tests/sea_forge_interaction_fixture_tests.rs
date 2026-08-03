#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/sea_forge_interaction_i1_i3")
        .join(name)
}

#[test]
fn faithful_modular_interaction_fixture_resolves_and_policy_passes() {
    let output = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .args(["parse", "--format", "json"])
        .arg(fixture("valid-instances.sea"))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let graph: domainforge_core::Graph =
        serde_json::from_slice(&output).expect("CLI emits a complete graph");

    assert_eq!(graph.entity_count(), 2);
    assert_eq!(graph.entity_instance_count(), 2);
    assert_eq!(graph.all_policies().len(), 1);
    assert!(
        graph.all_policies()[0]
            .evaluate(&graph)
            .unwrap()
            .is_satisfied
    );
}

#[test]
fn faithful_modular_interaction_fixture_rejects_invalid_enum_and_reference() {
    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("validate")
        .arg(fixture("invalid-instances.sea"))
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("maturity")
                .and(predicate::str::contains("unknown"))
                .and(predicate::str::contains("journey"))
                .and(predicate::str::contains("MISSING")),
        );
}

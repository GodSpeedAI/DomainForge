#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

fn write_modular_fixture(invalid_maturity: bool) -> (tempfile::TempDir, std::path::PathBuf) {
    let directory = tempdir().expect("temp fixture directory");
    let types = directory.path().join("types.sea");
    let instances = directory.path().join("instances.sea");
    std::fs::write(
        types,
        r#"@namespace "sea_forge.interaction"
export enum Maturity { specified = "specified", exercised = "exercised" }
export entity "CanonicalJourney" {
    key journey_id: string
    maturity: Maturity
}
"#,
    )
    .expect("write types module");
    let maturity = if invalid_maturity {
        "unknown"
    } else {
        "exercised"
    };
    std::fs::write(
        &instances,
        format!(
            r#"@namespace "sea_forge.interaction"
import {{ CanonicalJourney, Maturity }} from "./types.sea"
instance cj01 of "CanonicalJourney" {{
    journey_id: "CJ01",
    maturity: "{maturity}"
}}
"#
        ),
    )
    .expect("write instances module");
    (directory, instances)
}

#[test]
fn validate_resolves_imported_entity_for_same_namespace_instance() {
    let (_directory, entry) = write_modular_fixture(false);

    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("validate")
        .arg(&entry)
        .assert()
        .success()
        .stdout(predicate::str::contains("Validation succeeded"));
}

#[test]
fn parse_graph_json_contains_imported_entity_and_instance() {
    let (_directory, entry) = write_modular_fixture(false);

    let output = Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("parse")
        .arg("--format")
        .arg("json")
        .arg(&entry)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let graph: serde_json::Value = serde_json::from_slice(&output).expect("graph JSON");

    assert_eq!(
        graph["entities"].as_object().map(|items| items.len()),
        Some(1)
    );
    assert_eq!(
        graph["entity_instances"]
            .as_object()
            .map(|items| items.len()),
        Some(1)
    );
    assert!(graph["entity_contracts"].is_object());
}

#[test]
fn validate_rejects_invalid_typed_data_from_imported_contract() {
    let (_directory, entry) = write_modular_fixture(true);

    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("validate")
        .arg(&entry)
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("cj01")
                .and(predicate::str::contains("maturity"))
                .and(predicate::str::contains("unknown")),
        );
}

#[test]
fn validate_does_not_apply_entry_default_namespace_to_imported_modules() {
    let directory = tempdir().expect("temp fixture directory");
    let types = directory.path().join("types.sea");
    let entry = directory.path().join("instances.sea");
    std::fs::write(&types, "export entity \"Journey\" { key id: string }\n")
        .expect("write unnamespaced imported module");
    std::fs::write(
        &entry,
        r#"@namespace "sea_forge.interaction"
import { Journey } from "./types.sea"
instance cj01 of "Journey" { id: "CJ01" }
"#,
    )
    .expect("write entry module");

    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("validate")
        .arg(&entry)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "has no declared or registry namespace",
        ));
}

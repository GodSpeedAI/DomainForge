#![cfg(feature = "cli")]

//! No CLI subcommand resolved the Application Contract or Semantic Envelope
//! before this suite: `parse`/`validate`/`project` only ever resolved the
//! graph. `domainforge validate` therefore reported success on a file whose
//! declared operation was structurally broken, because it never looked at
//! the operation at all.

use assert_cmd::Command;
use predicates::prelude::*;
use std::path::PathBuf;
use tempfile::tempdir;

fn flagship_fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fixtures/application_generation/flagship")
        .join(name)
}

/// A minimal single-file operation whose `output` field does not project a
/// compatible `state` field (APP011) — the mutation this suite exercises.
const BROKEN_OPERATION: &str = r#"
@namespace "cli_application_test"

export entity "Widget" {
    key widget_id: string (min_length 1)
    label: string
}

record GetWidgetInput {
    widget_id: string
}

record GetWidgetOutput {
    widget_id: string
    quantity: int
}

operation get_widget {
    intent "return one widget by id"
    direction inbound
    actor anonymous
    access public
    input GetWidgetInput
    output GetWidgetOutput
    state Widget
    effect reads Widget
    transaction read_only
    failure widget_not_found for missing_state "no widget exists for the given widget_id"
    idempotency inherent
    concurrency read_snapshot
    evidence operation_trace
    lifecycle synchronous_request_response
}
"#;

/// A structurally sound single-file operation, deliberately independent of
/// the flagship fixtures: `query-read.sea` imports `command-write.sea`,
/// whose `order_total_within_limit` operation-bound policy currently fails
/// *graph* validation (limitation L9, not yet fixed), which would fail these
/// assertions for a reason unrelated to the `--application` flag itself.
const SOUND_OPERATION: &str = r#"
@namespace "cli_application_test_sound"

export entity "Widget" {
    key widget_id: string (min_length 1)
    label: string
}

record GetWidgetInput {
    widget_id: string
}

record GetWidgetOutput {
    widget_id: string (min_length 1)
    label: string
}

operation get_widget {
    intent "return one widget by id"
    direction inbound
    actor anonymous
    access public
    input GetWidgetInput
    output GetWidgetOutput
    state Widget
    effect reads Widget
    transaction read_only
    failure widget_not_found for missing_state "no widget exists for the given widget_id"
    idempotency inherent
    concurrency read_snapshot
    evidence operation_trace
    lifecycle synchronous_request_response
}
"#;

#[test]
fn validate_without_application_flag_ignores_operation_errors() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("broken.sea");
    std::fs::write(&path, BROKEN_OPERATION).expect("write fixture");

    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("validate")
        .arg(&path)
        .assert()
        .success();
}

#[test]
fn validate_with_application_flag_catches_operation_errors() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("broken.sea");
    std::fs::write(&path, BROKEN_OPERATION).expect("write fixture");

    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .args(["validate", "--application"])
        .arg(&path)
        .assert()
        .failure()
        .stdout(predicate::str::contains("Application contract invalid"));
}

#[test]
fn validate_with_application_flag_passes_a_sound_operation() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("sound.sea");
    std::fs::write(&path, SOUND_OPERATION).expect("write fixture");

    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .args(["validate", "--application"])
        .arg(&path)
        .assert()
        .success()
        .stdout(predicate::str::contains("Application contract valid"));
}

#[test]
fn contract_subcommand_prints_the_resolved_operation() {
    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("contract")
        .arg(flagship_fixture("query-read.sea"))
        .assert()
        .success()
        .stdout(
            predicate::str::contains("domainforge-application-contract/v1")
                .and(predicate::str::contains("get_order_status"))
                .and(predicate::str::contains("semantic_closure_hash")),
        );
}

#[test]
fn envelope_subcommand_prints_the_resolved_declarations() {
    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("envelope")
        .arg(flagship_fixture("query-read.sea"))
        .assert()
        .success()
        .stdout(
            predicate::str::contains("domainforge-semantic-envelope/v1")
                .and(predicate::str::contains("semantic_declarations"))
                .and(predicate::str::contains("semantic_closure_hash")),
        );
}

#[test]
fn flagship_command_write_fixture_passes_graph_validation() {
    // DomainForge's own flagship fixture declares an operation-bound policy
    // (`order_total_within_limit`, referenced via `access policy_governed
    // by ... at precondition`). Before the L9 fix, this failed
    // `domainforge validate` with "Policy evaluation is UNKNOWN (NULL)"
    // because the graph validator evaluated it against the bare graph,
    // where `total` (an input-record field, not a graph field) does not
    // resolve. This is the natural, pre-existing acceptance test for that
    // fix — no new fixture needed.
    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("validate")
        .arg(flagship_fixture("command-write.sea"))
        .assert()
        .success()
        .stdout(predicate::str::contains("Validation succeeded"));
}

#[test]
fn contract_subcommand_reports_diagnostics_for_a_broken_operation() {
    let dir = tempdir().expect("tempdir");
    let path = dir.path().join("broken.sea");
    std::fs::write(&path, BROKEN_OPERATION).expect("write fixture");

    Command::new(assert_cmd::cargo::cargo_bin!("domainforge"))
        .arg("contract")
        .arg(&path)
        .assert()
        .failure()
        .stderr(predicate::str::contains("effect_state_mismatch"));
}

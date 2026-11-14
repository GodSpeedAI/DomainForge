#![cfg(feature = "cli")]

//! `domainforge evaluate`: per-binding tri-state results. Golden cases a-d come
//! from the WP-10 spike (empty facts, mixed instances, unknown, operation-bound).

use assert_cmd::Command;
use serde_json::{json, Value};
use std::path::PathBuf;
use tempfile::tempdir;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/evaluate")
        .join(name)
}

fn run_with(model: &PathBuf, facts: &PathBuf, extra: &[&str]) -> std::process::Output {
    Command::cargo_bin("domainforge")
        .unwrap()
        .arg("evaluate")
        .arg("--model")
        .arg(model)
        .arg("--facts")
        .arg(facts)
        .args(["--format", "json"])
        .args(extra)
        .output()
        .unwrap()
}

fn evaluate(facts: &str) -> Value {
    let out = run_with(&fixture("model.sea"), &fixture(facts), &[]);
    assert!(
        out.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

fn golden(name: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(fixture(name)).unwrap()).unwrap()
}

fn policy<'a>(report: &'a Value, name: &str) -> &'a Value {
    report["policies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["policy"] == name)
        .unwrap_or_else(|| panic!("policy {name}"))
}

fn results(p: &Value, key: &str) -> Vec<(String, String)> {
    p["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b[key].as_str().unwrap().to_string(),
                b["result"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

fn pairs(items: &[(&str, &str)]) -> Vec<(String, String)> {
    items
        .iter()
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

#[test]
fn evaluate_case_a_empty_facts_is_vacuously_true_with_zero_support() {
    let report = evaluate("facts_a_empty.json");
    assert_eq!(report["policies"], golden("golden_a_empty.json"));
    let p = policy(&report, "executed_steps_have_approval");
    assert_eq!(
        (p["result"].as_str(), p["support"].as_u64()),
        (Some("true"), Some(0))
    );
    let op = policy(&report, "case_must_be_open");
    assert_eq!(op["result"], "unknown");
    assert_eq!(op["reason"], "no_operation_input");
}

#[test]
fn evaluate_case_b_identifies_the_violating_instance() {
    let report = evaluate("facts_b_mixed.json");
    assert_eq!(report["policies"], golden("golden_b_mixed.json"));
    let p = policy(&report, "executed_steps_have_approval");
    assert_eq!(p["result"], "false");
    assert_eq!(p["support"], 3);
    assert_eq!(p["binding_depth"], "outermost");
    assert_eq!(
        results(p, "instance"),
        pairs(&[("s_bad", "false"), ("s_ok", "true"), ("s_planned", "true")])
    );
}

#[test]
fn evaluate_case_c_missing_attribute_is_unknown_and_false_dominates() {
    let report = evaluate("facts_c_unknown.json");
    assert_eq!(report["policies"], golden("golden_c_unknown.json"));
    let p = policy(&report, "evidence_retained_30_days");
    assert_eq!(p["result"], "unknown");
    assert_eq!(
        results(p, "instance"),
        pairs(&[("e_missing", "unknown"), ("e_ok", "true")])
    );

    let report = evaluate("facts_c_known_bad.json");
    assert_eq!(report["policies"], golden("golden_c_known_bad.json"));
    let p = policy(&report, "evidence_retained_30_days");
    assert_eq!(p["result"], "false");
    assert_eq!(
        results(p, "instance"),
        pairs(&[("e_bad", "false"), ("e_missing", "unknown")])
    );
}

#[test]
fn evaluate_case_d_operation_bound_policy_open_closed_missing() {
    let report = evaluate("facts_d_operation.json");
    assert_eq!(report["policies"], golden("golden_d_operation.json"));
    let p = policy(&report, "case_must_be_open");
    assert_eq!(p["operation_bound"], true);
    assert_eq!(p["operation"], "close_case");
    assert_eq!(p["result"], "false");
    assert_eq!(
        results(p, "input_set_id"),
        pairs(&[
            ("in_closed", "false"),
            ("in_missing", "unknown"),
            ("in_open", "true")
        ])
    );
}

#[test]
fn evaluate_output_records_hashes_version_and_validates_against_schema() {
    let report = evaluate("facts_b_mixed.json");
    let model = std::fs::read(fixture("model.sea")).unwrap();
    let facts = std::fs::read(fixture("facts_b_mixed.json")).unwrap();
    assert_eq!(
        report["model_sha256"],
        domainforge_core::semantic_pack::canonical_json::compute_sha256(&model)
    );
    assert_eq!(
        report["facts_sha256"],
        domainforge_core::semantic_pack::canonical_json::compute_sha256(&facts)
    );
    assert_eq!(report["compiler"]["version"], env!("CARGO_PKG_VERSION"));
    assert!(report["world_ref"].is_null());

    let schema: Value =
        serde_json::from_str(include_str!("../../schemas/evaluate-output.v1.json")).unwrap();
    let compiled = jsonschema::JSONSchema::compile(&schema).unwrap();
    for facts in [
        "facts_a_empty.json",
        "facts_b_mixed.json",
        "facts_c_unknown.json",
        "facts_d_operation.json",
    ] {
        let report = evaluate(facts);
        if let Err(errors) = compiled.validate(&report) {
            panic!(
                "{facts}: {:?}",
                errors.map(|e| e.to_string()).collect::<Vec<_>>()
            );
        };
    }
    let facts_schema: Value =
        serde_json::from_str(include_str!("../../schemas/evaluate-facts.v1.json")).unwrap();
    let compiled = jsonschema::JSONSchema::compile(&facts_schema).unwrap();
    for facts in [
        "facts_a_empty.json",
        "facts_b_mixed.json",
        "facts_d_operation.json",
    ] {
        let doc: Value =
            serde_json::from_str(&std::fs::read_to_string(fixture(facts)).unwrap()).unwrap();
        assert!(
            compiled.is_valid(&doc),
            "{facts} must satisfy the facts schema"
        );
    }
}

#[test]
fn evaluate_world_ref_matches_envelope_derivation() {
    let model = fixture("model.sea");
    let out = run_with(
        &model,
        &fixture("facts_a_empty.json"),
        &["--world-name", "probe"],
    );
    assert!(out.status.success());
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    let world_ref = report["world_ref"].as_str().unwrap().to_string();
    assert!(world_ref.starts_with("world:probe@sha256:"));

    let envelope = Command::cargo_bin("domainforge")
        .unwrap()
        .args(["envelope", "--emit", "cep", "--world-name", "probe"])
        .arg(&model)
        .output()
        .unwrap();
    let envelope: Value = serde_json::from_slice(&envelope.stdout).unwrap();
    assert_eq!(envelope["scope"]["world_ref"], world_ref);
}

#[test]
fn evaluate_is_deterministic_byte_for_byte() {
    let run = || run_with(&fixture("model.sea"), &fixture("facts_b_mixed.json"), &[]).stdout;
    assert_eq!(run(), run());
}

#[test]
fn evaluate_exits_zero_when_policies_are_false_or_unknown() {
    let out = run_with(
        &fixture("model.sea"),
        &fixture("facts_c_known_bad.json"),
        &[],
    );
    assert_eq!(out.status.code(), Some(0));
}

fn facts_error(facts: Value) -> String {
    let dir = tempdir().unwrap();
    let path = dir.path().join("facts.json");
    std::fs::write(&path, facts.to_string()).unwrap();
    let out = run_with(&fixture("model.sea"), &path, &[]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "must fail with usage/input exit code"
    );
    assert!(out.stdout.is_empty(), "no report on input errors");
    String::from_utf8_lossy(&out.stderr).to_string()
}

const SCHEMA: &str = "domainforge.evaluate-facts.v1";

#[test]
fn evaluate_rejects_unknown_entity_field_operation_and_bad_facts() {
    let err = facts_error(json!({"schema": SCHEMA, "instances": [
        {"namespace": "evaluate_fixture", "entity": "Nope", "name": "x", "fields": {}}]}));
    assert!(err.contains("Nope"), "{err}");

    let err = facts_error(json!({"schema": SCHEMA, "instances": [
        {"namespace": "evaluate_fixture", "entity": "Step", "name": "x",
         "fields": {"step_id": "x", "status": "executed", "bogus": 1}}]}));
    assert!(err.contains("bogus"), "{err}");

    let err = facts_error(json!({"schema": SCHEMA, "operation_inputs": [
        {"id": "i", "operation": "no_such_op", "input": {}}]}));
    assert!(err.contains("no_such_op"), "{err}");

    let err = facts_error(json!({"schema": SCHEMA, "operation_inputs": [
        {"id": "i", "operation": "close_case",
         "input": {"bogus": {"kind": "string", "data": "x"}}}]}));
    assert!(err.contains("bogus"), "{err}");

    let err = facts_error(json!({"schema": SCHEMA, "operation_inputs": [
        {"id": "i", "operation": "close_case", "input": {}},
        {"id": "i", "operation": "close_case", "input": {}}]}));
    assert!(err.contains("duplicate"), "{err}");

    let err = facts_error(json!({"schema": "something.else"}));
    assert!(err.contains("unsupported facts schema"), "{err}");

    let err = facts_error(json!({"schema": SCHEMA, "surprise": true}));
    assert!(err.contains("surprise"), "{err}");

    let err = facts_error(json!({"schema": SCHEMA, "instances": [
        {"namespace": "evaluate_fixture", "entity": "Step", "name": "x",
         "fields": {"step_id": "x"}}]}));
    assert!(err.contains("status"), "required field missing: {err}");
}

#[test]
fn evaluate_missing_files_are_usage_errors() {
    let out = run_with(&fixture("model.sea"), &fixture("does_not_exist.json"), &[]);
    assert_eq!(out.status.code(), Some(2));
}

/// Behaviour probes recorded in docs/reference/evaluate.md.
const PROBE_MODEL: &str = r#"
@namespace "evaluate_probe"

entity "Item" {
    key item_id: string (min_length 1)
    flag: bool optional
}

policy single_flagged per Constraint Obligation priority 1 as: exists_unique i in entity_instances of "Item": (i.flag = true)

policy always_true per Constraint Obligation priority 2 as: 1 = 1
"#;

#[test]
fn evaluate_probe_exists_unique_under_null_and_quantifier_free_policy() {
    let dir = tempdir().unwrap();
    let model = dir.path().join("probe.sea");
    std::fs::write(&model, PROBE_MODEL).unwrap();
    let facts = dir.path().join("facts.json");
    std::fs::write(
        &facts,
        json!({"schema": SCHEMA, "instances": [
            {"namespace": "evaluate_probe", "entity": "Item", "name": "a", "fields": {"item_id": "a", "flag": true}},
            {"namespace": "evaluate_probe", "entity": "Item", "name": "b", "fields": {"item_id": "b"}}]})
        .to_string(),
    )
    .unwrap();
    let out = run_with(&model, &facts, &[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    let p = policy(&report, "single_flagged");
    // Observed: the existing evaluator expands exists_unique eagerly and counts
    // only literal-true items, so one true plus one NULL yields true (a
    // three-valued reading would be unknown). The policy-level result is the
    // existing evaluator's, reported as-is; the binding rows show the NULL.
    assert_eq!(p["result"], "true");
    assert_eq!(
        results(p, "instance"),
        pairs(&[("a", "true"), ("b", "unknown")])
    );
    let q = policy(&report, "always_true");
    assert_eq!(q["result"], "true");
    assert!(q["support"].is_null());
    assert_eq!(q["bindings"], json!([]));
    assert!(q.get("binding_depth").is_none());
}

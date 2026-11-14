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
    let p = report["policies"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["policy"] == "case_must_be_open" && r["input_set_id"] == "in_open")
        .unwrap();
    assert_eq!(p["operation_bound"], true);
    assert_eq!(p["operation"], "close_case");
    assert_eq!(p["result"], "true");
    let rows: Vec<(String, String)> = report["policies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["policy"] == "case_must_be_open")
        .map(|r| {
            (
                r["input_set_id"].as_str().unwrap().to_string(),
                r["result"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        pairs(&[
            ("in_closed", "false"),
            ("in_missing", "unknown"),
            ("in_open", "true")
        ])
    );
    assert_eq!(
        report["policies"].as_array().unwrap().len(),
        5,
        "no aggregate row"
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

fn probe_report(model_extra: &str, items: &[(&str, Option<bool>)]) -> Value {
    let dir = tempdir().unwrap();
    let model = dir.path().join("probe.sea");
    std::fs::write(&model, format!("{PROBE_MODEL}{model_extra}")).unwrap();
    let facts = dir.path().join("facts.json");
    let instances: Vec<Value> = items
        .iter()
        .map(|(name, flag)| {
            let mut fields = json!({"item_id": name});
            if let Some(flag) = flag {
                fields["flag"] = json!(flag);
            }
            json!({"namespace": "evaluate_probe", "entity": "Item", "name": name, "fields": fields})
        })
        .collect();
    std::fs::write(
        &facts,
        json!({"schema": SCHEMA, "instances": instances}).to_string(),
    )
    .unwrap();
    let out = run_with(&model, &facts, &[]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
}

const PROBE_MODEL: &str = r#"
@namespace "evaluate_probe"

entity "Item" {
    key item_id: string (min_length 1)
    flag: bool optional
}

policy single_flagged per Constraint Obligation priority 1 as: exists_unique i in entity_instances of "Item": (i.flag = true)

policy always_true per Constraint Obligation priority 2 as: 1 = 1
"#;

fn unique(items: &[(&str, Option<bool>)]) -> Value {
    policy(&probe_report("", items), "single_flagged").clone()
}

#[test]
fn evaluate_exists_unique_true_plus_null_is_unknown() {
    let p = unique(&[("a", Some(true)), ("b", None)]);
    assert_eq!(p["result"], "unknown");
    assert!(p.get("caveat").is_none() && p.get("evaluator_result").is_none());
    assert_eq!(
        results(&p, "instance"),
        pairs(&[("a", "true"), ("b", "unknown")])
    );
}

#[test]
fn evaluate_exists_unique_null_plus_false_is_unknown() {
    let p = unique(&[("a", None), ("b", Some(false))]);
    assert_eq!(p["result"], "unknown");
}

#[test]
fn evaluate_exists_unique_two_trues_plus_null_is_false() {
    let p = unique(&[("a", Some(true)), ("b", Some(true)), ("c", None)]);
    assert_eq!(p["result"], "false");
}

#[test]
fn evaluate_exists_unique_null_only_is_unknown() {
    assert_eq!(unique(&[("a", None)])["result"], "unknown");
}

const NESTED_POLICY: &str = "\npolicy nested_single per Constraint Obligation priority 3 as: 1 = 1 and exists_unique i in entity_instances of \"Item\": (i.flag = true)\n";

fn nested(items: &[(&str, Option<bool>)]) -> String {
    let report = probe_report(NESTED_POLICY, items);
    policy(&report, "nested_single")["result"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn evaluate_nested_exists_unique_is_three_valued() {
    assert_eq!(nested(&[("a", Some(true)), ("b", None)]), "unknown");
    assert_eq!(
        nested(&[("a", Some(true)), ("b", Some(true)), ("c", None)]),
        "false"
    );
    assert_eq!(nested(&[("a", Some(true)), ("b", Some(false))]), "true");
    assert_eq!(nested(&[("a", None)]), "unknown");
}

#[test]
fn evaluate_exists_unique_true_plus_false_is_true() {
    let p = unique(&[("a", Some(true)), ("b", Some(false))]);
    assert_eq!(p["result"], "true");
    assert!(p.get("caveat").is_none());
}

#[test]
fn evaluate_quantifier_free_policy_has_no_support_or_bindings() {
    let report = probe_report("", &[("a", Some(true))]);
    let q = policy(&report, "always_true");
    assert_eq!(q["result"], "true");
    assert!(q["support"].is_null());
    assert_eq!(q["bindings"], json!([]));
    assert!(q.get("binding_depth").is_none());
}

#[test]
fn evaluate_marks_model_declared_instances_and_counts_sources() {
    let report = probe_report(
        "\ninstance declared of \"Item\" {\n    item_id: \"declared\"\n    flag: true\n}\n",
        &[("from_facts", Some(false))],
    );
    assert_eq!(report["model_instance_count"], 1);
    assert_eq!(report["facts_instance_count"], 1);
    let p = policy(&report, "single_flagged");
    let sources: Vec<(String, String)> = p["bindings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|b| {
            (
                b["instance"].as_str().unwrap().to_string(),
                b["source"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        sources,
        pairs(&[("declared", "model"), ("from_facts", "facts")])
    );
}

fn eval_text(model: &str, facts: Value) -> std::process::Output {
    let dir = tempdir().unwrap();
    let m = dir.path().join("m.sea");
    std::fs::write(&m, model).unwrap();
    let f = dir.path().join("f.json");
    std::fs::write(&f, facts.to_string()).unwrap();
    run_with(&m, &f, &[])
}

fn base_model() -> String {
    std::fs::read_to_string(fixture("model.sea")).unwrap()
}

#[test]
fn evaluate_rejects_inputs_for_operation_without_policy_governance() {
    let model = base_model().replace(
        "access policy_governed by case_must_be_open at precondition fails with case_not_open",
        "access public",
    );
    let out = eval_text(
        &model,
        json!({"schema": SCHEMA, "operation_inputs": [{"id": "i", "operation": "close_case", "input": {}}]}),
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("close_case"));
}

#[test]
fn evaluate_rejects_operation_input_value_of_the_wrong_kind() {
    let model = base_model();
    let out = eval_text(
        &model,
        json!({"schema": SCHEMA, "operation_inputs": [{"id": "i", "operation": "close_case",
            "input": {"case_id": {"kind": "string", "data": "5"}}}]}),
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("case_id"));
    // string value for the string field is accepted
    let out = eval_text(
        &model,
        json!({"schema": SCHEMA, "operation_inputs": [{"id": "i", "operation": "close_case",
            "input": {"case_status": {"kind": "string", "data": "5"}}}]}),
    );
    assert_eq!(out.status.code(), Some(0));
}

#[test]
fn evaluate_rejects_int_field_given_a_string() {
    let model = base_model().replace(
        "case_status: string\n}",
        "case_status: string\n    attempts: int\n}",
    );
    let out = eval_text(
        &model,
        json!({"schema": SCHEMA, "operation_inputs": [{"id": "i", "operation": "close_case",
            "input": {"attempts": {"kind": "string", "data": "5"}}}]}),
    );
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("attempts"));
}

#[test]
fn evaluate_policy_bound_to_several_operations_yields_a_row_per_operation_and_input() {
    let base = base_model();
    let start = base.find("operation close_case {").unwrap();
    let second = base[start..]
        .replace("operation close_case {", "operation close_case_b {")
        .replace("case_not_open", "case_not_open_b")
        .replace("failure conflict", "failure conflict_b");
    let model = format!("{base}\n{second}");
    let input = |id: &str, op: &str, status: &str| json!({"id": id, "operation": op, "input": {"case_status": {"kind": "string", "data": status}}});
    let out = eval_text(
        &model,
        json!({"schema": SCHEMA, "operation_inputs": [
            input("x1", "close_case", "open"), input("x2", "close_case_b", "closed")]}),
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let report: Value = serde_json::from_slice(&out.stdout).unwrap();
    let rows: Vec<(String, String, String)> = report["policies"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["policy"] == "case_must_be_open")
        .map(|r| {
            (
                r["operation"].as_str().unwrap().to_string(),
                r["input_set_id"].as_str().unwrap_or("-").to_string(),
                r["result"].as_str().unwrap().to_string(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("close_case".into(), "x1".into(), "true".into()),
            ("close_case_b".into(), "x2".into(), "false".into())
        ]
    );
}

const REF_MODEL: &str = r#"
@namespace "evaluate_refs"

entity "Owner" {
    key owner_id: string (min_length 1)
}

entity "Pet" {
    key pet_id: string (min_length 1)
    owner: ref<Owner> optional
}

policy always_true per Constraint Obligation priority 1 as: forall p in entity_instances of "Pet": (1 = 1)
"#;

fn ref_fact(entity: &str, name: &str, owner: Option<&str>) -> Value {
    let key = if entity == "Owner" {
        "owner_id"
    } else {
        "pet_id"
    };
    let mut fields = json!({ key: name });
    if let Some(owner) = owner {
        fields["owner"] = json!(owner);
    }
    json!({"namespace": "evaluate_refs", "entity": entity, "name": name, "fields": fields})
}

#[test]
fn evaluate_facts_are_order_independent_for_entity_references() {
    let forward = json!({"schema": SCHEMA, "instances": [
        ref_fact("Pet", "rex", Some("ann")), ref_fact("Owner", "ann", None)]});
    let reverse = json!({"schema": SCHEMA, "instances": [
        ref_fact("Owner", "ann", None), ref_fact("Pet", "rex", Some("ann"))]});
    let a = eval_text(REF_MODEL, forward);
    let b = eval_text(REF_MODEL, reverse);
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    // facts_sha256 hashes the raw file bytes, so compare the evaluated results.
    let (a, b): (Value, Value) = (
        serde_json::from_slice(&a.stdout).unwrap(),
        serde_json::from_slice(&b.stdout).unwrap(),
    );
    assert_eq!(a["policies"], b["policies"]);
    assert_eq!(a["facts_instance_count"], b["facts_instance_count"]);
}

#[test]
fn evaluate_dangling_entity_reference_is_still_an_input_error() {
    let out = eval_text(
        REF_MODEL,
        json!({"schema": SCHEMA, "instances": [
            ref_fact("Owner", "ann", None), ref_fact("Pet", "rex", Some("nobody"))]}),
    );
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("dangling"));
}

#[test]
fn evaluate_rejects_an_operation_name_declared_in_two_namespaces() {
    let dir = tempdir().unwrap();
    let base = base_model();
    // The second module re-declares every operation under a second namespace;
    // failure codes must stay unique across the closure.
    let second = base
        .replace("evaluate_fixture", "evaluate_fixture_two")
        .replace("case_not_open", "case_not_open_two")
        .replace("failure conflict for", "failure conflict_two for")
        .replace("entity \"Case\"", "export entity \"Case\"");
    let first = base.replacen(
        "@namespace \"evaluate_fixture\"\n",
        "@namespace \"evaluate_fixture\"\nimport { Case as OtherCase } from \"./b.sea\"\n",
        1,
    );
    std::fs::write(dir.path().join("a.sea"), first).unwrap();
    std::fs::write(dir.path().join("b.sea"), second).unwrap();
    let facts = dir.path().join("f.json");
    std::fs::write(
        &facts,
        json!({"schema": SCHEMA, "operation_inputs": [{"id": "i", "operation": "close_case", "input": {}}]})
            .to_string(),
    )
    .unwrap();
    let out = run_with(&dir.path().join("a.sea"), &facts, &[]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr)
        .contains("operation input 'i': ambiguous operation 'close_case'"));
}

fn enum_run(supplied_symbol: &str) -> std::process::Output {
    let dir = tempdir().unwrap();
    let model = std::fs::read_to_string(fixture("model.sea"))
        .unwrap()
        .replace(
            "record CloseCaseInput {\n",
            "enum KindA { a = \"a\" }\nenum KindB { b = \"b\" }\n\nrecord CloseCaseInput {\n    kind: KindA\n",
        );
    let model_path = dir.path().join("model.sea");
    std::fs::write(&model_path, model).unwrap();
    let facts_path = dir.path().join("facts.json");
    std::fs::write(
        &facts_path,
        json!({"schema": SCHEMA, "operation_inputs": [
            {"id": "i", "operation": "close_case", "input": {
                "case_status": {"kind": "string", "data": "open"},
                "kind": {"kind": "enum", "data": {"symbol": supplied_symbol, "wire": "a"}}}}]})
        .to_string(),
    )
    .unwrap();
    run_with(&model_path, &facts_path, &[])
}

#[test]
fn evaluate_enum_input_must_carry_the_declared_symbol() {
    let ok = enum_run("evaluate_fixture.enum.KindA");
    assert_eq!(
        ok.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
    let bad = enum_run("evaluate_fixture.enum.KindB");
    assert_eq!(bad.status.code(), Some(2));
    assert!(bad.stdout.is_empty());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("field 'kind'"));
}

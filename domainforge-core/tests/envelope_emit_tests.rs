//! Slice 0B: `domainforge envelope` CEP emission (AC-DF-001/002/003 per the
//! frozen runtime spec §25).

#![cfg(feature = "cli")]

use domainforge_core::application::envelope::build_cep_envelope;
use domainforge_core::application::envelope::CepEnvelopeParams;
use serde_json::Value;
use std::path::PathBuf;
use std::process::{Command, Output};

const VALID_MODELS: [&str; 5] = [
    "domainforge-core/tests/fixtures/envelope/valid-basic.sea",
    "domainforge-core/tests/fixtures/envelope/valid-system.sea",
    "domainforge-core/tests/fixtures/envelope/valid-evolution.sea",
    "domainforge-core/tests/fixtures/envelope/valid-resources.sea",
    "domainforge-core/tests/fixtures/envelope/valid-declarations.sea",
];
const INVALID_MODEL: &str = "domainforge-core/tests/fixtures/envelope/invalid-syntax.sea";

fn fixture(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(path)
}

fn domainforge(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_domainforge"))
        .args(args)
        .output()
        .expect("domainforge binary runs")
}

fn emit_cep(path: &str) -> (std::process::ExitStatus, Value) {
    let output = domainforge(&[
        "envelope",
        "--emit",
        "cep",
        &fixture(path).to_string_lossy(),
    ]);
    let envelope: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("stdout is not a JSON envelope: {e}"));
    (output.status, envelope)
}

fn assert_hash_field(value: &Value) {
    let text = value.as_str().expect("string field");
    let mut parts = text.split(':');
    assert!(parts.next().is_some_and(|p| !p.is_empty()), "prefix");
    let hex = parts.next().expect("hex after prefix");
    assert_eq!(hex.len(), 64, "sha256 hex length");
    assert!(hex.chars().all(|c| c.is_ascii_hexdigit()), "hex digits");
}

fn assert_envelope_shape(envelope: &Value) {
    let object = envelope.as_object().expect("envelope is an object");
    for required in [
        "envelope_id",
        "cep_version",
        "envelope_version",
        "envelope_kind",
        "created_at",
        "created_by",
        "scope",
        "boundary_record",
        "completeness_status",
        "provenance_refs",
        "omission_status",
        "lineage_refs",
    ] {
        assert!(object.contains_key(required), "missing {required}");
    }
    assert_eq!(envelope["envelope_kind"], "semantic_snapshot");
    assert_eq!(envelope["envelope_version"], "1.0.0");
    assert!(!envelope["provenance_refs"].as_array().unwrap().is_empty());
    let boundary = envelope["boundary_record"].as_object().unwrap();
    for closed in [
        "scope",
        "included_sections",
        "excluded_sections",
        "known_omissions",
        "unknowns",
        "redactions",
        "compression_notes",
        "out_of_scope_entities",
        "limitations",
    ] {
        assert!(boundary.contains_key(closed), "boundary_record.{closed}");
    }
    assert!(envelope["boundary_record"]["known_omissions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o == "source_comments"));
    assert!(!envelope["provenance"].as_array().unwrap().is_empty());
    assert!(!envelope["references"].as_array().unwrap().is_empty());
}

#[test]
fn ac_df_001_envelope_shape_matches_canonical_contract() {
    for model in VALID_MODELS {
        let (status, envelope) = emit_cep(model);
        assert!(status.success(), "{model} must exit 0");
        assert_envelope_shape(&envelope);
        assert_eq!(envelope["validation_status"], "validated");
        assert!(!envelope
            .as_object()
            .unwrap()
            .contains_key("conformance_status"));
        let representations = envelope["representations"].as_array().unwrap();
        assert_eq!(representations.len(), 1);
        let representation = &representations[0];
        assert_eq!(representation["validation_status"], "valid");
        assert_hash_field(&representation["content_hash"]);
        assert_hash_field(&representation["semantic_hash"]);
        assert_eq!(representation["content_availability"], "available");
        assert!(representation["omission_record"]["known_omissions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|o| o == "source_formatting"));
        assert!(representation["preserved_distinctions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|d| d == "declared_vs_observed"));
    }
}

#[test]
fn ac_df_002_outer_variance_inner_hash_stability() {
    for model in VALID_MODELS {
        let mut ids = Vec::new();
        let mut hashes = Vec::new();
        for _ in 0..3 {
            let (status, envelope) = emit_cep(model);
            assert!(status.success());
            ids.push(envelope["envelope_id"].as_str().unwrap().to_string());
            hashes.push((
                envelope["representations"][0]["content_hash"]
                    .as_str()
                    .unwrap()
                    .to_string(),
                envelope["representations"][0]["semantic_hash"]
                    .as_str()
                    .unwrap()
                    .to_string(),
            ));
        }
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 3, "{model}: envelope_id must vary per emission");
        hashes.sort();
        hashes.dedup();
        assert_eq!(
            hashes.len(),
            1,
            "{model}: content/semantic hashes must be stable"
        );
    }
}

#[test]
fn ac_df_002_representation_emit_is_byte_identical() {
    for model in VALID_MODELS {
        let first = domainforge(&[
            "envelope",
            "--emit",
            "representation",
            &fixture(model).to_string_lossy(),
        ]);
        let second = domainforge(&[
            "envelope",
            "--emit",
            "representation",
            &fixture(model).to_string_lossy(),
        ]);
        assert!(first.status.success(), "{model}: first emit");
        assert!(second.status.success(), "{model}: second emit");
        assert_eq!(first.stdout, second.stdout, "{model}: byte-identical");
        let document: Value = serde_json::from_slice(&first.stdout).unwrap();
        assert_eq!(
            document["schema_version"],
            "domainforge-semantic-envelope/v1"
        );
        assert_hash_field(&document["self_hash"]);
    }
}

#[test]
fn ac_df_002_inner_hashes_match_representation_document() {
    for model in VALID_MODELS {
        let output = domainforge(&[
            "envelope",
            "--emit",
            "representation",
            &fixture(model).to_string_lossy(),
        ]);
        let document: Value = serde_json::from_slice(&output.stdout).unwrap();
        let (_, envelope) = emit_cep(model);
        assert_eq!(
            envelope["representations"][0]["content_hash"], document["self_hash"],
            "{model}: content_hash must equal document self_hash"
        );
        assert_eq!(
            envelope["representations"][0]["semantic_hash"], document["semantic_closure_hash"],
            "{model}: semantic_hash must equal closure hash"
        );
        assert_eq!(
            envelope["representations"][0]["representation_id"],
            format!("representation:{}", document["self_hash"].as_str().unwrap())
        );
    }
}

#[test]
fn ac_df_003_invalid_model_still_emits_valid_envelope() {
    let (status, envelope) = emit_cep(INVALID_MODEL);
    assert_eq!(status.code(), Some(1), "invalid model must exit non-zero");
    assert_envelope_shape(&envelope);
    assert_eq!(envelope["conformance_status"], "non_conformant");
    assert!(
        !envelope
            .as_object()
            .unwrap()
            .contains_key("validation_status"),
        "no envelope validation_status"
    );
    assert!(
        !envelope
            .as_object()
            .unwrap()
            .contains_key("representations"),
        "no fabricated representations"
    );
    let omissions = envelope["omissions"].as_array().unwrap();
    assert_eq!(omissions.len(), 1);
    assert_eq!(omissions[0]["omission_type"], "representation_unavailable");
    assert_eq!(omissions[0]["known_or_suspected"], "known");
    assert!(omissions[0]["affected_entities"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e.as_str().unwrap().ends_with("invalid-syntax.sea")));
    let extensions = &envelope["extensions"]["domainforge"];
    assert_eq!(extensions["model_validation_status"], "invalid");
    assert_hash_field(&extensions["invalid_declared_checkpoint_hash"]);
    assert!(!extensions["diagnostics"].as_array().unwrap().is_empty());
    assert!(envelope["boundary_record"]["known_omissions"]
        .as_array()
        .unwrap()
        .iter()
        .any(|o| o == "canonical_representation_D"));

    fn walk(value: &Value, found: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, child) in map {
                    if key == "semantic_hash" {
                        found.push(child.to_string());
                    }
                    walk(child, found);
                }
            }
            Value::Array(items) => items.iter().for_each(|item| walk(item, found)),
            _ => {}
        }
    }
    let mut semantic_hashes = Vec::new();
    walk(&envelope, &mut semantic_hashes);
    assert!(
        semantic_hashes.is_empty(),
        "no fabricated semantic hash: {semantic_hashes:?}"
    );
}

#[test]
fn ac_df_003_checkpoint_hash_is_stable_across_runs() {
    let first = emit_cep(INVALID_MODEL);
    let second = emit_cep(INVALID_MODEL);
    assert_eq!(first.0.code(), Some(1));
    assert_eq!(second.0.code(), Some(1));
    assert_eq!(
        first.1["extensions"]["domainforge"]["invalid_declared_checkpoint_hash"],
        second.1["extensions"]["domainforge"]["invalid_declared_checkpoint_hash"]
    );
    assert_ne!(first.1["envelope_id"], second.1["envelope_id"]);
}

#[test]
fn ac_df_003_representation_mode_on_invalid_model_fails_without_output() {
    let output = domainforge(&[
        "envelope",
        "--emit",
        "representation",
        &fixture(INVALID_MODEL).to_string_lossy(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
}

#[test]
fn scope_flag_replaces_default_scope() {
    let output = domainforge(&[
        "envelope",
        "--emit",
        "cep",
        "--scope",
        r#"{"repo_id":"sxr-test","model_ref":"m-1"}"#,
        &fixture(VALID_MODELS[0]).to_string_lossy(),
    ]);
    assert!(output.status.success());
    let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(envelope["scope"]["repo_id"], "sxr-test");
    assert_eq!(envelope["scope"]["model_ref"], "m-1");
    assert!(!envelope["scope"]
        .as_object()
        .unwrap()
        .contains_key("entry_logical_path"));
}

#[test]
fn invalid_scope_flag_exits_two() {
    let output = domainforge(&[
        "envelope",
        "--emit",
        "cep",
        "--scope",
        "not json",
        &fixture(VALID_MODELS[0]).to_string_lossy(),
    ]);
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn build_cep_envelope_is_pure_and_deterministic() {
    let diagnostics = vec!["test diagnostic".to_string()];
    fn params<'a>(
        envelope_id: &'a str,
        created_at: &'a str,
        diagnostics: &'a [String],
    ) -> CepEnvelopeParams<'a> {
        CepEnvelopeParams {
            doc: None,
            model_valid: false,
            source_set_hash:
                "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            invalid_declared_checkpoint_hash: Some(
                "sha256:2222222222222222222222222222222222222222222222222222222222222222",
            ),
            diagnostics,
            scope: serde_json::json!({"model_ref": "m-1"}),
            entry_logical_path: "logical/entry.sea",
            inline_threshold_bytes: 65_536,
            envelope_id,
            created_at,
            registry_content_hash: None,
            resolved_namespaces: &[],
        }
    }
    let first = build_cep_envelope(&params("id-1", "2026-08-07T00:00:00Z", &diagnostics));
    let second = build_cep_envelope(&params("id-1", "2026-08-07T00:00:00Z", &diagnostics));
    assert_eq!(
        first, second,
        "pinned inputs must produce identical envelopes"
    );
    let varied = build_cep_envelope(&params("id-2", "2026-08-07T00:00:01Z", &diagnostics));
    assert_ne!(
        first, varied,
        "envelope_id/created_at must be the only varying fields"
    );
}

// ── Slice 2: verification-contract projection (§14.0) ─────────────────────

fn emit_verification_contract(path: &str) -> (std::process::ExitStatus, Value) {
    let output = domainforge(&[
        "envelope",
        "--emit",
        "verification-contract",
        &fixture(path).to_string_lossy(),
    ]);
    let envelope: Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|e| panic!("stdout is not a JSON envelope: {e}"));
    (output.status, envelope)
}

#[test]
fn verification_contract_is_a_valid_work_request() {
    for model in VALID_MODELS {
        let (status, envelope) = emit_verification_contract(model);
        assert!(status.success(), "{model}: verification-contract exit 0");
        assert_eq!(envelope["envelope_kind"], "work_request");
        assert_eq!(envelope["validation_status"], "valid");
        assert_eq!(
            envelope["extensions"]["domainforge"]["profile"],
            "domainforge-semantic-verification/v1"
        );
        let questions = envelope["questions"].as_array().unwrap();
        let obligations = envelope["extensions"]["domainforge"]["obligations"]
            .as_array()
            .unwrap();
        assert_eq!(questions.len(), obligations.len());
        assert_eq!(
            envelope["constraints"].as_array().unwrap().len(),
            obligations.len()
        );
        for obligation in obligations {
            for field in [
                "obligation_id",
                "decl_key",
                "claim_id",
                "question_id",
                "question_form",
                "expected_answer_shape",
                "evidence_types",
                "projection_artifact_refs",
                "limitations",
            ] {
                assert!(
                    obligation.as_object().unwrap().contains_key(field),
                    "{field}"
                );
            }
            assert!(!obligation["evidence_types"].as_array().unwrap().is_empty());
        }
    }
}

#[test]
fn verification_contract_outer_varies_inner_stable() {
    for model in VALID_MODELS {
        let (_, first) = emit_verification_contract(model);
        let (_, second) = emit_verification_contract(model);
        assert_ne!(first["envelope_id"], second["envelope_id"]);
        let ob_first = first["extensions"]["domainforge"]["obligations"].clone();
        let ob_second = second["extensions"]["domainforge"]["obligations"].clone();
        assert_eq!(ob_first, ob_second, "{model}: obligations byte-stable");
        assert_eq!(
            first["extensions"]["domainforge"]["semantic_hash"],
            second["extensions"]["domainforge"]["semantic_hash"]
        );
    }
}

#[test]
fn verification_contract_requires_valid_model() {
    let output = domainforge(&[
        "envelope",
        "--emit",
        "verification-contract",
        &fixture(INVALID_MODEL).to_string_lossy(),
    ]);
    assert_ne!(output.status.code(), Some(0));
    // A syntax-invalid model takes the §14.4 failure path (emits the failed
    // snapshot envelope, not a work_request); either way it is not a success.
    if !output.stdout.is_empty() {
        let envelope: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_ne!(envelope["envelope_kind"], "work_request");
    }
}

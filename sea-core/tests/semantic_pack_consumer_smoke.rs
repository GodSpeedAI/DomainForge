use sea_core::semantic_pack::*;

fn make_concept(
    id: &str,
    canonical_name: &str,
) -> ConceptDef {
    ConceptDef {
        id: id.to_string(),
        canonical_name: canonical_name.to_string(),
        kind: ConceptKind::Entity,
        status: ConceptStatus::Active,
        definition: ConceptDefinition {
            text: format!("Definition of {}", canonical_name),
            definition_hash: String::new(),
            decision_ref: format!("dec_{}", id),
        },
        owner: "owner@test.com".to_string(),
        source_refs: vec![],
        examples: vec![],
        counterexamples: vec![],
        allowed_predicates: vec![],
        valid_contexts: vec![],
    }
}

fn make_valid_pack() -> SemanticPack {
    SemanticPack {
        schema_version: "0.3".to_string(),
        pack_id: "test-org/test-domain/1.0.0".to_string(),
        org_id: "test-org".to_string(),
        domain_id: "test-domain".to_string(),
        pack_version: "1.0.0".to_string(),
        meaning_version: "1.0.0".to_string(),
        meaning_fingerprint: String::new(),
        source_graph_hash: "sha256:test".to_string(),
        build_config_hash: "sha256:cfg".to_string(),
        review_manifest_hash: "sha256:rev".to_string(),
        created_at: "2026-06-07T00:00:00Z".to_string(),
        generator: schema::GeneratorInfo {
            name: "sea-core".to_string(),
            version: "0.3".to_string(),
        },
        trust: schema::PackTrust {
            approval_state: ApprovalState::Candidate,
            signature_state: SignatureState::Unsigned,
            signed_by: None,
            signature_alg: None,
            signature: None,
        },
        concepts: vec![make_concept("supplier", "Supplier")],
        relations: vec![],
        metrics: vec![],
        dimensions: vec![],
        units: vec![],
        aliases: vec![],
        mapping_rules: vec![],
        compatibility: schema::CompatibilityInfo::default(),
    }
}

fn make_invalid_pack() -> SemanticPack {
    let mut pack = make_valid_pack();
    pack.schema_version = "invalid-version".to_string();
    pack.concepts.push(ConceptDef {
        id: "supplier".to_string(),
        canonical_name: "Supplier".to_string(),
        kind: ConceptKind::Entity,
        status: ConceptStatus::Active,
        definition: ConceptDefinition {
            text: "Duplicate".to_string(),
            definition_hash: String::new(),
            decision_ref: "dec_dup".to_string(),
        },
        owner: "owner@test.com".to_string(),
        source_refs: vec![],
        examples: vec![],
        counterexamples: vec![],
        allowed_predicates: vec![],
        valid_contexts: vec![],
    });
    pack
}

#[test]
fn consumer_smoke_valid_pack_passes() {
    let pack = make_valid_pack();
    let options = ValidationOptions::default();
    let result = validate_graph_with_pack(&pack, "test://source", &options);

    let json = serde_json::to_string(&result).expect("serialize result");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("re-parse json");

    let status = parsed.get("status").and_then(|v| v.as_str()).unwrap_or("");
    assert_ne!(status, "failed");
}

#[test]
fn consumer_smoke_invalid_pack_fails() {
    let pack = make_invalid_pack();
    let options = ValidationOptions::default();
    let result = validate_graph_with_pack(&pack, "test://source", &options);

    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == SemanticDiagnosticCode::DuplicateConceptId
            || d.code == SemanticDiagnosticCode::PackSchemaMismatch));
}

#[test]
fn consumer_smoke_rejects_failing_validation_json() {
    let pack = make_invalid_pack();
    let options = ValidationOptions::default();
    let result = validate_graph_with_pack(&pack, "test://source", &options);

    let json = serde_json::to_string(&result).expect("serialize result");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("re-parse json");

    let diags = parsed.get("diagnostics").and_then(|v| v.as_array()).unwrap();
    assert!(!diags.is_empty());

    let has_error = diags.iter().any(|d| {
        d.get("severity").and_then(|v| v.as_str()) == Some("error")
    });
    assert!(has_error);
}

#[test]
fn consumer_smoke_strict_mode_unsigned_pack_blocked() {
    let pack = make_valid_pack();
    let options = ValidationOptions::strict();
    let result = validate_graph_with_pack(&pack, "test://source", &options);

    assert_eq!(result.status, SemanticValidationStatus::Blocked);
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == SemanticDiagnosticCode::PackUnsigned));
}

#[test]
fn consumer_smoke_unsigned_bypass_allows_pack() {
    let pack = make_valid_pack();
    let mut options = ValidationOptions::strict();
    options.allow_unsigned_test_fixtures = true;
    let result = validate_graph_with_pack(&pack, "test://source", &options);

    assert!(result.unsigned_fixture_bypass_used);
    assert_ne!(result.status, SemanticValidationStatus::Blocked);
}

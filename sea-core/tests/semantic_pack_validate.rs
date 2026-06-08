use sea_core::semantic_pack::*;

fn make_concept(
    id: &str,
    canonical_name: &str,
    status: ConceptStatus,
    definition_text: &str,
    owner: &str,
) -> ConceptDef {
    ConceptDef {
        id: id.to_string(),
        canonical_name: canonical_name.to_string(),
        kind: ConceptKind::Entity,
        status,
        definition: ConceptDefinition {
            text: definition_text.to_string(),
            definition_hash: String::new(),
            decision_ref: format!("dec_{}", id),
        },
        owner: owner.to_string(),
        source_refs: vec![],
        examples: vec![],
        counterexamples: vec![],
        allowed_predicates: vec![],
        valid_contexts: vec![],
    }
}

fn make_review_record(subject_id: &str, decision: ReviewDecision, def_hash: &str) -> ReviewRecord {
    ReviewRecord {
        decision_id: format!("dec_{}", subject_id),
        subject_type: "concept".to_string(),
        subject_id: subject_id.to_string(),
        decision,
        rationale: format!("Review for {}", subject_id),
        reviewer: "reviewer@test.com".to_string(),
        reviewed_at: "2026-06-07T00:00:00Z".to_string(),
        definition_hash: def_hash.to_string(),
        previous_definition_hash: None,
        new_definition_hash: None,
    }
}

fn make_minimal_pack() -> SemanticPack {
    let supplier = make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
        "A party that provides goods or services",
        "owner@test.com",
    );

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
        concepts: vec![supplier],
        relations: vec![],
        metrics: vec![],
        dimensions: vec![],
        units: vec![],
        aliases: vec![],
        mapping_rules: vec![],
        compatibility: schema::CompatibilityInfo::default(),
    }
}

#[test]
fn definition_hash_must_match_review_manifest() {
    let mut input = PackBuildInput {
        org_id: "test-org".to_string(),
        domain_id: "test-domain".to_string(),
        pack_version: "1.0.0".to_string(),
        meaning_version: "1.0.0".to_string(),
        approval: ApprovalState::Approved,
        concepts: vec![make_concept(
            "supplier",
            "Supplier",
            ConceptStatus::Active,
            "A party that provides goods or services",
            "owner@test.com",
        )],
        relations: vec![],
        metrics: vec![],
        dimensions: vec![],
        units: vec![],
        aliases: vec![],
        mapping_rules: vec![],
        review_records: vec![make_review_record(
            "supplier",
            ReviewDecision::Approve,
            "sha256:WRONG_HASH",
        )],
        previous_pack: None,
        allow_first_approved_version: true,
        source_graph_hash: "sha256:test".to_string(),
    };

    input.concepts[0].definition.definition_hash = "sha256:ACTUAL_HASH".to_string();

    let result = build_semantic_pack(input);
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(diags
        .iter()
        .any(|d| d.code == SemanticDiagnosticCode::UnreviewedConcept));
}

#[test]
fn rejected_concept_is_invalid_not_unknown() {
    let mut pack = make_minimal_pack();
    pack.concepts[0].status = ConceptStatus::Rejected;

    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://policy");
    let diag = validate_term("supplier", &pack, &options, source_ref);

    assert!(diag.is_some());
    let d = diag.unwrap();
    assert_eq!(d.semantic_truth, SemanticTruth::Invalid);
    assert_eq!(d.code, SemanticDiagnosticCode::RejectedConcept);
}

#[test]
fn deprecated_policy_error_in_strict_preserves_valid_truth() {
    let mut pack = make_minimal_pack();
    pack.concepts[0].status = ConceptStatus::Deprecated;

    let mut options = ValidationOptions::strict();
    options.deprecated_policy = DeprecatedPolicy::ErrorInStrict;

    let source_ref = SourceRef::synthetic("test://policy");
    let diag = validate_term("supplier", &pack, &options, source_ref);

    assert!(diag.is_some());
    let d = diag.unwrap();
    assert_eq!(d.semantic_truth, SemanticTruth::Valid);
    assert_eq!(d.code, SemanticDiagnosticCode::DeprecatedConcept);
    assert_eq!(d.severity, DiagnosticSeverity::Error);
}

#[test]
fn unknown_truth_survives_error_severity() {
    let pack = make_minimal_pack();

    let options = ValidationOptions {
        unknown_concept_policy: UnknownConceptPolicy::Error,
        ..ValidationOptions::default()
    };

    let source_ref = SourceRef::synthetic("test://policy");
    let diag = validate_term("nonexistent_concept", &pack, &options, source_ref);

    assert!(diag.is_some());
    let d = diag.unwrap();
    assert_eq!(d.semantic_truth, SemanticTruth::Unknown);
    assert_eq!(d.severity, DiagnosticSeverity::Error);
}

#[test]
fn expected_hash_mismatch_blocks_pack_load() {
    let pack = make_minimal_pack();
    let options = ValidationOptions {
        mode: ValidationMode::Strict,
        unknown_concept_policy: UnknownConceptPolicy::Error,
        deprecated_policy: DeprecatedPolicy::ErrorInStrict,
        require_signed_pack: true,
        allow_unsigned_test_fixtures: false,
    };

    let result = validate_graph_with_pack(&pack, "test://source", &options);
    assert_eq!(result.status, SemanticValidationStatus::Blocked);
    assert!(result
        .diagnostics
        .iter()
        .any(|d| d.code == SemanticDiagnosticCode::PackUnsigned));
}

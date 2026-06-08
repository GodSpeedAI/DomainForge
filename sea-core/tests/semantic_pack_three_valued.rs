use sea_core::semantic_pack::*;

fn make_concept(id: &str, canonical_name: &str, status: ConceptStatus) -> ConceptDef {
    ConceptDef {
        id: id.to_string(),
        canonical_name: canonical_name.to_string(),
        kind: ConceptKind::Entity,
        status,
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

fn make_pack_with_concepts(concepts: Vec<ConceptDef>) -> SemanticPack {
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
        concepts,
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
fn valid_concept_is_valid() {
    let pack = make_pack_with_concepts(vec![make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
    )]);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "supplier",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    assert_eq!(result.semantic_truth, SemanticTruth::Valid);
    assert_eq!(result.resolved_concept_id, Some("supplier".to_string()));
    assert!(result.diagnostic_code.is_none());
}

#[test]
fn unknown_concept_is_unknown() {
    let pack = make_pack_with_concepts(vec![make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
    )]);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "nonexistent",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    assert_eq!(result.semantic_truth, SemanticTruth::Unknown);
    assert_eq!(
        result.diagnostic_code,
        Some(SemanticDiagnosticCode::UnknownConcept)
    );
    assert!(result.resolved_concept_id.is_none());
}

#[test]
fn rejected_concept_is_invalid() {
    let pack = make_pack_with_concepts(vec![make_concept(
        "bad_thing",
        "BadThing",
        ConceptStatus::Rejected,
    )]);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "bad_thing",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    assert_eq!(result.semantic_truth, SemanticTruth::Invalid);
    assert_eq!(
        result.diagnostic_code,
        Some(SemanticDiagnosticCode::RejectedConcept)
    );
}

#[test]
fn proposed_concept_is_unknown() {
    let pack = make_pack_with_concepts(vec![make_concept(
        "draft_thing",
        "DraftThing",
        ConceptStatus::Proposed,
    )]);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "draft_thing",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    assert_eq!(result.semantic_truth, SemanticTruth::Unknown);
    assert_eq!(
        result.diagnostic_code,
        Some(SemanticDiagnosticCode::ProposedConcept)
    );
    assert_eq!(result.resolved_concept_id, Some("draft_thing".to_string()));
}

#[test]
fn deprecated_concept_is_valid_with_diagnostic() {
    let pack = make_pack_with_concepts(vec![make_concept(
        "old_thing",
        "OldThing",
        ConceptStatus::Deprecated,
    )]);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "old_thing",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    assert_eq!(result.semantic_truth, SemanticTruth::Valid);
    assert_eq!(
        result.diagnostic_code,
        Some(SemanticDiagnosticCode::DeprecatedConcept)
    );
    assert_eq!(result.resolved_concept_id, Some("old_thing".to_string()));
}

#[test]
fn canonical_name_resolution_works() {
    let pack = make_pack_with_concepts(vec![make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
    )]);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "Supplier",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    assert_eq!(result.semantic_truth, SemanticTruth::Valid);
    assert_eq!(result.resolved_concept_id, Some("supplier".to_string()));
}

#[test]
fn resolution_is_case_insensitive() {
    let pack = make_pack_with_concepts(vec![make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
    )]);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "SUPPLIER",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    assert_eq!(result.semantic_truth, SemanticTruth::Valid);
    assert_eq!(result.resolved_concept_id, Some("supplier".to_string()));
}

#[test]
fn unknown_severity_depends_on_policy() {
    let pack = make_pack_with_concepts(vec![]);

    let mut options_warn = ValidationOptions::default();
    options_warn.unknown_concept_policy = UnknownConceptPolicy::Warning;
    let source_ref1 = SourceRef::synthetic("test://term");
    let request1 = ResolveRequest {
        raw_text: "unknown",
        expected_kind: None,
        source_ref: source_ref1,
    };
    let result_warn = resolve_concept(&request1, &pack, &options_warn);
    assert_eq!(result_warn.diagnostic_severity, DiagnosticSeverity::Warning);

    let mut options_error = ValidationOptions::default();
    options_error.unknown_concept_policy = UnknownConceptPolicy::Error;
    let source_ref2 = SourceRef::synthetic("test://term");
    let request2 = ResolveRequest {
        raw_text: "unknown",
        expected_kind: None,
        source_ref: source_ref2,
    };
    let result_error = resolve_concept(&request2, &pack, &options_error);
    assert_eq!(result_error.diagnostic_severity, DiagnosticSeverity::Error);
}

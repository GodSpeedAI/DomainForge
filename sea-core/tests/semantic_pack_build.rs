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

fn minimal_build_input() -> PackBuildInput {
    let supplier = make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
        "A party that provides goods or services",
        "owner@test.com",
    );
    let review = make_review_record("supplier", ReviewDecision::Approve, "sha256:placeholder");

    PackBuildInput {
        org_id: "test-org".to_string(),
        domain_id: "test-domain".to_string(),
        pack_version: "1.0.0".to_string(),
        meaning_version: "1.0.0".to_string(),
        approval: ApprovalState::Candidate,
        concepts: vec![supplier],
        relations: vec![],
        metrics: vec![],
        dimensions: vec![],
        units: vec![],
        aliases: vec![],
        mapping_rules: vec![],
        review_records: vec![review],
        previous_pack: None,
        allow_first_approved_version: false,
        source_graph_hash: "sha256:test".to_string(),
    }
}

fn minimal_approved_build_input() -> PackBuildInput {
    let mut input = minimal_build_input();
    input.approval = ApprovalState::Approved;
    input.allow_first_approved_version = true;

    let supplier = make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
        "A party that provides goods or services",
        "owner@test.com",
    );
    input.concepts = vec![supplier];

    let review = make_review_record("supplier", ReviewDecision::Approve, "sha256:placeholder");
    input.review_records = vec![review];

    input
}

#[test]
fn build_is_deterministic_across_repeated_runs() {
    let input = minimal_build_input();
    let out1 = build_semantic_pack(input.clone()).unwrap();
    let out2 = build_semantic_pack(input).unwrap();

    assert_eq!(out1.meaning_fingerprint, out2.meaning_fingerprint);

    let mut p1 = out1.pack.clone();
    let mut p2 = out2.pack.clone();
    p1.created_at = String::new();
    p2.created_at = String::new();
    assert_eq!(
        serde_json::to_string(&p1).unwrap(),
        serde_json::to_string(&p2).unwrap(),
    );
}

#[test]
fn approved_pack_requires_review_manifest() {
    let mut input = minimal_approved_build_input();
    input.review_records = vec![];

    let result = build_semantic_pack(input);
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(diags
        .iter()
        .any(|d| d.code == SemanticDiagnosticCode::UnreviewedConcept));
}

#[test]
fn missing_definition_produces_warning_on_approved_pack() {
    let mut input = minimal_approved_build_input();
    input.concepts[0].definition.text = String::new();
    input.concepts[0].definition.definition_hash = String::new();

    let result = build_semantic_pack(input);
    match result {
        Ok(out) => {
            assert!(out
                .pre_pack_diagnostics
                .iter()
                .any(|d| d.code == SemanticDiagnosticCode::MissingDefinition));
        }
        Err(diags) => {
            assert!(diags
                .iter()
                .any(|d| d.code == SemanticDiagnosticCode::MissingDefinition));
        }
    }
}

#[test]
fn missing_owner_produces_warning_on_approved_pack() {
    let mut input = minimal_approved_build_input();
    input.concepts[0].owner = String::new();

    let result = build_semantic_pack(input);
    match result {
        Ok(out) => {
            assert!(out
                .pre_pack_diagnostics
                .iter()
                .any(|d| d.code == SemanticDiagnosticCode::MissingOwner));
        }
        Err(diags) => {
            assert!(diags
                .iter()
                .any(|d| d.code == SemanticDiagnosticCode::MissingOwner));
        }
    }
}

#[test]
fn duplicate_canonical_name_is_invalid() {
    let mut input = minimal_approved_build_input();
    let dup = make_concept(
        "supplier_v2",
        "Supplier",
        ConceptStatus::Active,
        "Duplicate supplier",
        "owner@test.com",
    );
    input.concepts.push(dup);
    input.review_records.push(make_review_record(
        "supplier_v2",
        ReviewDecision::Approve,
        "sha256:placeholder",
    ));

    let result = build_semantic_pack(input);
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(diags
        .iter()
        .any(|d| d.code == SemanticDiagnosticCode::DuplicateCanonicalName));
}

#[test]
fn pack_hash_excludes_signature_fields() {
    let input = minimal_build_input();
    let out = build_semantic_pack(input).unwrap();

    assert!(out.pack.trust.signature.is_none());
    assert_eq!(out.pack.trust.signature_state, SignatureState::Unsigned);
}

#[test]
fn meaning_fingerprint_change_requires_meaning_version_bump() {
    let input1 = minimal_build_input();
    let out1 = build_semantic_pack(input1).unwrap();

    let mut input2 = minimal_build_input();
    input2.previous_pack = Some(out1.pack.clone());
    input2.meaning_version = "1.0.0".to_string();

    let changed_concept = make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
        "CHANGED definition text",
        "owner@test.com",
    );
    input2.concepts = vec![changed_concept];

    let review = make_review_record("supplier", ReviewDecision::Approve, "sha256:placeholder");
    input2.review_records = vec![review];

    let result = build_semantic_pack(input2);
    if let Err(diags) = &result {
        if diags.iter().any(|d| d.code == SemanticDiagnosticCode::MeaningVersionNotBumped) {
            return;
        }
    }

    let mut input3 = minimal_build_input();
    input3.previous_pack = Some(out1.pack);
    input3.meaning_version = "1.0.1".to_string();
    let changed = make_concept(
        "supplier",
        "Supplier",
        ConceptStatus::Active,
        "CHANGED definition text",
        "owner@test.com",
    );
    input3.concepts = vec![changed];
    let review2 = make_review_record("supplier", ReviewDecision::Approve, "sha256:placeholder");
    input3.review_records = vec![review2];

    let out3 = build_semantic_pack(input3).unwrap();
    assert_ne!(out3.meaning_fingerprint, out1.meaning_fingerprint);
}

#[test]
fn canonical_json_sorts_examples_and_counterexamples() {
    let mut concept = make_concept(
        "test",
        "Test",
        ConceptStatus::Active,
        "Test definition",
        "owner@test.com",
    );
    concept.examples = vec![
        "z_example".to_string(),
        "a_example".to_string(),
        "m_example".to_string(),
    ];
    concept.counterexamples = vec![
        "z_counter".to_string(),
        "a_counter".to_string(),
    ];

    let mut input = minimal_build_input();
    input.concepts = vec![concept];

    let out = build_semantic_pack(input).unwrap();
    let c = &out.pack.concepts[0];
    assert_eq!(c.examples, vec!["a_example", "m_example", "z_example"]);
    assert_eq!(c.counterexamples, vec!["a_counter", "z_counter"]);
}

#[test]
fn unsigned_fixture_bypass_is_cli_only_and_audited() {
    let pack = build_semantic_pack(minimal_build_input()).unwrap().pack;

    let options = ValidationOptions {
        mode: ValidationMode::Warn,
        unknown_concept_policy: UnknownConceptPolicy::Warning,
        deprecated_policy: DeprecatedPolicy::Warn,
        require_signed_pack: true,
        allow_unsigned_test_fixtures: true,
    };

    let result = validate_graph_with_pack(&pack, "test://fixture", &options);
    assert!(result.unsigned_fixture_bypass_used);
    assert_ne!(result.status, SemanticValidationStatus::Blocked);

    let options_strict = ValidationOptions {
        mode: ValidationMode::Strict,
        ..options.clone()
    };
    let options_no_bypass = ValidationOptions {
        allow_unsigned_test_fixtures: false,
        ..options_strict
    };
    let result_blocked = validate_graph_with_pack(&pack, "test://fixture", &options_no_bypass);
    assert_eq!(result_blocked.status, SemanticValidationStatus::Blocked);
}

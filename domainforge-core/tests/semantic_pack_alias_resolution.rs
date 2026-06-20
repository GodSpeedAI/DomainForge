use domainforge_core::semantic_pack::*;

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

fn make_alias(alias: &str, target_id: &str, status: AliasStatus) -> AliasDef {
    AliasDef {
        alias: alias.to_string(),
        normalized_alias: normalize_lookup_key(alias),
        target_concept_id: target_id.to_string(),
        status,
        confidence: None,
        decision_ref: format!("dec_alias_{}", alias),
        source_ref: SourceRef::synthetic("test://alias"),
    }
}

fn make_pack_with_concepts_and_aliases(
    concepts: Vec<ConceptDef>,
    aliases: Vec<AliasDef>,
) -> SemanticPack {
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
            name: "domainforge-core".to_string(),
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
        aliases,
        mapping_rules: vec![],
        compatibility: schema::CompatibilityInfo::default(),
    }
}

#[test]
fn alias_resolution_uses_full_candidate_set() {
    let concepts = vec![
        make_concept("supplier", "Supplier", ConceptStatus::Active),
        make_concept("vendor", "VendorConcept", ConceptStatus::Active),
    ];

    let aliases = vec![
        make_alias("Vendor", "supplier", AliasStatus::Approved),
        make_alias("vendor", "vendor", AliasStatus::Approved),
    ];

    let pack = make_pack_with_concepts_and_aliases(concepts, aliases);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "Vendor",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    // "Vendor" normalizes to "vendor", matching two approved aliases targeting
    // different concepts → expect an ambiguity diagnostic.
    assert!(
        result.diagnostic_code.is_some(),
        "expected ambiguity diagnostic for conflicting 'vendor' aliases, got resolved_concept_id={:?}",
        result.resolved_concept_id
    );
}

#[test]
fn approved_and_deprecated_alias_same_key_blocks_approved_build() {
    let concepts = vec![
        make_concept("supplier_a", "SupplierA", ConceptStatus::Active),
        make_concept("supplier_b", "SupplierB", ConceptStatus::Active),
    ];

    let aliases = vec![
        make_alias("Vendor", "supplier_a", AliasStatus::Approved),
        make_alias("Vendor", "supplier_b", AliasStatus::Deprecated),
    ];

    let input = PackBuildInput {
        org_id: "test-org".to_string(),
        domain_id: "test-domain".to_string(),
        pack_version: "1.0.0".to_string(),
        meaning_version: "1.0.0".to_string(),
        approval: ApprovalState::Approved,
        concepts,
        relations: vec![],
        metrics: vec![],
        dimensions: vec![],
        units: vec![],
        aliases,
        mapping_rules: vec![],
        review_records: vec![
            ReviewRecord {
                decision_id: "dec_1".to_string(),
                subject_type: "concept".to_string(),
                subject_id: "supplier_a".to_string(),
                decision: ReviewDecision::Approve,
                rationale: "ok".to_string(),
                reviewer: "r@test.com".to_string(),
                reviewed_at: "2026-06-07T00:00:00Z".to_string(),
                definition_hash: "sha256:x".to_string(),
                previous_definition_hash: None,
                new_definition_hash: None,
            },
            ReviewRecord {
                decision_id: "dec_2".to_string(),
                subject_type: "concept".to_string(),
                subject_id: "supplier_b".to_string(),
                decision: ReviewDecision::Approve,
                rationale: "ok".to_string(),
                reviewer: "r@test.com".to_string(),
                reviewed_at: "2026-06-07T00:00:00Z".to_string(),
                definition_hash: "sha256:x".to_string(),
                previous_definition_hash: None,
                new_definition_hash: None,
            },
        ],
        previous_pack: None,
        allow_first_approved_version: true,
        source_graph_hash: "sha256:test".to_string(),
    };

    let result = build_semantic_pack(input);
    assert!(result.is_err());
    let diags = result.unwrap_err();
    assert!(diags
        .iter()
        .any(|d| d.code == SemanticDiagnosticCode::AliasConflict));
}

#[test]
fn approved_and_deprecated_alias_same_key_is_ambiguous() {
    let concepts = vec![
        make_concept("supplier_a", "SupplierA", ConceptStatus::Active),
        make_concept("supplier_b", "SupplierB", ConceptStatus::Active),
    ];

    let aliases = vec![
        make_alias("Provider", "supplier_a", AliasStatus::Ambiguous),
        make_alias("Provider", "supplier_b", AliasStatus::Ambiguous),
    ];

    let pack = make_pack_with_concepts_and_aliases(concepts, aliases);
    let options = ValidationOptions::default();
    let source_ref = SourceRef::synthetic("test://term");

    let request = ResolveRequest {
        raw_text: "Provider",
        expected_kind: None,
        source_ref,
    };
    let result = resolve_concept(&request, &pack, &options);

    assert_eq!(result.semantic_truth, SemanticTruth::Unknown);
    assert_eq!(
        result.diagnostic_code,
        Some(SemanticDiagnosticCode::AmbiguousAlias)
    );
}

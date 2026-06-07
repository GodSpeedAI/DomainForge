use sea_core::semantic_pack::*;

fn make_concept(
    id: &str,
    canonical_name: &str,
    def_hash: &str,
) -> ConceptDef {
    ConceptDef {
        id: id.to_string(),
        canonical_name: canonical_name.to_string(),
        kind: ConceptKind::Entity,
        status: ConceptStatus::Active,
        definition: ConceptDefinition {
            text: format!("Definition of {}", canonical_name),
            definition_hash: def_hash.to_string(),
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

fn make_pack(
    pack_id: &str,
    org: &str,
    domain: &str,
    concepts: Vec<ConceptDef>,
    units: Vec<schema::UnitDef>,
) -> SemanticPack {
    SemanticPack {
        schema_version: "0.3".to_string(),
        pack_id: pack_id.to_string(),
        org_id: org.to_string(),
        domain_id: domain.to_string(),
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
        units,
        aliases: vec![],
        mapping_rules: vec![],
        compatibility: schema::CompatibilityInfo::default(),
    }
}

#[test]
fn pack_set_hash_is_independent_of_config_array_order() {
    let pack_a = make_pack(
        "org/dom/a-1.0.0",
        "org",
        "dom",
        vec![make_concept("concept_1", "Concept1", "sha256:hash1")],
        vec![],
    );
    let pack_b = make_pack(
        "org/dom/b-1.0.0",
        "org",
        "dom",
        vec![make_concept("concept_2", "Concept2", "sha256:hash2")],
        vec![],
    );

    let result1 = merge_packs(&[pack_a.clone(), pack_b.clone()], &[0, 1]);
    let result2 = merge_packs(&[pack_b, pack_a], &[1, 0]);

    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert_eq!(
        result1.unwrap().merged_pack_hash,
        result2.unwrap().merged_pack_hash
    );
}

#[test]
fn pack_set_conflict_blocks_strict_mode() {
    let pack_a = make_pack(
        "org/dom/a-1.0.0",
        "org",
        "dom",
        vec![make_concept("shared_concept", "Shared", "sha256:hash_a")],
        vec![],
    );
    let pack_b = make_pack(
        "org/dom/b-1.0.0",
        "org",
        "dom",
        vec![make_concept("shared_concept", "Shared", "sha256:hash_b")],
        vec![],
    );

    let result = merge_packs(&[pack_a, pack_b], &[0, 1]);
    assert!(result.is_err());
    let conflicts = result.unwrap_err();
    assert!(conflicts
        .iter()
        .any(|c| c.conflict_type == pack_set::ConflictType::SameConceptIdDifferentHash));
}

#[test]
fn empty_pack_set_succeeds() {
    let result = merge_packs(&[], &[]);
    assert!(result.is_ok());
    let set = result.unwrap();
    assert!(set.packs.is_empty());
}

#[test]
fn single_pack_set_succeeds() {
    let pack = make_pack(
        "org/dom/p-1.0.0",
        "org",
        "dom",
        vec![make_concept("concept_1", "Concept1", "sha256:hash1")],
        vec![],
    );

    let result = merge_packs(&[pack], &[0]);
    assert!(result.is_ok());
    let set = result.unwrap();
    assert_eq!(set.packs.len(), 1);
}

#[test]
fn identical_concepts_no_conflict() {
    let pack_a = make_pack(
        "org/dom/a-1.0.0",
        "org",
        "dom",
        vec![make_concept("shared", "Shared", "sha256:same_hash")],
        vec![],
    );
    let pack_b = make_pack(
        "org/dom/b-1.0.0",
        "org",
        "dom",
        vec![make_concept("shared", "Shared", "sha256:same_hash")],
        vec![],
    );

    let result = merge_packs(&[pack_a, pack_b], &[0, 1]);
    assert!(result.is_ok());
}

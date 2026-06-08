#![cfg(feature = "signing")]

use sea_core::semantic_pack::*;

fn test_keypair_pem() -> (Vec<u8>, Vec<u8>) {
    use ed25519_dalek::pkcs8::{EncodePrivateKey, EncodePublicKey};

    let private_key = ed25519_dalek::SigningKey::from_bytes(&[7u8; 32]);
    let public_key = private_key.verifying_key();

    (
        private_key
            .to_pkcs8_pem(Default::default())
            .expect("failed to encode test private key")
            .to_string()
            .into_bytes(),
        public_key
            .to_public_key_pem(Default::default())
            .expect("failed to encode test public key")
            .into_bytes(),
    )
}

fn make_concept(id: &str, canonical_name: &str) -> ConceptDef {
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

fn make_pack() -> SemanticPack {
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

#[test]
fn signature_verifies_content_hash() {
    let pack = make_pack();
    let (private_key, public_key) = test_keypair_pem();

    let sign_output = sign_pack(&pack, &private_key).expect("signing failed");

    let mut signed_pack = pack;
    signed_pack.trust.signature_state = SignatureState::Signed;
    signed_pack.trust.signature = Some(sign_output.signature.clone());

    let verify_result = verify_pack_signature(&signed_pack, &public_key);
    assert!(verify_result.is_ok());
}

#[test]
fn signature_payload_pack_id_must_match_pack_field() {
    let mut pack = make_pack();
    let (private_key, public_key) = test_keypair_pem();

    let sign_output = sign_pack(&pack, &private_key).expect("signing failed");

    pack.pack_id = "tampered/different/pack-id".to_string();
    pack.trust.signature_state = SignatureState::Signed;
    pack.trust.signature_alg = Some(sign_output.signature_alg);
    pack.trust.signature = Some(sign_output.signature);

    let verify_result = verify_pack_signature(&pack, &public_key);
    assert!(verify_result.is_err());
}

#[test]
fn signature_payload_schema_version_must_match_pack_field() {
    let mut pack = make_pack();
    let (private_key, public_key) = test_keypair_pem();

    let sign_output = sign_pack(&pack, &private_key).expect("signing failed");

    pack.schema_version = "9.9.9".to_string();
    pack.trust.signature_state = SignatureState::Signed;
    pack.trust.signature_alg = Some(sign_output.signature_alg);
    pack.trust.signature = Some(sign_output.signature);

    let verify_result = verify_pack_signature(&pack, &public_key);
    assert!(verify_result.is_err());
}

#[test]
fn tampered_pack_fails_signature_verification() {
    let pack = make_pack();
    let (private_key, public_key) = test_keypair_pem();

    let sign_output = sign_pack(&pack, &private_key).expect("signing failed");

    let mut tampered_pack = pack;
    tampered_pack.concepts.push(ConceptDef {
        id: "injected_concept".to_string(),
        canonical_name: "InjectedConcept".to_string(),
        kind: ConceptKind::Entity,
        status: ConceptStatus::Active,
        definition: ConceptDefinition {
            text: "Malicious definition".to_string(),
            definition_hash: "sha256:fake".to_string(),
            decision_ref: "dec_fake".to_string(),
        },
        owner: "attacker@evil.com".to_string(),
        source_refs: vec![],
        examples: vec![],
        counterexamples: vec![],
        allowed_predicates: vec![],
        valid_contexts: vec![],
    });
    tampered_pack.trust.signature_state = SignatureState::Signed;
    tampered_pack.trust.signature_alg = Some(sign_output.signature_alg);
    tampered_pack.trust.signature = Some(sign_output.signature);

    let verify_result = verify_pack_signature(&tampered_pack, &public_key);
    assert!(verify_result.is_err());
}

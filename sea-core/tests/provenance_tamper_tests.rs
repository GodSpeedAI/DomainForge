#![cfg(feature = "cli")]

use sea_core::authority::pack::{compute_pack_hash, AuthorityPack};
use sea_core::authority::*;

fn minimal_policy(id: &str) -> AuthorityPolicy {
    AuthorityPolicy {
        policy_id: id.to_string(),
        modality: PolicyModality::Prohibition,
        priority: 100,
        applies_to: StructuralPredicates {
            predicates: {
                let mut m = std::collections::HashMap::new();
                m.insert("action".to_string(), serde_json::json!("ShipOrder"));
                m
            },
        },
        when: Some(ConditionPredicates {
            conditions: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "customer.credit_status".to_string(),
                    serde_json::json!("Hold"),
                );
                m
            },
        }),
        requires_fact: vec![FactRequirement {
            fact_path: "customer.credit_status".to_string(),
            allowed_source_classes: vec![SourceClass::SystemOfRecord],
            allowed_source_ids: vec![],
            max_age: None,
            evidence_ref_required: true,
            signature_required: false,
            minimum_confidence: None,
            required_transform: None,
            derived_from_source: None,
        }],
        semantics_version: "0.4".to_string(),
        override_spec: None,
        obligation_spec: None,
        description: None,
        evidence_ref: None,
    }
}

fn build_valid_pack() -> AuthorityPack {
    let policies = vec![minimal_policy("block_hold")];
    let hash = compute_pack_hash("test-pack", "1.0.0", "0.4", "standard", &policies)
        .expect("compute_pack_hash");
    AuthorityPack {
        id: "test-pack".to_string(),
        version: "1.0.0".to_string(),
        semantics_version: "0.4".to_string(),
        required_specificity_profile: "standard".to_string(),
        policies,
        hash,
        signature: None,
        owner: None,
        created_at: None,
        approved_by: None,
        evidence_ref: None,
    }
}

#[test]
fn pack_hash_validates_when_untampered() {
    let pack = build_valid_pack();
    assert!(
        pack.validate_hash().is_ok(),
        "untampered pack must pass hash validation"
    );
}

#[test]
fn signed_pack_tamper_detected() {
    let mut pack = build_valid_pack();
    assert!(pack.validate_hash().is_ok());

    pack.policies[0].priority = 1;

    let result = pack.validate_hash();
    assert!(result.is_err(), "tampered pack must fail hash validation");
    let err = result.unwrap_err();
    assert!(
        format!("{err:?}").contains("PackHashMismatch"),
        "error must be PackHashMismatch; got: {err:?}"
    );
}

#[test]
fn pack_hash_tamper_via_id_detected() {
    let mut pack = build_valid_pack();
    pack.id = "tampered-pack-id".to_string();
    assert!(
        pack.validate_hash().is_err(),
        "changing pack id without rehashing must be detected"
    );
}

#![cfg(feature = "cli")]

use sea_core::authority::pack::compute_pack_hash;
use sea_core::authority::*;

fn make_request(operation: &str, resource_type: &str) -> AuthorityRequest {
    use chrono::Utc;
    AuthorityRequest {
        request_id: "req-null-test-001".to_string(),
        actor: ActorContext {
            id: "user-1".to_string(),
            role: Some("Operator".to_string()),
            groups: vec![],
            service_account: None,
            agent_identity: None,
        },
        operation: operation.to_string(),
        resource: ResourceRef {
            id: None,
            type_: Some(resource_type.to_string()),
            extra: Default::default(),
        },
        context: serde_json::json!({}),
        requested_at: Utc::now(),
        correlation_id: None,
        risk_class: None,
        metadata: Default::default(),
    }
}

fn make_permission_pack() -> AuthorityPack {
    let policies = vec![AuthorityPolicy {
        policy_id: "allow_clear".to_string(),
        modality: PolicyModality::Permission,
        priority: 50,
        applies_to: StructuralPredicates {
            predicates: {
                let mut m = std::collections::HashMap::new();
                m.insert("action".to_string(), serde_json::json!("ShipOrder"));
                m.insert("resource.type".to_string(), serde_json::json!("Order"));
                m
            },
        },
        when: Some(ConditionPredicates {
            conditions: {
                let mut m = std::collections::HashMap::new();
                m.insert(
                    "customer.credit_status".to_string(),
                    serde_json::json!("Clear"),
                );
                m
            },
        }),
        requires_fact: vec![FactRequirement {
            fact_path: "customer.credit_status".to_string(),
            allowed_source_classes: vec![SourceClass::SystemOfRecord, SourceClass::Derived],
            allowed_source_ids: vec![],
            max_age: None,
            evidence_ref_required: false,
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
    }];
    let hash = compute_pack_hash("pp", "1.0.0", "0.4", "default", &policies).expect("hash");
    AuthorityPack {
        id: "pp".to_string(),
        version: "1.0.0".to_string(),
        semantics_version: "0.4".to_string(),
        required_specificity_profile: "default".to_string(),
        policies,
        hash,
        signature: None,
        owner: None,
        created_at: None,
        approved_by: None,
        evidence_ref: None,
    }
}

fn make_env_config(packs: Vec<serde_json::Value>) -> AuthorityEnvironmentConfig {
    AuthorityEnvironmentConfig {
        resolver_semantics_version: "0.4".to_string(),
        specificity_profile: SpecificityProfile::default_profile(),
        unknown_handling: UnknownHandlingConfig::defaults(),
        fact_sources: vec![FactSource {
            id: "credit-service".to_string(),
            source_class: SourceClass::SystemOfRecord,
            allowed_paths: vec!["customer.".to_string()],
            evidence_required: false,
            signature_required: false,
            max_response_latency_ms: 5000,
            health_endpoint: None,
            credential_ref: None,
            schema_ref: None,
            owner: None,
            recovery_hint: None,
        }],
        fact_transforms: vec![],
        authority_packs: packs,
        strict_mode: true,
        compatibility_lowering_version: "bounded_compatibility_v1".to_string(),
        resolver_version: "0.1.0".to_string(),
    }
}

#[test]
fn permission_with_missing_required_fact_is_not_allowed() {
    let pack = make_permission_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).expect("env created");
    env.validate().expect("env valid");

    let request = make_request("ShipOrder", "Order");
    let facts: Vec<FactEnvelope> = vec![];

    let result = env.evaluate(&request, &facts);
    assert!(
        result.is_ok(),
        "evaluation must complete even with missing facts"
    );
    let (trace, decision) = result.unwrap();

    assert_ne!(
        decision.final_decision,
        FinalDecision::Allow,
        "permission with a missing (unknown) required fact must NOT silently allow; \
         unknown evidence must not collapse to a positive authorization."
    );
    assert!(
        !trace.unknown_decisions.is_empty() || !trace.fact_envelopes.is_empty(),
        "trace must record the unknown/missing fact context; \
         unknown_decisions or fact_envelopes must be non-empty."
    );
}

// ---------------------------------------------------------------------------
// Derivation-level Null-poisoning (audit §4 #8 / G7 / Phase 6): the key invariant
// is that a derived fact built FROM a Null premise must itself be Null
// (poisoning propagates through the derivation), and a derived fact whose premise
// is absent must NOT be fabricated. These drive the `DerivedFactEngine` directly.
// ---------------------------------------------------------------------------

fn passthrough_transform(input_path: &str, output_path: &str) -> FactTransform {
    FactTransform {
        id: "credit_status_passthrough".to_string(),
        version: "1.0.0".to_string(),
        hash: "deadbeef".to_string(),
        inputs: vec![TransformInput {
            fact_path: input_path.to_string(),
            source_classes: vec![SourceClass::SystemOfRecord],
        }],
        output: TransformOutput {
            fact_path: output_path.to_string(),
        },
        purity: PurityFlags::default(),
        determinism_tests: vec![],
    }
}

fn fact_with_value(path: &str, value: serde_json::Value) -> FactEnvelope {
    use chrono::Utc;
    FactEnvelope {
        path: path.to_string(),
        value,
        source_class: SourceClass::SystemOfRecord,
        source_id: "credit-service".to_string(),
        observed_at: Utc::now(),
        expires_at: None,
        evidence_ref: None,
        signature: None,
        confidence: None,
        lineage: None,
    }
}

#[test]
fn derived_fact_from_null_value_premise_is_null() {
    let mut registry = FactTransformRegistry::new();
    registry
        .register(passthrough_transform(
            "customer.raw_status",
            "customer.credit_status",
        ))
        .expect("pure transform registers");
    let engine = DerivedFactEngine::new(registry);

    // Premise is PRESENT but its value is Null.
    let premise = fact_with_value("customer.raw_status", serde_json::Value::Null);

    let (derived, lineages) = engine
        .compute_derived_facts(&[premise], &["credit_status_passthrough@1.0.0".to_string()])
        .expect("derivation runs");

    assert_eq!(
        derived.len(),
        1,
        "derived fact must be produced from a present (null-valued) premise"
    );
    assert!(
        derived[0].value.is_null(),
        "Null-poisoning must propagate: a derived fact computed from a Null premise \
         must itself be Null, never a fabricated non-null value. Got: {:?}",
        derived[0].value
    );
    assert_eq!(
        lineages.len(),
        1,
        "lineage must record the derivation even when the derived value is Null"
    );
}

#[test]
fn derived_fact_from_missing_premise_is_not_fabricated() {
    let mut registry = FactTransformRegistry::new();
    registry
        .register(passthrough_transform(
            "customer.raw_status",
            "customer.credit_status",
        ))
        .expect("pure transform registers");
    let engine = DerivedFactEngine::new(registry);

    // Premise is ABSENT entirely.
    let facts: Vec<FactEnvelope> = vec![];

    let (derived, lineages) = engine
        .compute_derived_facts(&facts, &["credit_status_passthrough@1.0.0".to_string()])
        .expect("derivation runs");

    assert!(
        derived.is_empty(),
        "a derived fact whose premise is missing must NOT be fabricated; \
         the absent input must surface as unknown downstream, not a default value."
    );
    assert!(
        lineages.is_empty(),
        "no lineage may be emitted for a derived fact that was not produced"
    );
}

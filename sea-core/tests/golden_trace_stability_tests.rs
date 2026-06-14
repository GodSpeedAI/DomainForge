#![cfg(feature = "cli")]

use sea_core::authority::pack::compute_pack_hash;
use sea_core::authority::*;

fn make_request(operation: &str, actor_role: Option<&str>) -> AuthorityRequest {
    use chrono::Utc;
    AuthorityRequest {
        request_id: "req-fixed-001".to_string(),
        actor: ActorContext {
            id: "user-1".to_string(),
            role: actor_role.map(|s| s.to_string()),
            groups: vec![],
            service_account: None,
            agent_identity: None,
        },
        operation: operation.to_string(),
        resource: ResourceRef {
            id: None,
            type_: Some("Order".to_string()),
            extra: Default::default(),
        },
        context: serde_json::json!({}),
        requested_at: Utc::now(),
        correlation_id: None,
        risk_class: None,
        metadata: Default::default(),
    }
}

fn make_trusted_fact(path: &str, value: serde_json::Value) -> FactEnvelope {
    use chrono::Utc;
    FactEnvelope {
        path: path.to_string(),
        value,
        source_class: SourceClass::SystemOfRecord,
        source_id: "credit-service".to_string(),
        observed_at: Utc::now(),
        expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
        evidence_ref: Some("ref-1".to_string()),
        signature: None,
        confidence: None,
        lineage: None,
    }
}

fn make_prohibition_pack() -> AuthorityPack {
    let policies = vec![AuthorityPolicy {
        policy_id: "block_credit_hold".to_string(),
        modality: PolicyModality::Prohibition,
        priority: 100,
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
    }];
    let hash = compute_pack_hash("sp", "1.0.0", "0.4", "default", &policies).expect("hash");
    AuthorityPack {
        id: "sp".to_string(),
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
            evidence_required: true,
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
fn authority_trace_is_byte_stable_across_runs() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).expect("env created");
    env.validate().expect("env valid");

    let request = make_request("ShipOrder", Some("WarehouseOperator"));
    let facts = vec![make_trusted_fact(
        "customer.credit_status",
        serde_json::json!("Hold"),
    )];

    let (trace1, _) = env.evaluate(&request, &facts).expect("eval 1");

    let request2 = make_request("ShipOrder", Some("WarehouseOperator"));
    let facts2 = vec![make_trusted_fact(
        "customer.credit_status",
        serde_json::json!("Hold"),
    )];
    let (trace2, _) = env.evaluate(&request2, &facts2).expect("eval 2");

    let mut json1 = serde_json::to_value(&trace1).expect("serialize trace1");
    let mut json2 = serde_json::to_value(&trace2).expect("serialize trace2");

    if let Some(obj) = json1.as_object_mut() {
        obj.remove("created_at");
        obj.remove("decision_id");
    }
    if let Some(obj) = json2.as_object_mut() {
        obj.remove("created_at");
        obj.remove("decision_id");
    }

    assert_eq!(
        json1, json2,
        "AuthorityTrace must be byte-stable across identical evaluations \
         (excluding created_at and decision_id which are per-call unique). \
         Every hash, fact envelope, and decision field must be identical."
    );
}

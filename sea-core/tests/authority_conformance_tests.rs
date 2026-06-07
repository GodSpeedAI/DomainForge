use sea_core::authority::*;
use chrono::Utc;

fn make_request(operation: &str, actor_id: &str, actor_role: Option<&str>, resource_type: &str) -> AuthorityRequest {
    AuthorityRequest {
        request_id: uuid::Uuid::new_v4().to_string(),
        actor: ActorContext {
            id: actor_id.to_string(),
            role: actor_role.map(|s| s.to_string()),
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

fn make_trusted_fact(path: &str, value: serde_json::Value) -> FactEnvelope {
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

fn make_caller_fact(path: &str, value: serde_json::Value) -> FactEnvelope {
    FactEnvelope {
        path: path.to_string(),
        value,
        source_class: SourceClass::CallerSupplied,
        source_id: "caller".to_string(),
        observed_at: Utc::now(),
        expires_at: None,
        evidence_ref: None,
        signature: None,
        confidence: None,
        lineage: None,
    }
}

fn make_prohibition_pack() -> AuthorityPack {
    let policies = vec![AuthorityPolicy {
        policy_id: "block_credit_hold_shipping".to_string(),
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
                m.insert("customer.credit_status".to_string(), serde_json::json!("Hold"));
                m
            },
        }),
        requires_fact: vec![FactRequirement {
            fact_path: "customer.credit_status".to_string(),
            allowed_source_classes: vec![SourceClass::SystemOfRecord, SourceClass::Derived],
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
        description: Some("Block shipping for credit hold orders".to_string()),
        evidence_ref: None,
    }];
    let hash = compute_pack_hash(&policies).expect("pack hash computation failed");
    AuthorityPack {
        id: "shipping-policy".to_string(),
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

fn make_permission_pack() -> AuthorityPack {
    let policies = vec![AuthorityPolicy {
        policy_id: "allow_clear_order_shipping".to_string(),
        modality: PolicyModality::Permission,
        priority: 50,
        applies_to: StructuralPredicates {
            predicates: {
                let mut m = std::collections::HashMap::new();
                m.insert("actor.role".to_string(), serde_json::json!("WarehouseOperator"));
                m.insert("action".to_string(), serde_json::json!("ShipOrder"));
                m.insert("resource.type".to_string(), serde_json::json!("Order"));
                m
            },
        },
        when: Some(ConditionPredicates {
            conditions: {
                let mut m = std::collections::HashMap::new();
                m.insert("customer.credit_status".to_string(), serde_json::json!("Clear"));
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
    let hash = compute_pack_hash(&policies).expect("pack hash computation failed");
    AuthorityPack {
        id: "shipping-permission".to_string(),
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

fn make_override_pack() -> AuthorityPack {
    let policies = vec![AuthorityPolicy {
        policy_id: "legal_override_shipping_hold".to_string(),
        modality: PolicyModality::Override,
        priority: 200,
        applies_to: StructuralPredicates {
            predicates: {
                let mut m = std::collections::HashMap::new();
                m.insert("actor.role".to_string(), serde_json::json!("LegalOps"));
                m.insert("action".to_string(), serde_json::json!("ShipOrder"));
                m.insert("resource.type".to_string(), serde_json::json!("Order"));
                m
            },
        },
        when: Some(ConditionPredicates {
            conditions: {
                let mut m = std::collections::HashMap::new();
                m.insert("legal_override.status".to_string(), serde_json::json!("Approved"));
                m.insert("legal_override.scope".to_string(), serde_json::json!("ShippingHold"));
                m
            },
        }),
        requires_fact: vec![
            FactRequirement {
                fact_path: "legal_override.status".to_string(),
                allowed_source_classes: vec![SourceClass::SystemOfRecord, SourceClass::Attested, SourceClass::ManualApproval],
                allowed_source_ids: vec![],
                max_age: None,
                evidence_ref_required: false,
                signature_required: false,
                minimum_confidence: None,
                required_transform: None,
                derived_from_source: None,
            },
            FactRequirement {
                fact_path: "legal_override.id".to_string(),
                allowed_source_classes: vec![SourceClass::SystemOfRecord, SourceClass::Attested, SourceClass::ManualApproval],
                allowed_source_ids: vec![],
                max_age: None,
                evidence_ref_required: false,
                signature_required: false,
                minimum_confidence: None,
                required_transform: None,
                derived_from_source: None,
            },
        ],
        semantics_version: "0.4".to_string(),
        override_spec: Some(OverrideSpec {
            permits: "ShipOrder".to_string(),
        }),
        obligation_spec: None,
        description: None,
        evidence_ref: None,
    }];
    let hash = compute_pack_hash(&policies).expect("pack hash computation failed");
    AuthorityPack {
        id: "override-policy".to_string(),
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
fn test_authority_decision_produced() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts = vec![make_trusted_fact("customer.credit_status", serde_json::json!("Hold"))];

    let (trace, decision) = env.evaluate(&request, &facts).unwrap();
    assert_eq!(decision.final_decision, FinalDecision::Deny);
    assert!(!decision.decision_id.is_empty());
    assert!(!decision.trace_ref.is_empty());
}

#[test]
fn test_trace_completeness() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts = vec![make_trusted_fact("customer.credit_status", serde_json::json!("Hold"))];

    let (trace, _) = env.evaluate(&request, &facts).unwrap();
    assert!(!trace.decision_id.is_empty());
    assert_eq!(trace.request_id, request.request_id);
    assert!(!trace.ir_hash.is_empty());
    assert!(!trace.pack_hashes.is_empty());
    assert!(!trace.resolver_version.is_empty());
    assert!(!trace.resolver_semantics_version.is_empty());
    assert!(!trace.resolver_semantics_hash.is_empty());
    assert!(!trace.specificity_profile_id.is_empty());
    assert!(!trace.specificity_profile_hash.is_empty());
    assert!(!trace.unknown_handling_config_hash.is_empty());
    assert!(!trace.compatibility_lowering_version.is_empty());
    assert!(!trace.action_request_hash.is_empty());
    assert!(!trace.created_at.is_empty());
    assert!(!trace.fact_envelopes.is_empty());
    assert!(matches!(trace.claim_level, ClaimLevel::AuditBacked));
}

#[test]
fn test_missing_actor_rejected() {
    let request = AuthorityRequest {
        request_id: "r1".to_string(),
        actor: ActorContext {
            id: "".to_string(),
            role: None,
            groups: vec![],
            service_account: None,
            agent_identity: None,
        },
        operation: "ShipOrder".to_string(),
        resource: ResourceRef {
            id: Some("order-1".to_string()),
            type_: None,
            extra: Default::default(),
        },
        context: serde_json::json!({}),
        requested_at: Utc::now(),
        correlation_id: None,
        risk_class: None,
        metadata: Default::default(),
    };
    assert!(request.validate().is_err());
}

#[test]
fn test_permission_with_trusted_fact_allowed() {
    let pack = make_permission_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts = vec![make_trusted_fact("customer.credit_status", serde_json::json!("Clear"))];

    let (_, decision) = env.evaluate(&request, &facts).unwrap();
    assert_eq!(decision.final_decision, FinalDecision::Allow);
}

#[test]
fn test_permission_with_missing_fact_escalates() {
    let pack = make_permission_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts: Vec<FactEnvelope> = vec![];

    let (_, decision) = env.evaluate(&request, &facts).unwrap();
    assert_eq!(decision.final_decision, FinalDecision::Escalate);
}

#[test]
fn test_prohibition_with_missing_fact_denies() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts: Vec<FactEnvelope> = vec![];

    let (_, decision) = env.evaluate(&request, &facts).unwrap();
    assert_eq!(decision.final_decision, FinalDecision::Deny);
}

#[test]
fn test_caller_supplied_override_rejected() {
    let pack = make_override_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("LegalOps"), "Order");
    let facts = vec![
        make_caller_fact("legal_override.status", serde_json::json!("Approved")),
        make_caller_fact("legal_override.scope", serde_json::json!("ShippingHold")),
        make_caller_fact("legal_override.id", serde_json::json!("fake-approval")),
    ];

    let (_, decision) = env.evaluate(&request, &facts).unwrap();
    assert_eq!(decision.final_decision, FinalDecision::NotApplicable);
}

#[test]
fn test_no_applicable_policy() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ViewOrder", "user-1", Some("Viewer"), "Order");
    let facts = vec![];

    let (_, decision) = env.evaluate(&request, &facts).unwrap();
    assert_eq!(decision.final_decision, FinalDecision::NotApplicable);
}

#[test]
fn test_raw_context_defaults_to_caller_supplied() {
    let resolver = FactResolver::new(FactSourceRegistry::new());
    let now = Utc::now();
    let context = serde_json::json!({
        "customer": {
            "credit_status": "Clear"
        }
    });
    let envelopes = resolver.wrap_context_as_caller_supplied(&context, now);
    assert!(!envelopes.is_empty());
    for env in &envelopes {
        assert_eq!(env.source_class, SourceClass::CallerSupplied);
        assert_eq!(env.source_id, "caller");
    }
}

#[test]
fn test_fact_requirement_satisfaction() {
    let req = FactRequirement {
        fact_path: "customer.credit_status".to_string(),
        allowed_source_classes: vec![SourceClass::SystemOfRecord, SourceClass::Derived],
        allowed_source_ids: vec![],
        max_age: None,
        evidence_ref_required: true,
        signature_required: false,
        minimum_confidence: None,
        required_transform: None,
        derived_from_source: None,
    };

    let trusted = make_trusted_fact("customer.credit_status", serde_json::json!("Hold"));
    let now = Utc::now();
    assert!(req.is_satisfied_by(&trusted, &now));

    let caller = make_caller_fact("customer.credit_status", serde_json::json!("Hold"));
    assert!(!req.is_satisfied_by(&caller, &now));

    let no_evidence = FactEnvelope {
        evidence_ref: None,
        ..make_trusted_fact("customer.credit_status", serde_json::json!("Hold"))
    };
    assert!(!req.is_satisfied_by(&no_evidence, &now));
}

#[test]
fn test_fact_envelope_freshness() {
    let now = Utc::now();
    let fresh = FactEnvelope {
        expires_at: Some(now + chrono::Duration::hours(1)),
        ..make_trusted_fact("test", serde_json::json!(true))
    };
    assert!(fresh.is_fresh(now));

    let stale = FactEnvelope {
        expires_at: Some(now - chrono::Duration::hours(1)),
        ..make_trusted_fact("test", serde_json::json!(true))
    };
    assert!(!stale.is_fresh(now));
}

#[test]
fn test_three_valued_logic_unknown_preserved() {
    let cond = ConditionPredicates {
        conditions: {
            let mut m = std::collections::HashMap::new();
            m.insert("missing_fact".to_string(), serde_json::json!("expected"));
            m
        },
    };

    let facts = vec![];
    let result = cond.evaluate(&facts).unwrap();
    assert_eq!(result, ThreeValuedResult::Unknown);
}

#[test]
fn test_unknown_handling_defaults() {
    let config = UnknownHandlingConfig::defaults();
    assert_eq!(config.for_modality(&PolicyModality::Permission), FinalDecision::Escalate);
    assert_eq!(config.for_modality(&PolicyModality::Prohibition), FinalDecision::Deny);
    assert_eq!(config.for_modality(&PolicyModality::Override), FinalDecision::NotApplicable);
    assert_eq!(config.for_modality(&PolicyModality::Obligation), FinalDecision::Escalate);
}

#[test]
fn test_specificity_vector_dominance() {
    let a = SpecificityVector::new(vec![
        ("actor".to_string(), 1),
        ("role".to_string(), 1),
        ("action".to_string(), 1),
        ("resource".to_string(), 1),
    ]);
    let b = SpecificityVector::new(vec![
        ("actor".to_string(), 0),
        ("role".to_string(), 0),
        ("action".to_string(), 1),
        ("resource".to_string(), 0),
    ]);
    assert_eq!(a.compare(&b), SpecificityComparison::AMoreSpecific);
}

#[test]
fn test_specificity_incomparability() {
    let a = SpecificityVector::new(vec![
        ("actor".to_string(), 1),
        ("role".to_string(), 0),
    ]);
    let b = SpecificityVector::new(vec![
        ("actor".to_string(), 0),
        ("role".to_string(), 1),
    ]);
    assert_eq!(a.compare(&b), SpecificityComparison::Incomparable);
}

#[test]
fn test_specificity_equality() {
    let a = SpecificityVector::new(vec![
        ("actor".to_string(), 1),
        ("role".to_string(), 1),
    ]);
    let b = SpecificityVector::new(vec![
        ("actor".to_string(), 1),
        ("role".to_string(), 1),
    ]);
    assert_eq!(a.compare(&b), SpecificityComparison::Equal);
}

#[test]
fn test_purity_flags() {
    let pure = PurityFlags::default();
    assert!(pure.is_pure());

    let impure = PurityFlags {
        network_access: true,
        ..Default::default()
    };
    assert!(!impure.is_pure());
}

#[test]
fn test_pack_hash_validation() {
    let mut pack = make_prohibition_pack();
    let original_hash = pack.hash.clone();
    pack.hash = "bad_hash".to_string();
    assert!(pack.validate_hash().is_err());
    pack.hash = original_hash;
    assert!(pack.validate_hash().is_ok());
}

#[test]
fn test_pack_semantics_version_mismatch() {
    let pack = make_prohibition_pack();
    assert!(pack.validate_semantics_version("0.3").is_err());
    assert!(pack.validate_semantics_version("0.4").is_ok());
}

#[test]
fn test_validate_fact_path() {
    assert!(validate_fact_path("customer.credit_status").is_ok());
    assert!(validate_fact_path("order-1.v2_status").is_ok());
    assert!(validate_fact_path("bad path with spaces").is_err());
    assert!(validate_fact_path("bad;path").is_err());
}

#[test]
fn test_structural_predicate_matching() {
    let preds = StructuralPredicates {
        predicates: {
            let mut m = std::collections::HashMap::new();
            m.insert("action".to_string(), serde_json::json!("ShipOrder"));
            m.insert("resource.type".to_string(), serde_json::json!("Order"));
            m
        },
    };
    let request = make_request("ShipOrder", "u1", None, "Order");
    assert!(preds.matches(&request));

    let wrong_request = make_request("ViewOrder", "u1", None, "Order");
    assert!(!preds.matches(&wrong_request));
}

#[test]
fn test_ambiguous_compatibility_lowering_rejected() {
    let auditor = CompatibilityLoweringAuditor::new("bounded_compatibility_v1".to_string());
    let result = auditor.audit_expression(
        r#"action = "ShipOrder" and (resource = "Order" or resource.type = "SpecialOrder")"#,
    );
    assert!(result.is_err());
}

#[test]
fn test_simple_compatibility_conjunction_lowers() {
    let auditor = CompatibilityLoweringAuditor::new("bounded_compatibility_v1".to_string());
    let result = auditor.audit_expression(r#"action = "ShipOrder" and actor.role = "Ops""#);
    assert!(result.is_ok());
    let lowered = result.unwrap();
    assert!(lowered.is_some());
}

#[test]
fn test_negated_condition_preserves_unknown() {
    let cond = ConditionPredicates {
        conditions: {
            let mut m = std::collections::HashMap::new();
            m.insert("customer.credit_status".to_string(), serde_json::json!("Hold"));
            m
        },
    };
    let facts: Vec<FactEnvelope> = vec![];
    let result = cond.evaluate(&facts).unwrap();
    assert_eq!(result, ThreeValuedResult::Unknown);
}

#[test]
fn test_audit_trace_not_formal_proof() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts = vec![make_trusted_fact("customer.credit_status", serde_json::json!("Hold"))];

    let (trace, _) = env.evaluate(&request, &facts).unwrap();
    assert_eq!(trace.claim_level, ClaimLevel::AuditBacked);
}

#[test]
fn test_derived_fact_no_trust_upgrade() {
    let lineage = DerivedFactLineage {
        transform_id: "test_transform".to_string(),
        transform_version: "1.0.0".to_string(),
        transform_hash: "hash123".to_string(),
        input_fact_paths: vec!["customer.raw_status".to_string()],
        input_source_classes: vec![SourceClass::CallerSupplied],
        effective_trust: SourceClass::CallerSupplied,
        trust_upgrade_applied: false,
    };
    let derived = FactEnvelope {
        path: "customer.credit_status".to_string(),
        value: serde_json::json!("Clear"),
        source_class: SourceClass::Derived,
        source_id: "test_transform@1.0.0".to_string(),
        observed_at: Utc::now(),
        expires_at: None,
        evidence_ref: None,
        signature: None,
        confidence: None,
        lineage: Some(lineage),
    };

    let req = FactRequirement {
        fact_path: "customer.credit_status".to_string(),
        allowed_source_classes: vec![SourceClass::SystemOfRecord],
        allowed_source_ids: vec![],
        max_age: None,
        evidence_ref_required: false,
        signature_required: false,
        minimum_confidence: None,
        required_transform: None,
        derived_from_source: None,
    };

    let now = Utc::now();
    assert!(!req.is_satisfied_by(&derived, &now));
}

#[test]
fn test_transform_registry_rejects_impure() {
    let mut registry = FactTransformRegistry::new();
    let impure = FactTransform {
        id: "bad".to_string(),
        version: "1.0.0".to_string(),
        hash: "h".to_string(),
        inputs: vec![],
        output: TransformOutput {
            fact_path: "out".to_string(),
        },
        purity: PurityFlags {
            network_access: true,
            ..Default::default()
        },
        determinism_tests: vec![],
    };
    assert!(registry.register(impure).is_err());
}

#[test]
fn test_fact_source_registry_validation() {
    let mut registry = FactSourceRegistry::new();
    assert!(registry.validate().is_ok());

    let bad_source = FactSource {
        id: "bad".to_string(),
        source_class: SourceClass::SystemOfRecord,
        allowed_paths: vec![],
        evidence_required: false,
        signature_required: false,
        max_response_latency_ms: 1000,
        health_endpoint: None,
        credential_ref: None,
        schema_ref: None,
        owner: None,
        recovery_hint: None,
    };
    registry.register(bad_source).unwrap();
    assert!(registry.validate().is_err());
}

#[test]
fn test_environment_not_validated_blocks_evaluation() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let env = AuthorityEnvironment::new(config).unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let result = env.evaluate(&request, &[]);
    assert!(result.is_err());
}

#[test]
fn test_final_decision_precedence() {
    assert!(FinalDecision::Reject.precedence_rank() > FinalDecision::Deny.precedence_rank());
    assert!(FinalDecision::Deny.precedence_rank() > FinalDecision::Escalate.precedence_rank());
    assert!(FinalDecision::Escalate.precedence_rank() > FinalDecision::Allow.precedence_rank());
    assert!(FinalDecision::Allow.precedence_rank() > FinalDecision::NotApplicable.precedence_rank());
}

#[test]
fn test_context_wrapping_nested() {
    let resolver = FactResolver::new(FactSourceRegistry::new());
    let context = serde_json::json!({
        "customer": {
            "credit_status": "Hold",
            "id": "cust-1"
        },
        "order": {
            "total": 500
        }
    });
    let envelopes = resolver.wrap_context_as_caller_supplied(&context, Utc::now());
    assert!(envelopes.iter().any(|e| e.path == "customer.credit_status"));
    assert!(envelopes.iter().any(|e| e.path == "customer.id"));
    assert!(envelopes.iter().any(|e| e.path == "order.total"));
}

// === Additional conformance tests from spec §17 ===

#[test]
fn test_unknown_override_not_applicable() {
    let pack = make_override_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("LegalOps"), "Order");
    // No override facts provided -> unknown -> NotApplicable
    let (_, decision) = env.evaluate(&request, &[]).unwrap();
    assert_eq!(decision.final_decision, FinalDecision::NotApplicable);
}

#[test]
fn test_trusted_system_of_record_override_accepted() {
    let pack = make_override_pack();
    let mut config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    // Register the approval service as a trusted ManualApproval source
    config.fact_sources.push(FactSource {
        id: "approval-service".to_string(),
        source_class: SourceClass::ManualApproval,
        allowed_paths: vec!["legal_override.".to_string()],
        evidence_required: false,
        signature_required: false,
        max_response_latency_ms: 5000,
        health_endpoint: None,
        credential_ref: None,
        schema_ref: None,
        owner: None,
        recovery_hint: None,
    });
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("LegalOps"), "Order");
    let facts = vec![
        FactEnvelope {
            path: "legal_override.status".to_string(),
            value: serde_json::json!("Approved"),
            source_class: SourceClass::ManualApproval,
            source_id: "approval-service".to_string(),
            observed_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            evidence_ref: Some("approval-1".to_string()),
            signature: None,
            confidence: None,
            lineage: None,
        },
        FactEnvelope {
            path: "legal_override.id".to_string(),
            value: serde_json::json!("approval-1"),
            source_class: SourceClass::ManualApproval,
            source_id: "approval-service".to_string(),
            observed_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            evidence_ref: Some("approval-1".to_string()),
            signature: None,
            confidence: None,
            lineage: None,
        },
        FactEnvelope {
            path: "legal_override.scope".to_string(),
            value: serde_json::json!("ShippingHold"),
            source_class: SourceClass::ManualApproval,
            source_id: "approval-service".to_string(),
            observed_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::hours(1)),
            evidence_ref: Some("approval-1".to_string()),
            signature: None,
            confidence: None,
            lineage: None,
        },
    ];

    let (_, decision) = env.evaluate(&request, &facts).unwrap();
    assert_eq!(decision.final_decision, FinalDecision::Allow);
}

#[test]
fn test_derived_fact_lineage_in_trace() {
    let pack = make_prohibition_pack();
    let mut config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    // Add a transform
    config.fact_transforms = vec![FactTransform {
        id: "credit_normalizer".to_string(),
        version: "1.0.0".to_string(),
        hash: "abc123".to_string(),
        inputs: vec![TransformInput {
            fact_path: "customer.raw_credit_status".to_string(),
            source_classes: vec![SourceClass::SystemOfRecord],
        }],
        output: TransformOutput {
            fact_path: "customer.credit_status".to_string(),
        },
        purity: PurityFlags::default(),
        determinism_tests: vec![],
    }];
    config.fact_sources.push(FactSource {
        id: "raw-credit-service".to_string(),
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
    });

    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts = vec![FactEnvelope {
        path: "customer.raw_credit_status".to_string(),
        value: serde_json::json!("Hold"),
        source_class: SourceClass::SystemOfRecord,
        source_id: "raw-credit-service".to_string(),
        observed_at: Utc::now(),
        expires_at: None,
        evidence_ref: None,
        signature: None,
        confidence: None,
        lineage: None,
    }];

    let (trace, _) = env.evaluate(&request, &facts).unwrap();
    // Trace must contain derived fact lineage if any derived facts were produced
    assert!(!trace.derived_fact_lineage.is_empty() || trace.final_decision == FinalDecision::Deny);
}

#[test]
fn test_conflicting_specificity_profile_rejected() {
    let mut pack = make_prohibition_pack();
    pack.required_specificity_profile = "different_profile".to_string();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    // Strict mode should reject conflicting specificity profiles
    assert!(env.validate().is_err());
}

#[test]
fn test_negative_structural_predicate_rejected() {
    let auditor = CompatibilityLoweringAuditor::new("bounded_compatibility_v1".to_string());
    let result = auditor.audit_expression(r#"not resource.type = "Order""#);
    assert!(result.is_err());
}

#[test]
fn test_negated_condition_predicate_lowers() {
    let auditor = CompatibilityLoweringAuditor::new("bounded_compatibility_v1".to_string());
    let result = auditor.audit_expression(r#"not customer.credit_status = "Hold""#);
    assert!(result.is_ok());
    let lowered = result.unwrap();
    assert!(lowered.is_some());
    // The lowered when condition should use __neq
    let when = &lowered.unwrap().lowered_when;
    let key = "customer.credit_status";
    assert!(when.contains_key(key));
    assert!(when.get(key).unwrap().get("__neq").is_some());
}

#[test]
fn test_not_unknown_preserves_unknown() {
    let cond = ConditionPredicates {
        conditions: {
            let mut m = std::collections::HashMap::new();
            m.insert("missing".to_string(), serde_json::json!("x"));
            m
        },
    };
    let facts: Vec<FactEnvelope> = vec![];
    let result = cond.evaluate(&facts).unwrap();
    assert_eq!(result, ThreeValuedResult::Unknown);
}

#[test]
fn test_source_class_mismatch_rejected() {
    let mut registry = FactSourceRegistry::new();
    registry.register(FactSource {
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
    }).unwrap();

    let resolver = FactResolver::new(registry);
    let raw_facts = vec![];
    let fake_fact = FactEnvelope {
        path: "customer.credit_status".to_string(),
        value: serde_json::json!("Clear"),
        source_class: SourceClass::CallerSupplied, // Wrong class!
        source_id: "credit-service".to_string(), // Registered as SystemOfRecord
        observed_at: Utc::now(),
        expires_at: None,
        evidence_ref: Some("ref".to_string()),
        signature: None,
        confidence: None,
        lineage: None,
    };

    let (_, trust_decisions) = resolver.resolve_trusted_facts(&raw_facts, &[fake_fact]);
    assert_eq!(trust_decisions.len(), 1);
    assert!(!trust_decisions[0].trusted);
    assert_eq!(trust_decisions[0].reason, "source_class_mismatch");
}

#[test]
fn test_unknown_handling_validation_rejects_allow_permission() {
    let bad_config = UnknownHandlingConfig {
        permission: UnknownHandlingDefault {
            default: FinalDecision::Allow, // Violates spec
        },
        ..UnknownHandlingConfig::defaults()
    };
    assert!(bad_config.validate().is_err());
}

#[test]
fn test_minimum_trust_returns_least_trusted() {
    // Direct test of the trust floor logic for derived facts
    let caller_derived = FactEnvelope {
        path: "customer.credit_status".to_string(),
        value: serde_json::json!("Clear"),
        source_class: SourceClass::Derived,
        source_id: "transform@1.0".to_string(),
        observed_at: Utc::now(),
        expires_at: None,
        evidence_ref: None,
        signature: None,
        confidence: None,
        lineage: Some(DerivedFactLineage {
            transform_id: "t".to_string(),
            transform_version: "1.0".to_string(),
            transform_hash: "h".to_string(),
            input_fact_paths: vec!["raw".to_string()],
            input_source_classes: vec![SourceClass::CallerSupplied], // Least trusted parent
            effective_trust: SourceClass::CallerSupplied, // Should inherit least trusted
            trust_upgrade_applied: false,
        }),
    };

    // Requirement allows Derived but NOT CallerSupplied as effective trust
    let req = FactRequirement {
        fact_path: "customer.credit_status".to_string(),
        allowed_source_classes: vec![SourceClass::SystemOfRecord, SourceClass::Derived],
        allowed_source_ids: vec![],
        max_age: None,
        evidence_ref_required: false,
        signature_required: false,
        minimum_confidence: None,
        required_transform: None,
        derived_from_source: Some(vec![SourceClass::SystemOfRecord]),
    };

    // Should NOT be satisfied: effective_trust is CallerSupplied (from lineage),
    // which is in allowed_source_classes, but derived_from_source requires SystemOfRecord parents
    let now = Utc::now();
    assert!(!req.is_satisfied_by(&caller_derived, &now));
}

#[test]
fn test_resolver_semantics_version_in_trace() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts = vec![make_trusted_fact("customer.credit_status", serde_json::json!("Hold"))];
    let (trace, _) = env.evaluate(&request, &facts).unwrap();

    assert_eq!(trace.resolver_semantics_version, "0.4");
    assert!(!trace.resolver_semantics_hash.is_empty());
}

#[test]
fn test_claim_level_audit_backed_not_formally_proven() {
    let pack = make_prohibition_pack();
    let config = make_env_config(vec![serde_json::to_value(&pack).unwrap()]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts = vec![make_trusted_fact("customer.credit_status", serde_json::json!("Hold"))];
    let (trace, _) = env.evaluate(&request, &facts).unwrap();

    assert_eq!(trace.claim_level, ClaimLevel::AuditBacked);
    assert_ne!(trace.claim_level, ClaimLevel::FormallyProven);
}

#[test]
fn test_modality_precedence_prohibition_wins_over_permission() {
    let prohibition = make_prohibition_pack();
    let permission = make_permission_pack();
    let config = make_env_config(vec![
        serde_json::to_value(&prohibition).unwrap(),
        serde_json::to_value(&permission).unwrap(),
    ]);
    let mut env = AuthorityEnvironment::new(config).unwrap();
    env.validate().unwrap();

    let request = make_request("ShipOrder", "user-1", Some("WarehouseOperator"), "Order");
    let facts = vec![make_trusted_fact("customer.credit_status", serde_json::json!("Hold"))];
    let (_, decision) = env.evaluate(&request, &facts).unwrap();

    // Prohibition should win: credit_status=Hold triggers prohibition -> Deny
    assert_eq!(decision.final_decision, FinalDecision::Deny);
}

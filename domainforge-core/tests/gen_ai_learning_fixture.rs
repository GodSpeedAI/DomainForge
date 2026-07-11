//! Throwaway generator: writes fixtures/ai_learning/manufacturing_quality/authority/environment.json
//! Run once with: cargo test --test gen_ai_learning_fixture -- --ignored
use domainforge_core::authority::*;
use std::collections::HashMap;

fn permission(
    policy_id: &str,
    priority: i32,
    action: &str,
    role: Option<&str>,
    resource_type: &str,
    requires: Vec<FactRequirement>,
    description: &str,
) -> AuthorityPolicy {
    policy(
        policy_id,
        PolicyModality::Permission,
        priority,
        action,
        role,
        resource_type,
        requires,
        description,
    )
}

#[allow(clippy::too_many_arguments)]
fn policy(
    policy_id: &str,
    modality: PolicyModality,
    priority: i32,
    action: &str,
    role: Option<&str>,
    resource_type: &str,
    requires: Vec<FactRequirement>,
    description: &str,
) -> AuthorityPolicy {
    let mut predicates = HashMap::new();
    predicates.insert("action".to_string(), serde_json::json!(action));
    predicates.insert(
        "resource.type".to_string(),
        serde_json::json!(resource_type),
    );
    if let Some(role) = role {
        predicates.insert("actor.role".to_string(), serde_json::json!(role));
    }
    AuthorityPolicy {
        policy_id: policy_id.to_string(),
        modality,
        priority,
        applies_to: StructuralPredicates { predicates },
        when: None,
        requires_fact: requires,
        semantics_version: "0.4".to_string(),
        override_spec: None,
        obligation_spec: None,
        description: Some(description.to_string()),
        evidence_ref: None,
    }
}

#[test]
#[ignore]
fn generate_fixture_environment() {
    let policies = vec![
        permission(
            "allow_certified_auditor_close_finding",
            50,
            "close_audit_finding",
            Some("CertifiedAuditor"),
            "AuditFinding",
            vec![],
            "A Certified Auditor may close an audit finding.",
        ),
        policy(
            "deny_trainee_close_finding",
            PolicyModality::Prohibition,
            100,
            "close_audit_finding",
            Some("Trainee"),
            "AuditFinding",
            vec![],
            "A Trainee must not close audit findings.",
        ),
        permission(
            "allow_trainee_prepare_evidence",
            50,
            "prepare_audit_evidence",
            Some("Trainee"),
            "AuditEvidence",
            vec![],
            "A Trainee may prepare audit evidence.",
        ),
        permission(
            "allow_ops_manager_approve_extension",
            50,
            "approve_corrective_action_extension",
            Some("OperationsManager"),
            "CorrectiveActionExtension",
            vec![],
            "An Operations Manager may approve a corrective action extension.",
        ),
        permission(
            "escalate_high_risk_close_requires_ehs",
            60,
            "close_audit_finding",
            None,
            "HighRiskSafetyFinding",
            vec![FactRequirement {
                fact_path: "ehs.approval_status".to_string(),
                allowed_source_classes: vec![SourceClass::SystemOfRecord],
                allowed_source_ids: vec!["ehs-system".to_string()],
                max_age: None,
                evidence_ref_required: true,
                signature_required: false,
                minimum_confidence: None,
                required_transform: None,
                derived_from_source: None,
            }],
            "Closing a high-risk safety finding requires EHS approval on record; \
             absent that fact the decision escalates.",
        ),
    ];

    let id = "manufacturing-quality-authority";
    let version = "1.0.0";
    let semantics_version = "0.4";
    let profile = "default";
    let hash = compute_pack_hash(id, version, semantics_version, profile, &policies).unwrap();
    let pack = AuthorityPack {
        id: id.to_string(),
        version: version.to_string(),
        semantics_version: semantics_version.to_string(),
        required_specificity_profile: profile.to_string(),
        policies,
        hash,
        signature: None,
        owner: Some("quality-team@example.com".to_string()),
        created_at: Some("2026-07-02T00:00:00Z".to_string()),
        approved_by: None,
        evidence_ref: None,
    };

    let config = AuthorityEnvironmentConfig {
        resolver_semantics_version: "0.4".to_string(),
        specificity_profile: SpecificityProfile::default_profile(),
        unknown_handling: UnknownHandlingConfig::defaults(),
        fact_sources: vec![FactSource {
            id: "ehs-system".to_string(),
            source_class: SourceClass::SystemOfRecord,
            allowed_paths: vec!["ehs.".to_string()],
            evidence_required: true,
            signature_required: false,
            max_response_latency_ms: 5000,
            health_endpoint: None,
            credential_ref: None,
            schema_ref: None,
            owner: None,
            recovery_hint: None,
            public_key_pem: None,
        }],
        fact_transforms: vec![],
        authority_packs: vec![serde_json::to_value(&pack).unwrap()],
        strict_mode: true,
        compatibility_lowering_version: "bounded_compatibility_v1".to_string(),
        resolver_version: "0.1.0".to_string(),
    };

    // Prove the environment loads, validates, and yields the intended decisions.
    let mut env = AuthorityEnvironment::new(config.clone()).unwrap();
    env.validate().unwrap();
    let mk = |op: &str, role: &str, rtype: &str| AuthorityRequest {
        request_id: "fixture-check".to_string(),
        actor: ActorContext {
            id: format!("{}-1", role.to_lowercase()),
            role: Some(role.to_string()),
            groups: vec![],
            service_account: None,
            agent_identity: None,
        },
        operation: op.to_string(),
        resource: ResourceRef {
            id: None,
            type_: Some(rtype.to_string()),
            extra: Default::default(),
        },
        context: serde_json::json!({}),
        requested_at: chrono::Utc::now(),
        correlation_id: None,
        risk_class: None,
        metadata: Default::default(),
    };
    let cases = [
        (
            "close_audit_finding",
            "CertifiedAuditor",
            "AuditFinding",
            FinalDecision::Allow,
        ),
        (
            "close_audit_finding",
            "Trainee",
            "AuditFinding",
            FinalDecision::Deny,
        ),
        (
            "prepare_audit_evidence",
            "Trainee",
            "AuditEvidence",
            FinalDecision::Allow,
        ),
        (
            "approve_corrective_action_extension",
            "OperationsManager",
            "CorrectiveActionExtension",
            FinalDecision::Allow,
        ),
        (
            "close_audit_finding",
            "CertifiedAuditor",
            "HighRiskSafetyFinding",
            FinalDecision::Escalate,
        ),
        (
            "approve_corrective_action_extension",
            "EHSOfficer",
            "AuditEvidence",
            FinalDecision::NotApplicable,
        ),
    ];
    for (op, role, rtype, expected) in cases {
        let (_, decision) = env.evaluate(&mk(op, role, rtype), &[]).unwrap();
        assert_eq!(
            decision.final_decision, expected,
            "unexpected decision for ({op}, {role}, {rtype}): {}",
            decision.reason_code
        );
    }

    let json = serde_json::to_string_pretty(&config).unwrap();
    let out = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../fixtures/ai_learning/manufacturing_quality/authority/environment.json"
    );
    std::fs::write(out, json + "\n").unwrap();
    println!("wrote {out}");
}

use crate::authority::{
    AuthorityEnvironment as RustAuthorityEnvironment,
    AuthorityEnvironmentConfig as RustAuthorityEnvironmentConfig,
    ClaimLevel as RustClaimLevel, FactEnvelope as RustFactEnvelope,
    FinalDecision as RustFinalDecision, PolicyModality as RustPolicyModality,
    SourceClass as RustSourceClass,
};
use napi_derive::napi;

// =============================================================================
// Enums
// =============================================================================

#[napi]
pub enum FinalDecision {
    Allow,
    Deny,
    Escalate,
    NotApplicable,
    Reject,
}

impl From<FinalDecision> for RustFinalDecision {
    fn from(v: FinalDecision) -> Self {
        match v {
            FinalDecision::Allow => RustFinalDecision::Allow,
            FinalDecision::Deny => RustFinalDecision::Deny,
            FinalDecision::Escalate => RustFinalDecision::Escalate,
            FinalDecision::NotApplicable => RustFinalDecision::NotApplicable,
            FinalDecision::Reject => RustFinalDecision::Reject,
        }
    }
}

impl From<RustFinalDecision> for FinalDecision {
    fn from(v: RustFinalDecision) -> Self {
        match v {
            RustFinalDecision::Allow => FinalDecision::Allow,
            RustFinalDecision::Deny => FinalDecision::Deny,
            RustFinalDecision::Escalate => FinalDecision::Escalate,
            RustFinalDecision::NotApplicable => FinalDecision::NotApplicable,
            RustFinalDecision::Reject => FinalDecision::Reject,
        }
    }
}

#[napi]
pub enum PolicyModality {
    Permission,
    Prohibition,
    Obligation,
    Override,
}

impl From<PolicyModality> for RustPolicyModality {
    fn from(v: PolicyModality) -> Self {
        match v {
            PolicyModality::Permission => RustPolicyModality::Permission,
            PolicyModality::Prohibition => RustPolicyModality::Prohibition,
            PolicyModality::Obligation => RustPolicyModality::Obligation,
            PolicyModality::Override => RustPolicyModality::Override,
        }
    }
}

impl From<RustPolicyModality> for PolicyModality {
    fn from(v: RustPolicyModality) -> Self {
        match v {
            RustPolicyModality::Permission => PolicyModality::Permission,
            RustPolicyModality::Prohibition => PolicyModality::Prohibition,
            RustPolicyModality::Obligation => PolicyModality::Obligation,
            RustPolicyModality::Override => PolicyModality::Override,
        }
    }
}

#[napi]
pub enum SourceClass {
    CallerSupplied,
    RuntimeObserved,
    SystemOfRecord,
    Attested,
    ManualApproval,
    Derived,
    UnknownSource,
}

impl From<SourceClass> for RustSourceClass {
    fn from(v: SourceClass) -> Self {
        match v {
            SourceClass::CallerSupplied => RustSourceClass::CallerSupplied,
            SourceClass::RuntimeObserved => RustSourceClass::RuntimeObserved,
            SourceClass::SystemOfRecord => RustSourceClass::SystemOfRecord,
            SourceClass::Attested => RustSourceClass::Attested,
            SourceClass::ManualApproval => RustSourceClass::ManualApproval,
            SourceClass::Derived => RustSourceClass::Derived,
            SourceClass::UnknownSource => RustSourceClass::UnknownSource,
        }
    }
}

impl From<RustSourceClass> for SourceClass {
    fn from(v: RustSourceClass) -> Self {
        match v {
            RustSourceClass::CallerSupplied => SourceClass::CallerSupplied,
            RustSourceClass::RuntimeObserved => SourceClass::RuntimeObserved,
            RustSourceClass::SystemOfRecord => SourceClass::SystemOfRecord,
            RustSourceClass::Attested => SourceClass::Attested,
            RustSourceClass::ManualApproval => SourceClass::ManualApproval,
            RustSourceClass::Derived => SourceClass::Derived,
            RustSourceClass::UnknownSource => SourceClass::UnknownSource,
        }
    }
}

#[napi]
pub enum ClaimLevel {
    AuditBacked,
    Validated,
    FormallyProven,
}

impl From<ClaimLevel> for RustClaimLevel {
    fn from(v: ClaimLevel) -> Self {
        match v {
            ClaimLevel::AuditBacked => RustClaimLevel::AuditBacked,
            ClaimLevel::Validated => RustClaimLevel::Validated,
            ClaimLevel::FormallyProven => RustClaimLevel::FormallyProven,
        }
    }
}

impl From<RustClaimLevel> for ClaimLevel {
    fn from(v: RustClaimLevel) -> Self {
        match v {
            RustClaimLevel::AuditBacked => ClaimLevel::AuditBacked,
            RustClaimLevel::Validated => ClaimLevel::Validated,
            RustClaimLevel::FormallyProven => ClaimLevel::FormallyProven,
        }
    }
}

// =============================================================================
// Functions
// =============================================================================

/// Evaluate an authority request against a policy environment.
#[napi]
pub fn evaluate_authority(
    config_json: String,
    request_json: String,
    facts_json: Option<String>,
) -> napi::Result<EvaluateAuthorityResult> {
    let facts = facts_json.unwrap_or_else(|| "[]".to_string());

    let config: RustAuthorityEnvironmentConfig = serde_json::from_str(&config_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid config: {}", e)))?;
    let mut env = RustAuthorityEnvironment::new(config)
        .map_err(|e| napi::Error::from_reason(format!("Environment error: {}", e)))?;
    env.validate()
        .map_err(|e| napi::Error::from_reason(format!("Validation error: {}", e)))?;

    let request: crate::authority::AuthorityRequest = serde_json::from_str(&request_json)
        .map_err(|e| napi::Error::from_reason(format!("Invalid request: {}", e)))?;
    let fact_envs: Vec<RustFactEnvelope> = serde_json::from_str(&facts)
        .map_err(|e| napi::Error::from_reason(format!("Invalid facts: {}", e)))?;

    let (trace, decision) = env.evaluate(&request, &fact_envs)
        .map_err(|e| napi::Error::from_reason(format!("Evaluation error: {}", e)))?;

    Ok(EvaluateAuthorityResult {
        trace_json: serde_json::to_string(&trace)
            .map_err(|e| napi::Error::from_reason(format!("Trace serialization error: {}", e)))?,
        decision_json: serde_json::to_string(&decision)
            .map_err(|e| napi::Error::from_reason(format!("Decision serialization error: {}", e)))?,
    })
}

#[napi(object)]
pub struct EvaluateAuthorityResult {
    pub trace_json: String,
    pub decision_json: String,
}

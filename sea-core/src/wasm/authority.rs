use crate::authority::{
    AuthorityEnvironment as RustAuthorityEnvironment,
    AuthorityEnvironmentConfig as RustAuthorityEnvironmentConfig,
    FactEnvelope as RustFactEnvelope,
    FinalDecision as RustFinalDecision, PolicyModality as RustPolicyModality,
    SourceClass as RustSourceClass, ClaimLevel as RustClaimLevel,
};
use wasm_bindgen::prelude::*;

// =============================================================================
// Enums
// =============================================================================

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum FinalDecision {
    Allow = 0,
    Deny = 1,
    Escalate = 2,
    NotApplicable = 3,
    Reject = 4,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum PolicyModality {
    Permission = 0,
    Prohibition = 1,
    Obligation = 2,
    Override = 3,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum SourceClass {
    CallerSupplied = 0,
    RuntimeObserved = 1,
    SystemOfRecord = 2,
    Attested = 3,
    ManualApproval = 4,
    Derived = 5,
    UnknownSource = 6,
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

#[wasm_bindgen]
#[derive(Clone, Copy, Debug)]
pub enum ClaimLevel {
    AuditBacked = 0,
    Validated = 1,
    FormallyProven = 2,
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

/// Evaluate an authority request against a policy environment configuration.
///
/// # Arguments
/// * `config_json` - JSON string of AuthorityEnvironmentConfig
/// * `request_json` - JSON string of AuthorityRequest
/// * `facts_json` - JSON array string of FactEnvelope objects
///
/// # Returns
/// JSON string with `{ "trace": {...}, "decision": {...} }`
#[wasm_bindgen(js_name = "evaluateAuthority")]
pub fn evaluate_authority(
    config_json: &str,
    request_json: &str,
    facts_json: Option<String>,
) -> Result<JsValue, JsError> {
    let config: RustAuthorityEnvironmentConfig = serde_json::from_str(config_json)
        .map_err(|e| JsError::new(&format!("Invalid config: {}", e)))?;
    let mut env = RustAuthorityEnvironment::new(config)
        .map_err(|e| JsError::new(&format!("Environment error: {}", e)))?;
    env.validate()
        .map_err(|e| JsError::new(&format!("Validation error: {}", e)))?;

    let request: crate::authority::AuthorityRequest = serde_json::from_str(request_json)
        .map_err(|e| JsError::new(&format!("Invalid request: {}", e)))?;
    let facts: Vec<RustFactEnvelope> = match facts_json.as_deref() {
        None | Some("") => vec![],
        Some(s) => serde_json::from_str(s)
            .map_err(|e| JsError::new(&format!("Invalid facts: {}", e)))?,
    };

    let (trace, decision) = env.evaluate(&request, &facts)
        .map_err(|e| JsError::new(&format!("Evaluation error: {}", e)))?;

    let result = serde_json::json!({
        "trace": trace,
        "decision": decision,
    });

    Ok(serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsError::new(&format!("Serialization error: {}", e)))?)
}

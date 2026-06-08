use crate::authority::{
    AuthorityEnvironment as RustAuthorityEnvironment,
    AuthorityEnvironmentConfig as RustAuthorityEnvironmentConfig, ClaimLevel as RustClaimLevel,
    FactEnvelope as RustFactEnvelope, FinalDecision as RustFinalDecision,
    PolicyModality as RustPolicyModality, SourceClass as RustSourceClass,
};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

// =============================================================================
// Enums
// =============================================================================

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
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

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
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

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
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

#[pyclass(eq, eq_int)]
#[derive(Clone, PartialEq)]
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
// Structs
// =============================================================================

#[pyclass]
pub struct AuthorityEnvironment {
    inner: RustAuthorityEnvironment,
}

#[pymethods]
impl AuthorityEnvironment {
    #[new]
    fn new(config_json: &str) -> PyResult<Self> {
        let config: RustAuthorityEnvironmentConfig = serde_json::from_str(config_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid config JSON: {}", e)))?;
        let env = RustAuthorityEnvironment::new(config)
            .map_err(|e| PyValueError::new_err(format!("Authority error: {}", e)))?;
        Ok(Self { inner: env })
    }

    fn validate(&mut self) -> PyResult<()> {
        self.inner
            .validate()
            .map_err(|e| PyValueError::new_err(format!("Validation error: {}", e)))
    }

    #[pyo3(signature = (request_json, facts_json = "[]"))]
    fn evaluate(&self, request_json: &str, facts_json: &str) -> PyResult<(String, String)> {
        let request: crate::authority::AuthorityRequest = serde_json::from_str(request_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid request JSON: {}", e)))?;
        let facts: Vec<RustFactEnvelope> = serde_json::from_str(facts_json)
            .map_err(|e| PyValueError::new_err(format!("Invalid facts JSON: {}", e)))?;
        let (trace, decision) = self
            .inner
            .evaluate(&request, &facts)
            .map_err(|e| PyValueError::new_err(format!("Evaluation error: {}", e)))?;
        let trace_json = serde_json::to_string(&trace)
            .map_err(|e| PyValueError::new_err(format!("Trace serialization error: {}", e)))?;
        let decision_json = serde_json::to_string(&decision)
            .map_err(|e| PyValueError::new_err(format!("Decision serialization error: {}", e)))?;
        Ok((trace_json, decision_json))
    }

    fn __repr__(&self) -> String {
        format!("<Py AuthorityEnvironment>")
    }
}

#[pyclass]
#[derive(Clone)]
pub struct AuthorityErrorCode {
    #[pyo3(get)]
    pub code: String,
    #[pyo3(get)]
    pub message: String,
    #[pyo3(get)]
    pub recoverable: bool,
}

#[pymethods]
impl AuthorityErrorCode {
    fn __repr__(&self) -> String {
        format!(
            "AuthorityError(code='{}', message='{}')",
            self.code, self.message
        )
    }
}

/// Evaluate an authority request against a policy environment configuration.
///
/// Args:
///     config_json: JSON string of AuthorityEnvironmentConfig
///     request_json: JSON string of AuthorityRequest
///     facts_json: JSON array of FactEnvelope objects (optional)
///
/// Returns:
///     Tuple of (trace_json, decision_json)
#[pyfunction]
#[pyo3(signature = (config_json, request_json, facts_json = "[]"))]
pub fn evaluate_authority(
    config_json: &str,
    request_json: &str,
    facts_json: &str,
) -> PyResult<(String, String)> {
    let config: RustAuthorityEnvironmentConfig = serde_json::from_str(config_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid config: {}", e)))?;
    let mut env = RustAuthorityEnvironment::new(config)
        .map_err(|e| PyValueError::new_err(format!("Environment error: {}", e)))?;
    env.validate()
        .map_err(|e| PyValueError::new_err(format!("Validation error: {}", e)))?;

    let request: crate::authority::AuthorityRequest = serde_json::from_str(request_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid request: {}", e)))?;
    let facts: Vec<RustFactEnvelope> = serde_json::from_str(facts_json)
        .map_err(|e| PyValueError::new_err(format!("Invalid facts: {}", e)))?;

    let (trace, decision) = env
        .evaluate(&request, &facts)
        .map_err(|e| PyValueError::new_err(format!("Evaluation error: {}", e)))?;

    let trace_json = serde_json::to_string(&trace).unwrap_or_default();
    let decision_json = serde_json::to_string(&decision).unwrap_or_default();
    Ok((trace_json, decision_json))
}

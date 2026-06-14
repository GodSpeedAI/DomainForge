use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthorityErrorCode {
    InvalidAuthorityEnvironment,
    InvalidPolicyPack,
    FactTrustViolation,
    SourceUnavailable,
    InvalidTransform,
    SpecificityConflict,
    AmbiguousCompatibilityLowering,
    TracePersistenceFailure,
    MissingConfigError,
    ParseError,
    SchemaError,
    UnsupportedKindError,
    MissingCredentialError,
    PolicyParseError,
    PolicyEvaluationError,
    InvalidReloadError,
    ConflictingSpecificityProfileError,
    InvalidFactSourceError,
    InvalidTransformError,
    ResolverSemanticsHashMismatch,
    PackHashMismatch,
    InvalidRequest,
    ConflictResolutionError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityError {
    pub code: AuthorityErrorCode,
    pub message: String,
    pub recoverable: bool,
    pub recoverability_hint: Option<String>,
    pub context: Box<serde_json::Value>,
}

impl AuthorityError {
    pub fn new(code: AuthorityErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            recoverable: false,
            recoverability_hint: None,
            context: Box::new(serde_json::json!({})),
        }
    }

    pub fn recoverable(mut self) -> Self {
        self.recoverable = true;
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.recoverability_hint = Some(hint.into());
        self
    }

    pub fn with_context(mut self, ctx: serde_json::Value) -> Self {
        self.context = Box::new(ctx);
        self
    }

    pub fn missing_config(field: &str) -> Self {
        Self::new(
            AuthorityErrorCode::MissingConfigError,
            format!("Missing required config: {}", field),
        )
    }

    pub fn invalid_environment(msg: impl Into<String>) -> Self {
        Self::new(AuthorityErrorCode::InvalidAuthorityEnvironment, msg)
    }

    pub fn fact_trust_violation(fact_path: &str, reason: &str) -> Self {
        Self::new(
            AuthorityErrorCode::FactTrustViolation,
            format!("Fact trust violation for '{}': {}", fact_path, reason),
        )
    }

    pub fn source_unavailable(source_id: &str, reason: &str) -> Self {
        Self::new(
            AuthorityErrorCode::SourceUnavailable,
            format!("Fact source '{}' unavailable: {}", source_id, reason),
        )
        .recoverable()
        .with_hint(format!("Check upstream {} availability", source_id))
    }

    pub fn invalid_transform(id: &str, reason: &str) -> Self {
        Self::new(
            AuthorityErrorCode::InvalidTransform,
            format!("Transform '{}' invalid: {}", id, reason),
        )
    }

    pub fn specificity_conflict(policy_a: &str, policy_b: &str) -> Self {
        Self::new(
            AuthorityErrorCode::SpecificityConflict,
            format!(
                "Incomparable specificity between '{}' and '{}'",
                policy_a, policy_b
            ),
        )
    }

    pub fn ambiguous_lowering(expr: &str) -> Self {
        Self::new(
            AuthorityErrorCode::AmbiguousCompatibilityLowering,
            format!(
                "Ambiguous compatibility expression cannot be safely lowered: {}",
                expr
            ),
        )
    }

    pub fn trace_persistence_failure(reason: &str) -> Self {
        Self::new(
            AuthorityErrorCode::TracePersistenceFailure,
            format!("Trace persistence failed: {}", reason),
        )
        .recoverable()
        .with_hint("Check evidence sink availability and retry")
    }

    pub fn conflicting_specificity_profile(pack_a: &str, pack_b: &str) -> Self {
        Self::new(
            AuthorityErrorCode::ConflictingSpecificityProfileError,
            format!(
                "Packs '{}' and '{}' require incompatible specificity profiles",
                pack_a, pack_b
            ),
        )
    }

    pub fn pack_hash_mismatch(pack_id: &str) -> Self {
        Self::new(
            AuthorityErrorCode::PackHashMismatch,
            format!("Pack '{}' hash mismatch", pack_id),
        )
    }

    pub fn semantics_hash_mismatch() -> Self {
        Self::new(
            AuthorityErrorCode::ResolverSemanticsHashMismatch,
            "Resolver semantics hash mismatch",
        )
    }

    pub fn invalid_request(msg: impl Into<String>) -> Self {
        Self::new(AuthorityErrorCode::InvalidRequest, msg)
    }
}

impl fmt::Display for AuthorityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)
    }
}

impl std::error::Error for AuthorityError {}

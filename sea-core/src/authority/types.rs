use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

use crate::authority::error::AuthorityError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalDecision {
    Allow,
    Deny,
    Escalate,
    NotApplicable,
    Reject,
}

impl FinalDecision {
    pub fn precedence_rank(&self) -> u8 {
        match self {
            Self::Reject => 5,
            Self::Deny => 4,
            Self::Escalate => 3,
            Self::Allow => 2,
            Self::NotApplicable => 1,
        }
    }
}

impl std::fmt::Display for FinalDecision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allow => write!(f, "Allow"),
            Self::Deny => write!(f, "Deny"),
            Self::Escalate => write!(f, "Escalate"),
            Self::NotApplicable => write!(f, "NotApplicable"),
            Self::Reject => write!(f, "Reject"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PolicyModality {
    Permission,
    Prohibition,
    Obligation,
    Override,
}

impl std::fmt::Display for PolicyModality {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Permission => write!(f, "Permission"),
            Self::Prohibition => write!(f, "Prohibition"),
            Self::Obligation => write!(f, "Obligation"),
            Self::Override => write!(f, "Override"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceClass {
    CallerSupplied,
    RuntimeObserved,
    SystemOfRecord,
    Attested,
    ManualApproval,
    Derived,
    UnknownSource,
}

impl std::fmt::Display for SourceClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CallerSupplied => write!(f, "caller_supplied"),
            Self::RuntimeObserved => write!(f, "runtime_observed"),
            Self::SystemOfRecord => write!(f, "system_of_record"),
            Self::Attested => write!(f, "attested"),
            Self::ManualApproval => write!(f, "manual_approval"),
            Self::Derived => write!(f, "derived"),
            Self::UnknownSource => write!(f, "unknown_source"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityRequest {
    pub request_id: String,
    pub actor: ActorContext,
    pub operation: String,
    pub resource: ResourceRef,
    #[serde(default)]
    pub context: serde_json::Value,
    pub requested_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_class: Option<String>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl AuthorityRequest {
    pub fn validate(&self) -> Result<(), AuthorityError> {
        if self.request_id.is_empty() {
            return Err(AuthorityError::invalid_request("request_id is required"));
        }
        if self.actor.id.is_empty() {
            return Err(AuthorityError::invalid_request("actor.id is required"));
        }
        if self.operation.is_empty() {
            return Err(AuthorityError::invalid_request("operation is required"));
        }
        if self.resource.id.is_none() && self.resource.type_.is_none() {
            return Err(AuthorityError::invalid_request(
                "resource must include at least id or type",
            ));
        }
        Ok(())
    }

    pub fn action_hash_input(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.request_id,
            self.actor.id,
            self.operation,
            serde_json::to_string(&self.resource).unwrap_or_default()
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorContext {
    pub id: String,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_account: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_identity: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactEnvelope {
    pub path: String,
    pub value: serde_json::Value,
    pub source_class: SourceClass,
    pub source_id: String,
    pub observed_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage: Option<DerivedFactLineage>,
}

impl FactEnvelope {
    pub fn is_fresh(&self, now: DateTime<Utc>) -> bool {
        match self.expires_at {
            Some(exp) => now < exp,
            None => true,
        }
    }

    pub fn is_trusted(&self) -> bool {
        !matches!(
            self.source_class,
            SourceClass::CallerSupplied | SourceClass::UnknownSource
        )
    }

    pub fn is_caller_supplied(&self) -> bool {
        matches!(self.source_class, SourceClass::CallerSupplied)
    }

    pub fn effective_trust(&self) -> SourceClass {
        if let Some(ref lineage) = self.lineage {
            lineage.effective_trust.clone()
        } else {
            self.source_class.clone()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedFactLineage {
    pub transform_id: String,
    pub transform_version: String,
    pub transform_hash: String,
    pub input_fact_paths: Vec<String>,
    pub input_source_classes: Vec<SourceClass>,
    pub effective_trust: SourceClass,
    pub trust_upgrade_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactRequirement {
    pub fact_path: String,
    pub allowed_source_classes: Vec<SourceClass>,
    #[serde(default)]
    pub allowed_source_ids: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<chrono::Duration>,
    #[serde(default)]
    pub evidence_ref_required: bool,
    #[serde(default)]
    pub signature_required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_transform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from_source: Option<Vec<SourceClass>>,
}

impl FactRequirement {
    pub fn is_satisfied_by(&self, envelope: &FactEnvelope, now: &DateTime<Utc>) -> bool {
        if self.fact_path != envelope.path {
            return false;
        }

        let effective = envelope.effective_trust();
        if !self.allowed_source_classes.contains(&effective) {
            return false;
        }

        if !self.allowed_source_ids.is_empty()
            && !self.allowed_source_ids.contains(&envelope.source_id)
        {
            return false;
        }

        if self.evidence_ref_required && envelope.evidence_ref.is_none() {
            return false;
        }

        if self.signature_required && envelope.signature.is_none() {
            return false;
        }

        if let Some(min_conf) = self.minimum_confidence {
            match envelope.confidence {
                Some(c) if c >= min_conf => {}
                _ => return false,
            }
        }

        // Enforce derived_from_source lineage check for derived facts
        if let Some(ref required_parent_sources) = self.derived_from_source {
            if let Some(ref lineage) = envelope.lineage {
                // Every input parent source must be in the allowed set
                for parent_class in &lineage.input_source_classes {
                    if !required_parent_sources.contains(parent_class) {
                        return false;
                    }
                }
            } else if effective == SourceClass::Derived {
                // Derived fact without lineage info cannot satisfy derived_from_source requirement
                return false;
            }
        }

        // Enforce max_age freshness
        if let Some(max_age) = &self.max_age {
            let oldest_allowed = *now - *max_age;
            if envelope.observed_at < oldest_allowed {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSource {
    pub id: String,
    pub source_class: SourceClass,
    pub allowed_paths: Vec<String>,
    pub evidence_required: bool,
    pub signature_required: bool,
    pub max_response_latency_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub credential_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schema_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub recovery_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactTransform {
    pub id: String,
    pub version: String,
    pub hash: String,
    pub inputs: Vec<TransformInput>,
    pub output: TransformOutput,
    pub purity: PurityFlags,
    pub determinism_tests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformInput {
    pub fact_path: String,
    pub source_classes: Vec<SourceClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformOutput {
    pub fact_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurityFlags {
    pub network_access: bool,
    pub filesystem_access: bool,
    pub clock_access: bool,
    pub random_access: bool,
    pub side_effects: bool,
    pub global_state_access: bool,
}

impl PurityFlags {
    pub fn is_pure(&self) -> bool {
        !self.network_access
            && !self.filesystem_access
            && !self.clock_access
            && !self.random_access
            && !self.side_effects
            && !self.global_state_access
    }
}

impl Default for PurityFlags {
    fn default() -> Self {
        Self {
            network_access: false,
            filesystem_access: false,
            clock_access: false,
            random_access: false,
            side_effects: false,
            global_state_access: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificityProfile {
    pub id: String,
    pub dimensions: Vec<String>,
    pub scoring_rules: serde_json::Value,
    pub hash: String,
}

impl SpecificityProfile {
    pub fn default_profile() -> Self {
        Self {
            id: "default".to_string(),
            dimensions: vec![
                "actor".to_string(),
                "role".to_string(),
                "action".to_string(),
                "resource".to_string(),
                "scope".to_string(),
                "condition".to_string(),
            ],
            scoring_rules: serde_json::json!({}),
            hash: "default_v1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificityVector {
    pub dimensions: Vec<(String, u32)>,
}

impl SpecificityVector {
    pub fn new(dimensions: Vec<(String, u32)>) -> Self {
        Self { dimensions }
    }

    pub fn compare(&self, other: &Self) -> SpecificityComparison {
        let all_dims: std::collections::HashSet<&str> = self
            .dimensions
            .iter()
            .chain(other.dimensions.iter())
            .map(|(name, _)| name.as_str())
            .collect();

        let get_score = |dims: &[(String, u32)], name: &str| -> u32 {
            dims.iter().find(|(n, _)| n == name).map(|(_, s)| *s).unwrap_or(0)
        };

        let a_dominates = all_dims.iter().all(|name| {
            get_score(&self.dimensions, name) >= get_score(&other.dimensions, name)
        });
        let a_strictly_dominates = all_dims.iter().any(|name| {
            get_score(&self.dimensions, name) > get_score(&other.dimensions, name)
        });

        let b_dominates = all_dims.iter().all(|name| {
            get_score(&other.dimensions, name) >= get_score(&self.dimensions, name)
        });
        let b_strictly_dominates = all_dims.iter().any(|name| {
            get_score(&other.dimensions, name) > get_score(&self.dimensions, name)
        });

        match (
            a_dominates,
            a_strictly_dominates,
            b_dominates,
            b_strictly_dominates,
        ) {
            (true, false, true, false) => SpecificityComparison::Equal,
            (true, true, false, _) => SpecificityComparison::AMoreSpecific,
            (false, _, true, true) => SpecificityComparison::BMoreSpecific,
            _ => SpecificityComparison::Incomparable,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpecificityComparison {
    AMoreSpecific,
    BMoreSpecific,
    Equal,
    Incomparable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownHandlingConfig {
    pub permission: UnknownHandlingDefault,
    pub prohibition: UnknownHandlingDefault,
    pub obligation: UnknownHandlingDefault,
    pub override_: UnknownHandlingDefault,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownHandlingDefault {
    pub default: FinalDecision,
}

impl UnknownHandlingConfig {
    pub fn defaults() -> Self {
        Self {
            permission: UnknownHandlingDefault {
                default: FinalDecision::Escalate,
            },
            prohibition: UnknownHandlingDefault {
                default: FinalDecision::Deny,
            },
            obligation: UnknownHandlingDefault {
                default: FinalDecision::Escalate,
            },
            override_: UnknownHandlingDefault {
                default: FinalDecision::NotApplicable,
            },
        }
    }

    pub fn for_modality(&self, modality: &PolicyModality) -> FinalDecision {
        match modality {
            PolicyModality::Permission => self.permission.default,
            PolicyModality::Prohibition => self.prohibition.default,
            PolicyModality::Obligation => self.obligation.default,
            PolicyModality::Override => self.override_.default,
        }
    }

    pub fn validate(&self) -> Result<(), AuthorityError> {
        // Spec §7.10: unknown handling must follow modality-aware defaults
        // Permission unknown MUST NOT be Allow, Prohibition unknown MUST NOT be Allow
        if matches!(self.permission.default, FinalDecision::Allow) {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidAuthorityEnvironment,
                "Unknown permission must not default to Allow",
            ));
        }
        if matches!(self.prohibition.default, FinalDecision::Allow) {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidAuthorityEnvironment,
                "Unknown prohibition must not default to Allow",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityDecision {
    pub decision_id: String,
    pub request_id: String,
    pub final_decision: FinalDecision,
    pub reason_code: String,
    pub trace_ref: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_action_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownDecision {
    pub policy_id: String,
    pub unknown_handling_applied: bool,
    pub unknown_modality: PolicyModality,
    pub unknown_default_result: FinalDecision,
    pub unknown_reason: String,
    pub affected_fact_paths: Vec<String>,
    pub fact_source_ids: Vec<String>,
    pub availability_classification: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operator_action_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactTrustDecision {
    pub fact_path: String,
    pub required_sources: Vec<SourceClass>,
    pub observed_source: SourceClass,
    pub trusted: bool,
    pub reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityLoweringDecision {
    pub policy_id: String,
    pub original_expression: String,
    pub lowered: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionEvaluation {
    pub policy_id: String,
    pub condition_result: ThreeValuedResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unknown_handling_result: Option<FinalDecision>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreeValuedResult {
    True,
    False,
    Unknown,
}

impl From<crate::policy::ThreeValuedBool> for ThreeValuedResult {
    fn from(v: crate::policy::ThreeValuedBool) -> Self {
        match v {
            crate::policy::ThreeValuedBool::True => ThreeValuedResult::True,
            crate::policy::ThreeValuedBool::False => ThreeValuedResult::False,
            crate::policy::ThreeValuedBool::Null => ThreeValuedResult::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimLevel {
    AuditBacked,
    Validated,
    FormallyProven,
}

impl std::fmt::Display for ClaimLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AuditBacked => write!(f, "audit_backed"),
            Self::Validated => write!(f, "validated"),
            Self::FormallyProven => write!(f, "formally_proven"),
        }
    }
}

static FACT_PATH_RE: LazyLock<regex::Regex> =
    LazyLock::new(|| regex::Regex::new(r"^[A-Za-z0-9_.\-]+$").unwrap());

pub fn validate_fact_path(path: &str) -> Result<(), AuthorityError> {
    if !FACT_PATH_RE.is_match(path) {
        return Err(AuthorityError::invalid_request(format!(
            "Invalid fact path '{}': must contain only [A-Za-z0-9_.-]",
            path
        )));
    }
    Ok(())
}

pub fn compute_hash(data: &str) -> String {
    use xxhash_rust::xxh64::xxh64;
    format!("{:016x}", xxh64(data.as_bytes(), 0))
}

pub fn compute_deterministic_hash(data: &str) -> String {
    use xxhash_rust::xxh64::xxh64;
    format!("{:016x}", xxh64(data.as_bytes(), 42))
}

pub fn generate_decision_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

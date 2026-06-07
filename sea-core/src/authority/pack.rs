use serde::{Deserialize, Serialize};

use super::error::AuthorityError;
use super::types::*;

pub fn compute_pack_hash(policies: &[AuthorityPolicy]) -> Result<String, AuthorityError> {
    let canonical = serde_json::to_string(policies).map_err(|e|
        AuthorityError::new(
            super::error::AuthorityErrorCode::InvalidPolicyPack,
            format!("Failed to serialize policies for hash computation: {}", e),
        )
    )?;
    Ok(compute_deterministic_hash(&canonical))
}
use super::policy::AuthorityPolicy;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityPack {
    pub id: String,
    pub version: String,
    pub semantics_version: String,
    pub required_specificity_profile: String,
    pub policies: Vec<AuthorityPolicy>,
    pub hash: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
}

impl AuthorityPack {
    pub fn validate(&self) -> Result<(), AuthorityError> {
        if self.id.is_empty() {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidPolicyPack,
                "Pack id is required",
            ));
        }
        if self.version.is_empty() {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidPolicyPack,
                format!("Pack '{}' version is required", self.id),
            ));
        }

        for policy in &self.policies {
            policy.validate()?;
        }

        Ok(())
    }

    pub fn validate_hash(&self) -> Result<(), AuthorityError> {
        if self.hash.is_empty() {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidPolicyPack,
                format!("Pack '{}' hash is required", self.id),
            ));
        }
        let canonical = serde_json::to_string(&self.policies)
            .map_err(|_| AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidPolicyPack,
                format!("Pack '{}' failed to serialize policies for hash validation", self.id),
            ))?;
        let computed = compute_deterministic_hash(&canonical);
        if self.hash != computed {
            return Err(AuthorityError::pack_hash_mismatch(&self.id));
        }
        Ok(())
    }

    pub fn validate_semantics_version(&self, supported: &str) -> Result<(), AuthorityError> {
        if self.semantics_version != supported {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidPolicyPack,
                format!(
                    "Pack '{}' requires semantics '{}' but '{}' is supported",
                    self.id, self.semantics_version, supported
                ),
            ));
        }
        Ok(())
    }

    pub fn identity_key(&self) -> String {
        format!("{}@{}#{}", self.id, self.version, self.hash)
    }
}

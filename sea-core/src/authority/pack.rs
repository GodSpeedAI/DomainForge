use serde::{Deserialize, Serialize};

use super::error::AuthorityError;
use super::policy::AuthorityPolicy;
use super::types::*;

#[derive(Serialize)]
struct PackHashPayload<'a> {
    id: &'a str,
    version: &'a str,
    semantics_version: &'a str,
    required_specificity_profile: &'a str,
    policies: &'a [AuthorityPolicy],
}

pub fn compute_pack_hash(
    id: &str,
    version: &str,
    semantics_version: &str,
    required_specificity_profile: &str,
    policies: &[AuthorityPolicy],
) -> Result<String, AuthorityError> {
    let payload = PackHashPayload {
        id,
        version,
        semantics_version,
        required_specificity_profile,
        policies,
    };

    let canonical = serde_json::to_string(&payload).map_err(|e| {
        AuthorityError::new(
            super::error::AuthorityErrorCode::InvalidPolicyPack,
            format!(
                "Failed to serialize pack '{}' for hash computation: {}",
                id, e
            ),
        )
    })?;
    Ok(compute_deterministic_hash(&canonical))
}

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

        let payload = PackHashPayload {
            id: &self.id,
            version: &self.version,
            semantics_version: &self.semantics_version,
            required_specificity_profile: &self.required_specificity_profile,
            policies: &self.policies,
        };

        let canonical = serde_json::to_string(&payload).map_err(|e| {
            AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidPolicyPack,
                format!(
                    "Pack '{}' failed to serialize for hash validation: {}",
                    self.id, e
                ),
            )
        })?;
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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::AuthorityError;
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorityPolicy {
    pub policy_id: String,
    pub modality: PolicyModality,
    pub priority: i32,
    pub applies_to: StructuralPredicates,
    #[serde(default)]
    pub when: Option<ConditionPredicates>,
    #[serde(default)]
    pub requires_fact: Vec<FactRequirement>,
    pub semantics_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_spec: Option<OverrideSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obligation_spec: Option<ObligationSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralPredicates {
    #[serde(flatten)]
    pub predicates: HashMap<String, serde_json::Value>,
}

impl StructuralPredicates {
    pub fn matches(&self, request: &AuthorityRequest) -> bool {
        for (key, expected) in &self.predicates {
            let actual = match key.as_str() {
                "action" => serde_json::Value::String(request.operation.clone()),
                "actor.id" => serde_json::Value::String(request.actor.id.clone()),
                "actor.role" => request
                    .actor
                    .role
                    .as_ref()
                    .map(|r| serde_json::Value::String(r.clone()))
                    .unwrap_or(serde_json::Value::Null),
                "resource.id" => request
                    .resource
                    .id
                    .as_ref()
                    .map(|r| serde_json::Value::String(r.clone()))
                    .unwrap_or(serde_json::Value::Null),
                "resource.type" => request
                    .resource
                    .type_
                    .as_ref()
                    .map(|r| serde_json::Value::String(r.clone()))
                    .unwrap_or(serde_json::Value::Null),
                _ => request
                    .metadata
                    .get(key)
                    .cloned()
                    .unwrap_or(serde_json::Value::Null),
            };
            if *expected != actual {
                return false;
            }
        }
        true
    }

    pub fn specificity_score(&self, profile: &SpecificityProfile) -> SpecificityVector {
        let mut dims = Vec::new();
        for dim_name in &profile.dimensions {
            let score = self.predicates.iter().filter(|(k, _)| *k == dim_name || k.starts_with(&format!("{}.", dim_name))).count() as u32;
            dims.push((dim_name.clone(), score));
        }
        SpecificityVector::new(dims)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConditionPredicates {
    #[serde(flatten)]
    pub conditions: HashMap<String, serde_json::Value>,
}

impl ConditionPredicates {
    pub fn evaluate(
        &self,
        facts: &[FactEnvelope],
    ) -> Result<ThreeValuedResult, AuthorityError> {
        if self.conditions.is_empty() {
            return Ok(ThreeValuedResult::True);
        }

        let mut results = Vec::new();
        for (path, expected) in &self.conditions {
            let fact = facts.iter().find(|f| f.path == *path);
            match fact {
                Some(f) if f.value.is_null() => {
                    results.push(ThreeValuedResult::Unknown);
                }
                Some(f) => {
                    if &f.value == expected {
                        results.push(ThreeValuedResult::True);
                    } else {
                        results.push(ThreeValuedResult::False);
                    }
                }
                None => {
                    results.push(ThreeValuedResult::Unknown);
                }
            }
        }

        let all_true = results.iter().all(|r| *r == ThreeValuedResult::True);
        let any_false = results.iter().any(|r| *r == ThreeValuedResult::False);
        let any_unknown = results.iter().any(|r| *r == ThreeValuedResult::Unknown);

        if any_false {
            Ok(ThreeValuedResult::False)
        } else if any_unknown {
            Ok(ThreeValuedResult::Unknown)
        } else if all_true {
            Ok(ThreeValuedResult::True)
        } else {
            Ok(ThreeValuedResult::Unknown)
        }
    }

    pub fn condition_specificity_score(&self, profile: &SpecificityProfile) -> u32 {
        let condition_dim = "condition";
        if profile.dimensions.contains(&condition_dim.to_string()) {
            self.conditions.len() as u32
        } else {
            0
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideSpec {
    pub permits: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObligationSpec {
    pub action: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline: Option<String>,
}

impl AuthorityPolicy {
    pub fn validate(&self) -> Result<(), AuthorityError> {
        if self.policy_id.is_empty() {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::PolicyParseError,
                "policy_id is required",
            ));
        }
        if self.modality == PolicyModality::Override && self.override_spec.is_none() {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::PolicyParseError,
                format!("Override policy '{}' must have override_spec", self.policy_id),
            ));
        }
        Ok(())
    }

    pub fn is_candidate(&self, request: &AuthorityRequest) -> bool {
        self.applies_to.matches(request)
    }

    pub fn evaluate_conditions(
        &self,
        facts: &[FactEnvelope],
    ) -> Result<ThreeValuedResult, AuthorityError> {
        match &self.when {
            Some(cond) => cond.evaluate(facts),
            None => Ok(ThreeValuedResult::True),
        }
    }

    pub fn check_fact_requirements(
        &self,
        facts: &[FactEnvelope],
        now: &chrono::DateTime<chrono::Utc>,
    ) -> Vec<(FactRequirement, bool, Option<FactEnvelope>)> {
        self.requires_fact
            .iter()
            .map(|req| {
                let matched = facts.iter().find(|f| req.is_satisfied_by(f, now));
                let satisfied = matched.is_some();
                (req.clone(), satisfied, matched.cloned())
            })
            .collect()
    }

    pub fn compute_specificity(&self, profile: &SpecificityProfile) -> SpecificityVector {
        let mut dims = self.applies_to.specificity_score(profile).dimensions;
        if let Some(ref when) = self.when {
            let cond_idx = dims.iter().position(|(d, _)| d == "condition");
            if let Some(idx) = cond_idx {
                dims[idx].1 += when.condition_specificity_score(profile);
            }
        }
        SpecificityVector::new(dims)
    }
}

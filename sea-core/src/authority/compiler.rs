use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::AuthorityError;
use super::policy::*;
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityExpression {
    pub expression: String,
    pub location: Option<String>,
}

pub struct PolicyCompiler {
    semantics_version: String,
    compatibility_version: String,
}

impl PolicyCompiler {
    pub fn new(semantics_version: String, compatibility_version: String) -> Self {
        Self {
            semantics_version,
            compatibility_version,
        }
    }

    pub fn compile(
        &self,
        raw_policies: Vec<RawPolicy>,
    ) -> Result<Vec<AuthorityPolicy>, AuthorityError> {
        let mut compiled = Vec::new();
        for raw in raw_policies {
            let policy = self.compile_one(raw)?;
            compiled.push(policy);
        }
        Ok(compiled)
    }

    pub fn semantics_version(&self) -> &str {
        &self.semantics_version
    }

    pub fn compatibility_version(&self) -> &str {
        &self.compatibility_version
    }

    fn compile_one(&self, raw: RawPolicy) -> Result<AuthorityPolicy, AuthorityError> {
        let applies_to = self.parse_structural_predicates(&raw.applies_to)?;
        let when = if raw.when.is_empty() {
            None
        } else {
            Some(self.parse_condition_predicates(&raw.when)?)
        };
        let requires_fact = self.parse_fact_requirements(&raw.requires_fact)?;

        Ok(AuthorityPolicy {
            policy_id: raw.policy_id,
            modality: raw.modality,
            priority: raw.priority,
            applies_to,
            when,
            requires_fact,
            semantics_version: self.semantics_version.clone(),
            override_spec: raw.override_spec,
            obligation_spec: raw.obligation_spec,
            description: raw.description,
            evidence_ref: raw.evidence_ref,
        })
    }

    fn parse_structural_predicates(
        &self,
        raw: &HashMap<String, serde_json::Value>,
    ) -> Result<StructuralPredicates, AuthorityError> {
        for key in raw.keys() {
            validate_fact_path(key)?;
        }
        Ok(StructuralPredicates {
            predicates: raw.clone(),
        })
    }

    fn parse_condition_predicates(
        &self,
        raw: &HashMap<String, serde_json::Value>,
    ) -> Result<ConditionPredicates, AuthorityError> {
        for key in raw.keys() {
            validate_fact_path(key)?;
        }
        Ok(ConditionPredicates {
            conditions: raw.clone(),
        })
    }

    fn parse_fact_requirements(
        &self,
        raw: &[RawFactRequirement],
    ) -> Result<Vec<FactRequirement>, AuthorityError> {
        let mut requirements = Vec::new();
        for req in raw {
            validate_fact_path(&req.fact_path)?;
            requirements.push(FactRequirement {
                fact_path: req.fact_path.clone(),
                allowed_source_classes: req.allowed_source_classes.clone(),
                allowed_source_ids: req.allowed_source_ids.clone().unwrap_or_default(),
                max_age: req
                    .max_age
                    .as_ref()
                    .map(|s| parse_duration(s))
                    .transpose()?,
                evidence_ref_required: req.evidence_ref_required.unwrap_or(false),
                signature_required: req.signature_required.unwrap_or(false),
                minimum_confidence: req.minimum_confidence,
                required_transform: req.required_transform.clone(),
                derived_from_source: req.derived_from_source.clone(),
            });
        }
        Ok(requirements)
    }
}

fn parse_duration(s: &str) -> Result<chrono::Duration, AuthorityError> {
    let s = s.trim();
    let seconds = if s.ends_with('s') {
        s.trim_end_matches('s').parse::<i64>().ok()
    } else if s.ends_with('m') {
        s.trim_end_matches('m').parse::<i64>().ok().map(|v| v * 60)
    } else if s.ends_with('h') {
        s.trim_end_matches('h')
            .parse::<i64>()
            .ok()
            .map(|v| v * 3600)
    } else if s.ends_with('d') {
        s.trim_end_matches('d')
            .parse::<i64>()
            .ok()
            .map(|v| v * 86400)
    } else {
        s.parse::<i64>().ok().map(|v| v * 3600) // default hours
    };
    seconds
        .map(chrono::Duration::seconds)
        .ok_or_else(|| AuthorityError::invalid_environment(format!("Invalid duration: '{}'", s)))
}

pub struct CompatibilityLoweringAuditor {
    version: String,
}

impl CompatibilityLoweringAuditor {
    pub fn new(version: String) -> Result<Self, AuthorityError> {
        if version.trim().is_empty() {
            return Err(AuthorityError::invalid_environment(
                "Compatibility lowering version is required",
            ));
        }

        Ok(Self { version })
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn audit_expression(&self, expr: &str) -> Result<Option<LoweredPolicy>, AuthorityError> {
        let expr_lc = expr.to_lowercase();

        // Reject all structural OR as ambiguous (spec §14.1.7, §17.7)
        if expr_lc.contains(" or ") {
            return Err(AuthorityError::ambiguous_lowering(expr));
        }

        // Handle negated conditions: `not X = "Y"` lowers to when condition
        // Reject negative structural predicates (spec §14.1.7)
        if let Some(inner) = expr_lc.strip_prefix("not ") {
            let inner_trimmed = inner.trim();
            // Reject if the negated expression targets structural keys
            if inner_trimmed.starts_with("resource.")
                || inner_trimmed.starts_with("actor.")
                || inner_trimmed.starts_with("action")
            {
                return Err(AuthorityError::ambiguous_lowering(expr));
            }
            // Lower `not fact.path = "value"` to when condition with negated comparison
            if let Some(eq_pos) = inner_trimmed.find(" = ") {
                let key = expr[4..][..eq_pos].trim().to_string();
                let val = expr[4..][eq_pos + 3..].trim().trim_matches('"').to_string();
                return Ok(Some(LoweredPolicy {
                    original: expr.to_string(),
                    lowered_applies_to: HashMap::new(),
                    lowered_when: {
                        let mut m = HashMap::new();
                        m.insert(key, serde_json::json!({"__neq": val}));
                        m
                    },
                }));
            }
            return Err(AuthorityError::ambiguous_lowering(expr));
        }

        if expr_lc.contains(" and ") {
            // Split on case-insensitive " and " boundaries
            let mut parts: Vec<&str> = Vec::new();
            let mut last = 0;
            let lc_bytes = expr_lc.as_bytes();
            let pattern = b" and ";
            let plen = pattern.len();
            for i in 0..=lc_bytes.len().saturating_sub(plen) {
                if &lc_bytes[i..i + plen] == pattern {
                    parts.push(&expr[last..i]);
                    last = i + plen;
                }
            }
            parts.push(&expr[last..]);
            let mut applies_to = HashMap::new();
            let mut when = HashMap::new();
            for part in parts {
                let trimmed = part.trim();
                let trimmed_lc = trimmed.to_lowercase();
                if trimmed_lc.starts_with("resource.")
                    || trimmed_lc.starts_with("actor.")
                    || trimmed_lc.starts_with("action")
                {
                    if let Some((k, v)) = parse_equality(trimmed) {
                        applies_to.insert(k, v);
                    }
                } else if let Some((k, v)) = parse_equality(trimmed) {
                    when.insert(k, v);
                }
            }
            return Ok(Some(LoweredPolicy {
                original: expr.to_string(),
                lowered_applies_to: applies_to,
                lowered_when: when,
            }));
        }

        if let Some((k, v)) = parse_equality(expr) {
            let mut applies_to = HashMap::new();
            applies_to.insert(k, v);
            return Ok(Some(LoweredPolicy {
                original: expr.to_string(),
                lowered_applies_to: applies_to,
                lowered_when: HashMap::new(),
            }));
        }

        Ok(None)
    }
}

fn parse_equality(s: &str) -> Option<(String, serde_json::Value)> {
    let s = s.trim();
    if let Some(eq_pos) = s.find(" = ") {
        let key = s[..eq_pos].trim().to_string();
        let val = s[eq_pos + 3..].trim().trim_matches('"').to_string();
        Some((key, serde_json::Value::String(val)))
    } else {
        None
    }
}

#[derive(Debug, Clone)]
pub struct LoweredPolicy {
    pub original: String,
    pub lowered_applies_to: HashMap<String, serde_json::Value>,
    pub lowered_when: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawPolicy {
    pub policy_id: String,
    pub modality: PolicyModality,
    pub priority: i32,
    #[serde(default)]
    pub applies_to: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub when: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub requires_fact: Vec<RawFactRequirement>,
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
pub struct RawFactRequirement {
    pub fact_path: String,
    pub allowed_source_classes: Vec<SourceClass>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_source_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_ref_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_confidence: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_transform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from_source: Option<Vec<SourceClass>>,
}

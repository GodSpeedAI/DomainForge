use chrono::Utc;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::error::AuthorityError;
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactSourceRegistry {
    pub sources: IndexMap<String, FactSource>,
}

impl FactSourceRegistry {
    pub fn new() -> Self {
        Self {
            sources: IndexMap::new(),
        }
    }

    pub fn register(&mut self, source: FactSource) -> Result<(), AuthorityError> {
        if source.id.is_empty() {
            return Err(AuthorityError::new(
                super::error::AuthorityErrorCode::InvalidFactSourceError,
                "Fact source id is required",
            ));
        }
        self.sources.insert(source.id.clone(), source);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&FactSource> {
        self.sources.get(id)
    }

    pub fn validate(&self) -> Result<(), AuthorityError> {
        for source in self.sources.values() {
            if source.allowed_paths.is_empty() {
                return Err(AuthorityError::new(
                    super::error::AuthorityErrorCode::InvalidFactSourceError,
                    format!("Source '{}' must declare allowed_paths", source.id),
                ));
            }
        }
        Ok(())
    }
}

impl Default for FactSourceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FactResolver {
    registry: FactSourceRegistry,
}

impl FactResolver {
    pub fn new(registry: FactSourceRegistry) -> Self {
        Self { registry }
    }

    pub fn wrap_context_as_caller_supplied(
        &self,
        context: &serde_json::Value,
        observed_at: chrono::DateTime<Utc>,
    ) -> Vec<FactEnvelope> {
        let mut envelopes = Vec::new();
        if let Some(obj) = context.as_object() {
            Self::flatten_context(obj, "", observed_at, &mut envelopes);
        }
        envelopes
    }

    fn flatten_context(
        obj: &serde_json::Map<String, serde_json::Value>,
        prefix: &str,
        observed_at: chrono::DateTime<Utc>,
        envelopes: &mut Vec<FactEnvelope>,
    ) {
        for (key, value) in obj {
            let path = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };
            match value {
                serde_json::Value::Object(inner) => {
                    Self::flatten_context(inner, &path, observed_at, envelopes);
                }
                _ => {
                    envelopes.push(FactEnvelope {
                        path,
                        value: value.clone(),
                        source_class: SourceClass::CallerSupplied,
                        source_id: "caller".to_string(),
                        observed_at,
                        expires_at: None,
                        evidence_ref: None,
                        signature: None,
                        confidence: None,
                        lineage: None,
                    });
                }
            }
        }
    }

    pub fn resolve_trusted_facts(
        &self,
        raw_facts: &[FactEnvelope],
        provided_trusted: &[FactEnvelope],
    ) -> (Vec<FactEnvelope>, Vec<FactTrustDecision>) {
        let mut resolved = raw_facts.to_vec();
        let mut trust_decisions = Vec::new();

        for fact in provided_trusted {
            let source_id = &fact.source_id;
            let _source_class = &fact.source_class;
            let is_registered = self.registry.sources.contains_key(source_id);

            if !is_registered {
                trust_decisions.push(FactTrustDecision {
                    fact_path: fact.path.clone(),
                    required_sources: vec![],
                    observed_source: fact.source_class.clone(),
                    trusted: false,
                    reason: "unregistered_source".to_string(),
                    evidence_ref: fact.evidence_ref.clone(),
                });
                continue;
            }

            let source = self.registry.get(source_id).unwrap();

            // Verify source_class matches the registry entry's declared class
            if fact.source_class != source.source_class {
                trust_decisions.push(FactTrustDecision {
                    fact_path: fact.path.clone(),
                    required_sources: vec![source.source_class.clone()],
                    observed_source: fact.source_class.clone(),
                    trusted: false,
                    reason: "source_class_mismatch".to_string(),
                    evidence_ref: fact.evidence_ref.clone(),
                });
                continue;
            }
            let path_allowed = source.allowed_paths.iter().any(|allowed_path| {
                fact.path == *allowed_path
                    || if allowed_path.ends_with('.') {
                        fact.path.starts_with(allowed_path)
                    } else {
                        fact.path.starts_with(&format!("{}.", allowed_path))
                    }
            });

            if !path_allowed {
                trust_decisions.push(FactTrustDecision {
                    fact_path: fact.path.clone(),
                    required_sources: vec![source.source_class.clone()],
                    observed_source: fact.source_class.clone(),
                    trusted: false,
                    reason: "path_not_allowed".to_string(),
                    evidence_ref: fact.evidence_ref.clone(),
                });
                continue;
            }

            if source.evidence_required && fact.evidence_ref.is_none() {
                trust_decisions.push(FactTrustDecision {
                    fact_path: fact.path.clone(),
                    required_sources: vec![source.source_class.clone()],
                    observed_source: fact.source_class.clone(),
                    trusted: false,
                    reason: "missing_evidence_ref".to_string(),
                    evidence_ref: None,
                });
                continue;
            }

            if source.signature_required && fact.signature.is_none() {
                trust_decisions.push(FactTrustDecision {
                    fact_path: fact.path.clone(),
                    required_sources: vec![source.source_class.clone()],
                    observed_source: fact.source_class.clone(),
                    trusted: false,
                    reason: "missing_signature".to_string(),
                    evidence_ref: fact.evidence_ref.clone(),
                });
                continue;
            }

            let now = Utc::now();
            if !fact.is_fresh(now) {
                trust_decisions.push(FactTrustDecision {
                    fact_path: fact.path.clone(),
                    required_sources: vec![source.source_class.clone()],
                    observed_source: fact.source_class.clone(),
                    trusted: false,
                    reason: "stale_fact".to_string(),
                    evidence_ref: fact.evidence_ref.clone(),
                });
                continue;
            }

            trust_decisions.push(FactTrustDecision {
                fact_path: fact.path.clone(),
                required_sources: vec![source.source_class.clone()],
                observed_source: fact.source_class.clone(),
                trusted: true,
                reason: "trusted".to_string(),
                evidence_ref: fact.evidence_ref.clone(),
            });

            if let Some(pos) = resolved.iter().position(|f| f.path == fact.path) {
                resolved[pos] = fact.clone();
            } else {
                resolved.push(fact.clone());
            }
        }

        (resolved, trust_decisions)
    }
}

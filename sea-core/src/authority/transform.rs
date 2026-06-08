use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use super::error::AuthorityError;
use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactTransformRegistry {
    transforms: IndexMap<String, FactTransform>,
}

impl FactTransformRegistry {
    pub fn new() -> Self {
        Self {
            transforms: IndexMap::new(),
        }
    }

    pub fn register(&mut self, transform: FactTransform) -> Result<(), AuthorityError> {
        if !transform.purity.is_pure() {
            return Err(AuthorityError::invalid_transform(
                &transform.id,
                "Transform must be pure (no network, filesystem, clock, random, side effects, or global state access)",
            ));
        }
        let key = format!("{}@{}", transform.id, transform.version);
        self.transforms.insert(key, transform);
        Ok(())
    }

    pub fn get(&self, id: &str, version: &str) -> Option<&FactTransform> {
        let key = format!("{}@{}", id, version);
        self.transforms.get(&key)
    }

    pub fn validate(&self) -> Result<(), AuthorityError> {
        for transform in self.transforms.values() {
            if !transform.purity.is_pure() {
                return Err(AuthorityError::invalid_transform(
                    &transform.id,
                    "Impure transform found in registry",
                ));
            }
        }
        Ok(())
    }
}

pub struct DerivedFactEngine {
    registry: FactTransformRegistry,
}

impl DerivedFactEngine {
    pub fn new(registry: FactTransformRegistry) -> Self {
        Self { registry }
    }

    pub fn compute_derived_facts(
        &self,
        trusted_facts: &[FactEnvelope],
        transforms_to_apply: &[String],
    ) -> Result<(Vec<FactEnvelope>, Vec<DerivedFactLineage>), AuthorityError> {
        let mut derived = Vec::new();
        let mut lineages = Vec::new();

        for transform_key in transforms_to_apply {
            let parts: Vec<&str> = transform_key.split('@').collect();
            if parts.len() != 2 {
                return Err(AuthorityError::invalid_transform(
                    transform_key,
                    "Transform key must be in format id@version",
                ));
            }
            let transform = self.registry.get(parts[0], parts[1]).ok_or_else(|| {
                AuthorityError::invalid_transform(transform_key, "Transform not registered")
            })?;

            let mut input_values = Vec::new();
            let mut input_source_classes = Vec::new();
            let mut all_inputs_present = true;

            for input in &transform.inputs {
                match trusted_facts.iter().find(|f| f.path == input.fact_path) {
                    Some(fact) => {
                        input_values.push(fact.clone());
                        input_source_classes.push(fact.effective_trust());
                    }
                    None => {
                        all_inputs_present = false;
                        break;
                    }
                }
            }

            if !all_inputs_present {
                continue;
            }

            let effective_trust = minimum_trust(&input_source_classes);

            let output_value = apply_simple_transform(transform, &input_values);

            let lineage = DerivedFactLineage {
                transform_id: transform.id.clone(),
                transform_version: transform.version.clone(),
                transform_hash: transform.hash.clone(),
                input_fact_paths: input_values.iter().map(|f| f.path.clone()).collect(),
                input_source_classes: input_source_classes.clone(),
                effective_trust: effective_trust.clone(),
                trust_upgrade_applied: false,
            };

            let envelope = FactEnvelope {
                path: transform.output.fact_path.clone(),
                value: output_value,
                source_class: SourceClass::Derived,
                source_id: format!("{}@{}", transform.id, transform.version),
                observed_at: chrono::Utc::now(),
                expires_at: input_values.iter().filter_map(|f| f.expires_at).min(),
                evidence_ref: None,
                signature: None,
                confidence: input_values
                    .iter()
                    .filter_map(|f| f.confidence)
                    .reduce(f64::min),
                lineage: Some(lineage.clone()),
            };

            derived.push(envelope);
            lineages.push(lineage);
        }

        Ok((derived, lineages))
    }
}

fn minimum_trust(classes: &[SourceClass]) -> SourceClass {
    let trust_order = [
        SourceClass::SystemOfRecord,
        SourceClass::Attested,
        SourceClass::ManualApproval,
        SourceClass::RuntimeObserved,
        SourceClass::Derived,
        SourceClass::CallerSupplied,
        SourceClass::UnknownSource,
    ];
    if classes.is_empty() {
        return SourceClass::UnknownSource;
    }
    let mut max_idx = 0;
    for class in classes {
        if let Some(idx) = trust_order.iter().position(|t| t == class) {
            max_idx = max_idx.max(idx);
        }
    }
    trust_order[max_idx].clone()
}

fn apply_simple_transform(
    _transform: &FactTransform,
    inputs: &[FactEnvelope],
) -> serde_json::Value {
    if inputs.len() == 1 {
        inputs[0].value.clone()
    } else {
        serde_json::Value::Array(inputs.iter().map(|f| f.value.clone()).collect())
    }
}

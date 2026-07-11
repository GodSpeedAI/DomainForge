//! Generation / validation / coverage reports shared by all three projections.

use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
pub struct GenerationReport {
    pub projection_kind: String,
    pub recipe_ref: String,
    pub recipe_hash: String,
    pub source_model_ref: String,
    pub source_model_hash: String,
    pub generator_version: String,
    pub seed: u64,
    pub started_at: String,
    pub completed_at: String,
    pub artifacts_generated: Vec<String>,
    pub families_generated: Vec<String>,
    pub generation_methods_used: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ValidationReport {
    pub projection_kind: String,
    /// "passed" | "failed"
    pub status: String,
    pub requirements_passed: u64,
    pub requirements_failed: u64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub rejected_artifacts: Vec<RejectedArtifact>,
    pub repair_recommendations: Vec<String>,
    /// Projection-specific counters (rows_generated, duplicate_exact_count, …).
    #[serde(flatten)]
    pub details: Map<String, Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RejectedArtifact {
    pub artifact_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverageReport {
    pub projection_kind: String,
    /// Projection-specific distributions and counts.
    #[serde(flatten)]
    pub details: Map<String, Value>,
    /// Computed, never hand-written: uncovered policies/entities/families.
    pub known_gaps: Vec<String>,
}

pub fn counts_to_value<K: Ord + std::fmt::Display>(counts: &BTreeMap<K, u64>) -> Value {
    let mut map = Map::new();
    for (k, v) in counts {
        map.insert(k.to_string(), Value::from(*v));
    }
    Value::Object(map)
}

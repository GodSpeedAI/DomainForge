//! Shared provenance embedded in every generated record and manifest.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provenance {
    pub source_model_ref: String,
    /// Canonical-JSON SHA-256 of the parsed Graph (semantic_pack::canonical_json).
    pub source_model_hash: String,
    /// ConceptIds / policy ids of every concept this artifact was derived from.
    pub source_refs: Vec<String>,
    /// "template_generation" | "authority_resolver" | "graph_export"
    pub generation_method: String,
    pub generator_version: String,
    pub recipe_ref: String,
    pub recipe_hash: String,
    pub seed: u64,
    /// RFC3339; injectable via --created-at for byte-identical golden runs.
    pub created_at: String,
    pub validation_status: String,
    /// "unreviewed" by default; "not_required" only where a stated rule proves
    /// review unnecessary (e.g. verbatim entity_lookup rows).
    pub review_status: String,
}

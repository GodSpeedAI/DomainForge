use crate::parser::ast::{ProjectionOverride, TargetFormat};
use crate::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionContract {
    pub id: ConceptId,
    pub name: String,
    pub namespace: String,
    pub target_format: TargetFormat,
    pub overrides: Vec<ProjectionOverride>,
}

impl ProjectionContract {
    pub fn new(
        id: ConceptId,
        name: String,
        namespace: String,
        target_format: TargetFormat,
        overrides: Vec<ProjectionOverride>,
    ) -> Self {
        Self {
            id,
            name,
            namespace,
            target_format,
            overrides,
        }
    }
}

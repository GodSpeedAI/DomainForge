use crate::parser::ast::{ProjectionOverride, TargetFormat};
use crate::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectionContract {
    id: ConceptId,
    name: String,
    namespace: String,
    target_format: TargetFormat,
    overrides: Vec<ProjectionOverride>,
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

    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn target_format(&self) -> &TargetFormat {
        &self.target_format
    }

    pub fn overrides(&self) -> &[ProjectionOverride] {
        &self.overrides
    }
}

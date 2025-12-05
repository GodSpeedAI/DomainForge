use crate::parser::ast::{MappingRule, TargetFormat};
use crate::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingContract {
    id: ConceptId,
    name: String,
    namespace: String,
    target_format: TargetFormat,
    rules: Vec<MappingRule>,
}

impl MappingContract {
    pub fn new(
        id: ConceptId,
        name: String,
        namespace: String,
        target_format: TargetFormat,
        rules: Vec<MappingRule>,
    ) -> Self {
        Self {
            id,
            name,
            namespace,
            target_format,
            rules,
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

    pub fn rules(&self) -> &[MappingRule] {
        &self.rules
    }
}

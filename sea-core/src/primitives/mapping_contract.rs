use crate::parser::ast::{MappingRule, TargetFormat};
use crate::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingContract {
    pub id: ConceptId,
    pub name: String,
    pub namespace: String,
    pub target_format: TargetFormat,
    pub rules: Vec<MappingRule>,
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
}

use crate::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    id: ConceptId,
    name: String,
    entity_id: ConceptId,
    namespace: String,
}

impl Role {
    pub fn new(name: impl Into<String>, entity_id: ConceptId, namespace: impl Into<String>) -> Self {
        let name = name.into();
        let namespace = namespace.into();
        let id = ConceptId::from_concept(&namespace, &name);
        
        Self {
            id,
            name,
            entity_id,
            namespace,
        }
    }

    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entity_id(&self) -> &ConceptId {
        &self.entity_id
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

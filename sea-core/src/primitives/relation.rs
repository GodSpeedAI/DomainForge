use crate::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Relation {
    id: ConceptId,
    name: String,
    subject_role_id: Option<ConceptId>,
    object_role_id: Option<ConceptId>,
    via_resource_id: Option<ConceptId>,
    namespace: String,
}

impl Relation {
    pub fn new(
        name: impl Into<String>,
        namespace: impl Into<String>,
        subject_role_id: Option<ConceptId>,
        object_role_id: Option<ConceptId>,
        via_resource_id: Option<ConceptId>,
    ) -> Self {
        let name = name.into();
        let namespace = namespace.into();
        let id = ConceptId::from_concept(&namespace, &name);
        
        Self {
            id,
            name,
            subject_role_id,
            object_role_id,
            via_resource_id,
            namespace,
        }
    }

    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
    
    pub fn subject_role_id(&self) -> Option<&ConceptId> {
        self.subject_role_id.as_ref()
    }
    
    pub fn object_role_id(&self) -> Option<&ConceptId> {
        self.object_role_id.as_ref()
    }
    
    pub fn via_resource_id(&self) -> Option<&ConceptId> {
        self.via_resource_id.as_ref()
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }
}

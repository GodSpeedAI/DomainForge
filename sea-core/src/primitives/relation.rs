use crate::ConceptId;
use serde::{Deserialize, Serialize};

/// Represents a typed fact between two roles with an optional flow linkage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationType {
    id: ConceptId,
    name: String,
    namespace: String,
    subject_role: ConceptId,
    predicate: String,
    object_role: ConceptId,
    via_flow: Option<ConceptId>,
}

impl RelationType {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        namespace: impl Into<String>,
        subject_role: ConceptId,
        predicate: impl Into<String>,
        object_role: ConceptId,
        via_flow: Option<ConceptId>,
    ) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        let id = ConceptId::from_concept(&namespace, &name);

        Self {
            id,
            name,
            namespace,
            subject_role,
            predicate: predicate.into(),
            object_role,
            via_flow,
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

    pub fn subject_role(&self) -> &ConceptId {
        &self.subject_role
    }

    pub fn predicate(&self) -> &str {
        &self.predicate
    }

    pub fn object_role(&self) -> &ConceptId {
        &self.object_role
    }

    pub fn via_flow(&self) -> Option<&ConceptId> {
        self.via_flow.as_ref()
    }
}

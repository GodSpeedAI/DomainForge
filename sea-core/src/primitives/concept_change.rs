use crate::ConceptId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConceptChange {
    id: ConceptId,
    name: String,
    from_version: String,
    to_version: String,
    migration_policy: String,
    breaking_change: bool,
}

impl ConceptChange {
    pub fn new(
        name: impl Into<String>,
        from_version: impl Into<String>,
        to_version: impl Into<String>,
        migration_policy: impl Into<String>,
        breaking_change: bool,
    ) -> Self {
        let name = name.into();
        // Concept changes are usually global or tied to a namespace, but here we assume default namespace for simplicity
        // or we could add namespace support. For now, let's assume global or default.
        let namespace = "default";
        let id = ConceptId::from_concept(namespace, &name);

        Self {
            id,
            name,
            from_version: from_version.into(),
            to_version: to_version.into(),
            migration_policy: migration_policy.into(),
            breaking_change,
        }
    }

    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn from_version(&self) -> &str {
        &self.from_version
    }

    pub fn to_version(&self) -> &str {
        &self.to_version
    }

    pub fn migration_policy(&self) -> &str {
        &self.migration_policy
    }

    pub fn is_breaking_change(&self) -> bool {
        self.breaking_change
    }
}

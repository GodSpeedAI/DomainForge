use crate::ConceptId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Represents a named role that can be assigned to entities within a namespace.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Role {
    id: ConceptId,
    name: String,
    namespace: String,
    attributes: HashMap<String, Value>,
}

impl Role {
    /// Creates a new role within the provided namespace.
    pub fn new_with_namespace(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        let id = ConceptId::from_concept(&namespace, &name);

        Self {
            id,
            name,
            namespace,
            attributes: HashMap::new(),
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

    pub fn attributes(&self) -> &HashMap<String, Value> {
        &self.attributes
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: Value) {
        self.attributes.insert(key.into(), value);
    }
}

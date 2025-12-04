use crate::ConceptId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

const DEFAULT_NAMESPACE: &str = "default";

/// Represents a concrete instance of an Entity with specific field values.
///
/// Instances allow defining specific data objects in the DSL that can be
/// referenced in policies and flows.
///
/// # Examples
///
/// ```
/// use sea_core::primitives::Instance;
/// use serde_json::json;
///
/// let mut vendor = Instance::new_with_namespace("vendor_123", "Vendor", "default");
/// vendor.set_field("name", json!("Acme Corp"));
/// vendor.set_field("credit_limit", json!(50000));
///
/// assert_eq!(vendor.entity_type(), "Vendor");
/// assert_eq!(vendor.get_field("name"), Some(&json!("Acme Corp")));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    id: ConceptId,
    name: String,
    entity_type: String,
    namespace: String,
    fields: HashMap<String, Value>,
}

impl Instance {
    /// Creates a new Instance (default namespace).
    pub fn new(name: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self::new_with_namespace(name, entity_type, DEFAULT_NAMESPACE)
    }

    pub fn new_with_namespace(
        name: impl Into<String>,
        entity_type: impl Into<String>,
        namespace: impl Into<String>,
    ) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        let entity_type = entity_type.into();
        let id = ConceptId::from_concept(&namespace, &format!("{}:{}", entity_type, name));

        Self {
            id,
            name,
            entity_type,
            namespace,
            fields: HashMap::new(),
        }
    }

    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn entity_type(&self) -> &str {
        &self.entity_type
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn set_field(&mut self, key: impl Into<String>, value: Value) {
        self.fields.insert(key.into(), value);
    }

    pub fn get_field(&self, key: &str) -> Option<&Value> {
        self.fields.get(key)
    }

    /// Returns all fields as a reference.
    pub fn fields(&self) -> &HashMap<String, Value> {
        &self.fields
    }

    /// Returns a mutable reference to fields for bulk updates.
    pub fn fields_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.fields
    }
}

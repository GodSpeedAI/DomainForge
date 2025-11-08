use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;
use serde_json::Value;

/// Represents a physical instance of a resource at a specific entity location.
///
/// Instances capture the "WHERE" of resources - they represent specific physical
/// instances of resources located at particular entities.
///
/// # Examples
///
/// ```
/// use sea_core::primitives::{Entity, Resource, Instance};
///
/// let warehouse = Entity::new("Warehouse A");
/// let product = Resource::new("Camera", "units");
///
/// let camera_123 = Instance::new(
///     product.id().clone(),
///     warehouse.id().clone()
/// );
///
/// assert_eq!(camera_123.entity_id(), warehouse.id());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    id: Uuid,
    resource_id: Uuid,
    entity_id: Uuid,
    namespace: Option<String>,
    attributes: HashMap<String, Value>,
}

impl Instance {
    /// Creates a new Instance with a generated UUID.
    pub fn new(resource_id: Uuid, entity_id: Uuid) -> Self {
        Self {
            id: Uuid::new_v4(),
            resource_id,
            entity_id,
            namespace: None,
            attributes: HashMap::new(),
        }
    }

    pub fn new_with_namespace(resource_id: Uuid, entity_id: Uuid, namespace: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            resource_id,
            entity_id,
            namespace: Some(namespace.into()),
            attributes: HashMap::new(),
        }
    }

    pub fn id(&self) -> &Uuid { &self.id }
    pub fn resource_id(&self) -> &Uuid { &self.resource_id }
    pub fn entity_id(&self) -> &Uuid { &self.entity_id }
    pub fn namespace(&self) -> Option<&str> { self.namespace.as_deref() }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: Value) {
        self.attributes.insert(key.into(), value);
    }

    pub fn get_attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }
}

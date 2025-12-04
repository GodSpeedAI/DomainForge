use crate::ConceptId;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

const DEFAULT_NAMESPACE: &str = "default";

/// Represents a physical instance of a resource at a specific entity location.
///
/// Instances capture the "WHERE" of resources - they represent specific physical
/// instances of resources located at particular entities.
///
/// # Examples
///
/// ```
/// use sea_core::primitives::{Entity, Resource, ResourceInstance};
/// use sea_core::units::unit_from_string;
///
/// let warehouse = Entity::new_with_namespace("Warehouse A".to_string(), "default".to_string());
/// let product = Resource::new_with_namespace("Camera", unit_from_string("units"), "default".to_string());
///
/// let camera_123 = ResourceInstance::new(
///     product.id().clone(),
///     warehouse.id().clone()
/// );
///
/// assert_eq!(camera_123.entity_id(), warehouse.id());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceInstance {
    id: ConceptId,
    resource_id: ConceptId,
    entity_id: ConceptId,
    namespace: String,
    attributes: HashMap<String, Value>,
}

impl ResourceInstance {
    /// Creates a new ResourceInstance (default namespace).
    pub fn new(resource_id: ConceptId, entity_id: ConceptId) -> Self {
        Self::new_with_namespace(resource_id, entity_id, DEFAULT_NAMESPACE)
    }

    pub fn new_with_namespace(
        resource_id: ConceptId,
        entity_id: ConceptId,
        namespace: impl Into<String>,
    ) -> Self {
        let namespace = namespace.into();
        // Use UUID v4 for instances to ensure uniqueness (instances are unique occurrences)
        let id = ConceptId::from_uuid(Uuid::new_v4());

        Self {
            id,
            resource_id,
            entity_id,
            namespace,
            attributes: HashMap::new(),
        }
    }

    pub fn id(&self) -> &ConceptId {
        &self.id
    }
    pub fn resource_id(&self) -> &ConceptId {
        &self.resource_id
    }
    pub fn entity_id(&self) -> &ConceptId {
        &self.entity_id
    }
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: Value) {
        self.attributes.insert(key.into(), value);
    }

    pub fn get_attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }

    /// Returns all attributes as a reference.
    pub fn attributes(&self) -> &HashMap<String, Value> {
        &self.attributes
    }
}

use crate::ConceptId;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid;

const DEFAULT_NAMESPACE: &str = "default";

/// Represents a transfer of a resource between two entities.
///
/// Flows are the "MOVEMENT" in enterprise models - they capture
/// the transfer of resources from one entity to another.
///
/// # Examples
///
/// ```
/// use sea_core::primitives::{Entity, Resource, Flow};
/// use sea_core::units::unit_from_string;
/// use rust_decimal::Decimal;
///
/// let warehouse = Entity::new_with_namespace("Warehouse".to_string(), "default".to_string());
/// let factory = Entity::new_with_namespace("Factory".to_string(), "default".to_string());
/// let product = Resource::new_with_namespace("Widget", unit_from_string("units"), "default".to_string());
///
/// let flow = Flow::new(
///     product.id().clone(),
///     warehouse.id().clone(),
///     factory.id().clone(),
///     Decimal::from(100)
/// );
///
/// assert_eq!(flow.quantity(), Decimal::from(100));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Flow {
    id: ConceptId,
    resource_id: ConceptId,
    from_id: ConceptId,
    to_id: ConceptId,
    quantity: Decimal,
    namespace: String,
    attributes: HashMap<String, Value>,
}

impl Flow {
    /// Creates a new Flow using the default namespace.
    pub fn new(
        resource_id: ConceptId,
        from_id: ConceptId,
        to_id: ConceptId,
        quantity: Decimal,
    ) -> Self {
        Self::new_with_namespace(resource_id, from_id, to_id, quantity, DEFAULT_NAMESPACE)
    }

    /// Creates a new Flow with namespace.
    pub fn new_with_namespace(
        resource_id: ConceptId,
        from_id: ConceptId,
        to_id: ConceptId,
        quantity: Decimal,
        namespace: impl Into<String>,
    ) -> Self {
        let namespace = namespace.into();
        // Use UUID v4 for flows to ensure uniqueness (flows are events, not concepts)
        let id = ConceptId::from_uuid(uuid::Uuid::new_v4());

        Self {
            id,
            resource_id,
            from_id,
            to_id,
            quantity,
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
    pub fn from_id(&self) -> &ConceptId {
        &self.from_id
    }
    pub fn to_id(&self) -> &ConceptId {
        &self.to_id
    }
    pub fn quantity(&self) -> Decimal {
        self.quantity
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

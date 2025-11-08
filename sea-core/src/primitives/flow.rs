use serde::{Deserialize, Serialize};
use uuid::Uuid;
use rust_decimal::Decimal;
use std::collections::HashMap;
use serde_json::Value;

/// Represents a transfer of a resource between two entities.
///
/// Flows are the "MOVEMENT" in enterprise models - they capture
/// the transfer of resources from one entity to another.
///
/// # Examples
///
/// ```
/// use sea_core::primitives::{Entity, Resource, Flow};
/// use rust_decimal::Decimal;
///
/// let warehouse = Entity::new("Warehouse");
/// let factory = Entity::new("Factory");
/// let product = Resource::new("Widget", "units");
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
    id: Uuid,
    resource_id: Uuid,
    from_id: Uuid,
    to_id: Uuid,
    quantity: Decimal,
    namespace: Option<String>,
    attributes: HashMap<String, Value>,
}

impl Flow {
    /// Creates a new Flow with a generated UUID.
    pub fn new(resource_id: Uuid, from_id: Uuid, to_id: Uuid, quantity: Decimal) -> Self {
        Self {
            id: Uuid::new_v4(),
            resource_id,
            from_id,
            to_id,
            quantity,
            namespace: None,
            attributes: HashMap::new(),
        }
    }

    pub fn id(&self) -> &Uuid { &self.id }
    pub fn resource_id(&self) -> &Uuid { &self.resource_id }
    pub fn from_id(&self) -> &Uuid { &self.from_id }
    pub fn to_id(&self) -> &Uuid { &self.to_id }
    pub fn quantity(&self) -> Decimal { self.quantity }
    pub fn namespace(&self) -> Option<&str> { self.namespace.as_deref() }

    pub fn set_attribute(&mut self, key: impl Into<String>, value: Value) {
        self.attributes.insert(key.into(), value);
    }

    pub fn get_attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }
}

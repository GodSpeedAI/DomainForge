use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a quantifiable subject of value in enterprise models.
///
/// Resources are the "WHAT" - things that flow between entities,
/// measured in specific units (units, kg, USD, etc.)
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use sea_core::primitives::Resource;
///
/// let product = Resource::new("Camera", "units");
/// assert_eq!(product.name(), "Camera");
/// assert_eq!(product.unit(), "units");
/// assert_eq!(product.namespace(), None);
/// ```
///
/// With namespace:
///
/// ```
/// use sea_core::primitives::Resource;
///
/// let currency = Resource::new_with_namespace("USD", "currency", "finance");
/// assert_eq!(currency.namespace(), Some("finance"));
/// ```
///
/// With custom attributes:
///
/// ```
/// use sea_core::primitives::Resource;
/// use serde_json::json;
///
/// let mut gold = Resource::new("Gold", "kg");
/// gold.set_attribute("purity", json!(0.999));
/// gold.set_attribute("origin", json!("South Africa"));
///
/// assert_eq!(gold.get_attribute("purity"), Some(&json!(0.999)));
/// ```
///
/// Serialization:
///
/// ```
/// use sea_core::primitives::Resource;
///
/// let resource = Resource::new("Silver", "oz");
/// let json = serde_json::to_string(&resource).unwrap();
/// let deserialized: Resource = serde_json::from_str(&json).unwrap();
/// assert_eq!(resource.name(), deserialized.name());
/// assert_eq!(resource.unit(), deserialized.unit());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    id: Uuid,
    name: String,
    unit: String,
    namespace: Option<String>,
    attributes: HashMap<String, Value>,
}

impl Resource {
    /// Creates a new Resource with a generated UUID.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Resource;
    ///
    /// let resource = Resource::new("Camera", "units");
    /// assert_eq!(resource.name(), "Camera");
    /// assert_eq!(resource.unit(), "units");
    /// ```
    pub fn new(name: impl Into<String>, unit: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            unit: unit.into(),
            namespace: None,
            attributes: HashMap::new(),
        }
    }

    /// Creates a new Resource with a specific namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Resource;
    ///
    /// let resource = Resource::new_with_namespace("USD", "currency", "finance");
    /// assert_eq!(resource.namespace(), Some("finance"));
    /// ```
    pub fn new_with_namespace(
        name: impl Into<String>,
        unit: impl Into<String>,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            unit: unit.into(),
            namespace: Some(namespace.into()),
            attributes: HashMap::new(),
        }
    }

    /// Returns the resource's unique identifier.
    pub fn id(&self) -> &Uuid {
        &self.id
    }

    /// Returns the resource's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the resource's unit of measurement.
    pub fn unit(&self) -> &str {
        &self.unit
    }

    /// Returns the resource's namespace, if any.
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// Sets a custom attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Resource;
    /// use serde_json::json;
    ///
    /// let mut resource = Resource::new("Gold", "kg");
    /// resource.set_attribute("purity", json!(0.999));
    /// assert_eq!(resource.get_attribute("purity"), Some(&json!(0.999)));
    /// ```
    pub fn set_attribute(&mut self, key: impl Into<String>, value: Value) {
        self.attributes.insert(key.into(), value);
    }

    /// Gets a custom attribute.
    ///
    /// Returns `None` if the attribute doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Resource;
    /// use serde_json::json;
    ///
    /// let mut resource = Resource::new("Gold", "kg");
    /// resource.set_attribute("purity", json!(0.999));
    /// assert_eq!(resource.get_attribute("purity"), Some(&json!(0.999)));
    /// assert_eq!(resource.get_attribute("missing"), None);
    /// ```
    pub fn get_attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }
}

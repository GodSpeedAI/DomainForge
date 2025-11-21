use crate::units::Unit;
use crate::ConceptId;
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
/// use sea_core::units::{Unit, Dimension};
/// use rust_decimal::Decimal;
///
/// let units = Unit::new("units", "units", Dimension::Count, Decimal::from(1), "units");
/// let product = Resource::new_with_namespace("Camera", units, "default".to_string());
/// assert_eq!(product.name(), "Camera");
/// assert_eq!(product.unit().symbol(), "units");
/// assert_eq!(product.namespace(), "default");
/// ```
///
/// With namespace:
///
/// ```
/// use sea_core::primitives::Resource;
/// use sea_core::units::{Unit, Dimension};
/// use rust_decimal::Decimal;
///
/// let currency = Unit::new("currency", "currency", Dimension::Currency, Decimal::from(1), "currency");
/// let usd = Resource::new_with_namespace("USD", currency, "finance");
/// assert_eq!(usd.namespace(), "finance");
/// ```
///
/// With custom attributes:
///
/// ```
/// use sea_core::primitives::Resource;
/// use sea_core::units::{Unit, Dimension};
/// use rust_decimal::Decimal;
/// use serde_json::json;
///
/// let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1), "kg");
/// let mut gold = Resource::new_with_namespace("Gold", kg, "default".to_string());
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
/// use sea_core::units::{Unit, Dimension};
/// use rust_decimal::Decimal;
///
/// let oz = Unit::new("oz", "ounce", Dimension::Mass, Decimal::new(28349523, 9), "oz");
/// let resource = Resource::new_with_namespace("Silver", oz, "default".to_string());
/// let json = serde_json::to_string(&resource).unwrap();
/// let deserialized: Resource = serde_json::from_str(&json).unwrap();
/// assert_eq!(resource.name(), deserialized.name());
/// assert_eq!(resource.unit().symbol(), deserialized.unit().symbol());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    id: ConceptId,
    name: String,
    unit: Unit,
    namespace: String,
    attributes: HashMap<String, Value>,
}

impl Resource {
    /// Creates a new Resource (deprecated - use new_with_namespace).
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Resource;
    /// use sea_core::units::{Unit, Dimension};
    /// use rust_decimal::Decimal;
    ///
    /// let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1), "kg");
    /// let resource = Resource::new_with_namespace("Camera", kg, "default".to_string());
    /// assert_eq!(resource.name(), "Camera");
    /// assert_eq!(resource.unit().symbol(), "kg");
    /// ```
    #[deprecated(note = "Use new_with_namespace instead")]
    pub fn new(name: impl Into<String>, unit: Unit) -> Self {
        let name = name.into();
        let namespace = "default".to_string();
        Self {
            id: ConceptId::from_concept(&namespace, &name),
            name,
            unit,
            namespace,
            attributes: HashMap::new(),
        }
    }

    /// Creates a new Resource with a specific namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Resource;
    /// use sea_core::units::{Unit, Dimension};
    /// use rust_decimal::Decimal;
    ///
    /// let usd = Unit::new("USD", "US Dollar", Dimension::Currency, Decimal::from(1), "USD");
    /// let resource = Resource::new_with_namespace("USD", usd, "finance");
    /// assert_eq!(resource.namespace(), "finance");
    /// ```
    pub fn new_with_namespace(
        name: impl Into<String>,
        unit: Unit,
        namespace: impl Into<String>,
    ) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        let id = ConceptId::from_concept(&namespace, &name);

        Self {
            id,
            name,
            unit,
            namespace,
            attributes: HashMap::new(),
        }
    }

    /// Creates a Resource from a legacy UUID for backward compatibility.
    pub fn from_legacy_uuid(
        uuid: Uuid,
        name: impl Into<String>,
        unit: Unit,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            id: ConceptId::from_legacy_uuid(uuid),
            name: name.into(),
            unit,
            namespace: namespace.into(),
            attributes: HashMap::new(),
        }
    }

    /// Returns the resource's unique identifier.
    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    /// Returns the resource's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the resource's unit of measurement.
    pub fn unit(&self) -> &Unit {
        &self.unit
    }

    /// Returns the resource's unit symbol (for backward compatibility).
    pub fn unit_symbol(&self) -> &str {
        self.unit.symbol()
    }

    /// Returns the resource's namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Sets a custom attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Resource;
    /// use sea_core::units::unit_from_string;
    /// use serde_json::json;
    ///
    /// let mut resource = Resource::new_with_namespace("Gold", unit_from_string("kg"), "default".to_string());
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
    /// use sea_core::units::unit_from_string;
    /// use serde_json::json;
    ///
    /// let mut resource = Resource::new_with_namespace("Gold", unit_from_string("kg"), "default".to_string());
    /// resource.set_attribute("purity", json!(0.999));
    /// assert_eq!(resource.get_attribute("purity"), Some(&json!(0.999)));
    /// assert_eq!(resource.get_attribute("missing"), None);
    /// ```
    pub fn get_attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }

    /// Returns all attributes as a reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Resource;
    /// use sea_core::units::unit_from_string;
    /// use serde_json::json;
    ///
    /// let mut resource = Resource::new_with_namespace("Gold", unit_from_string("kg"), "default".to_string());
    /// resource.set_attribute("purity", json!(0.999));
    /// resource.set_attribute("origin", json!("South Africa"));
    ///
    /// let attrs = resource.attributes();
    /// assert_eq!(attrs.len(), 2);
    /// assert_eq!(attrs.get("purity"), Some(&json!(0.999)));
    /// ```
    pub fn attributes(&self) -> &HashMap<String, Value> {
        &self.attributes
    }
}

use crate::ConceptId;
use crate::SemanticVersion;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a business actor, location, or organizational unit.
///
/// Entities are the "WHO" in enterprise models - the actors that perform
/// actions, hold resources, or participate in flows.
///
/// # Examples
///
/// Basic usage:
///
/// ```
/// use sea_core::primitives::Entity;
///
/// let warehouse = Entity::new_with_namespace("Main Warehouse", "default");
/// assert_eq!(warehouse.name(), "Main Warehouse");
/// assert_eq!(warehouse.namespace(), "default");
/// ```
///
/// With namespace:
///
/// ```
/// use sea_core::primitives::Entity;
///
/// let warehouse = Entity::new_with_namespace("Warehouse A", "logistics");
/// assert_eq!(warehouse.namespace(), "logistics");
/// ```
///
/// With custom attributes:
///
/// ```
/// use sea_core::primitives::Entity;
/// use serde_json::json;
///
/// let mut factory = Entity::new_with_namespace("Factory", "default");
/// factory.set_attribute("capacity", json!(5000));
/// factory.set_attribute("location", json!("Building 3"));
///
/// assert_eq!(factory.get_attribute("capacity"), Some(&json!(5000)));
/// ```
///
/// Serialization:
///
/// ```
/// use sea_core::primitives::Entity;
///
/// let entity = Entity::new_with_namespace("Test Entity", "default");
/// let json = serde_json::to_string(&entity).unwrap();
/// let deserialized: Entity = serde_json::from_str(&json).unwrap();
/// assert_eq!(entity.name(), deserialized.name());
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Entity {
    id: ConceptId,
    name: String,
    namespace: String,
    version: Option<SemanticVersion>,
    replaces: Option<String>,
    changes: Vec<String>,
    attributes: HashMap<String, Value>,
}

impl Entity {
    /// Creates a new Entity with a generated UUID (deprecated - use new_with_namespace).
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Entity;
    ///
    /// let entity = Entity::new_with_namespace("Warehouse", "default");
    /// assert_eq!(entity.name(), "Warehouse");
    /// ```
    #[deprecated(note = "use new_with_namespace instead")]
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        let namespace = "default".to_string();
        Self {
            id: ConceptId::from_concept(&namespace, &name),
            name,
            namespace,
            version: None,
            replaces: None,
            changes: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Creates a new Entity with a specific namespace.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Entity;
    ///
    /// let entity = Entity::new_with_namespace("Warehouse", "logistics");
    /// assert_eq!(entity.namespace(), "logistics");
    /// ```
    pub fn new_with_namespace(name: impl Into<String>, namespace: impl Into<String>) -> Self {
        let namespace = namespace.into();
        let name = name.into();
        let id = ConceptId::from_concept(&namespace, &name);

        Self {
            id,
            name,
            namespace,
            version: None,
            replaces: None,
            changes: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Sets the entity version.
    pub fn with_version(mut self, version: SemanticVersion) -> Self {
        self.version = Some(version);
        self
    }

    /// Sets the entity that this version replaces.
    pub fn with_replaces(mut self, replaces: String) -> Self {
        self.replaces = Some(replaces);
        self
    }

    /// Sets the list of changes in this version.
    pub fn with_changes(mut self, changes: Vec<String>) -> Self {
        self.changes = changes;
        self
    }

    /// Returns the entity version.
    pub fn version(&self) -> Option<&SemanticVersion> {
        self.version.as_ref()
    }

    /// Returns the entity that this version replaces.
    pub fn replaces(&self) -> Option<&str> {
        self.replaces.as_deref()
    }

    /// Returns the list of changes in this version.
    pub fn changes(&self) -> &[String] {
        &self.changes
    }

    /// Creates an Entity from a legacy UUID for backward compatibility.
    pub fn from_legacy_uuid(
        uuid: Uuid,
        name: impl Into<String>,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            id: ConceptId::from_legacy_uuid(uuid),
            name: name.into(),
            namespace: namespace.into(),
            version: None,
            replaces: None,
            changes: Vec::new(),
            attributes: HashMap::new(),
        }
    }

    /// Returns the entity's unique identifier.
    pub fn id(&self) -> &ConceptId {
        &self.id
    }

    /// Returns the entity's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the entity's namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// Sets a custom attribute.
    ///
    /// # Examples
    ///
    /// ```
    /// use sea_core::primitives::Entity;
    /// use serde_json::json;
    ///
    /// let mut entity = Entity::new_with_namespace("Factory", "default");
    /// entity.set_attribute("capacity", json!(5000));
    /// assert_eq!(entity.get_attribute("capacity"), Some(&json!(5000)));
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
    /// use sea_core::primitives::Entity;
    /// use serde_json::json;
    ///
    /// let mut entity = Entity::new_with_namespace("Factory", "default");
    /// entity.set_attribute("capacity", json!(5000));
    /// assert_eq!(entity.get_attribute("capacity"), Some(&json!(5000)));
    /// assert_eq!(entity.get_attribute("missing"), None);
    /// ```
    pub fn get_attribute(&self, key: &str) -> Option<&Value> {
        self.attributes.get(key)
    }

    /// Returns all attributes as a reference.
    pub fn attributes(&self) -> &HashMap<String, Value> {
        &self.attributes
    }
}

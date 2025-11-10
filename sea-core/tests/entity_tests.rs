use sea_core::primitives::Entity;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_entity_new_creates_valid_uuid() {
    let entity = Entity::new_with_namespace("Test Entity", "default".to_string());
    assert!(Uuid::parse_str(&entity.id().to_string()).is_ok());
}

#[test]
fn test_entity_name_is_stored() {
    let entity = Entity::new_with_namespace("Assembly Line A", "default".to_string());
    assert_eq!(entity.name(), "Assembly Line A");
}

#[test]
fn test_entity_with_namespace() {
    let entity = Entity::new_with_namespace("Warehouse", "logistics");
    assert_eq!(entity.namespace(), "logistics");
}

#[test]
fn test_entity_defaults_to_default_namespace() {
    let entity = Entity::new_with_namespace("Factory", "default".to_string());
    assert_eq!(entity.namespace(), "default");
}

#[test]
fn test_entity_set_attribute() {
    let mut entity = Entity::new_with_namespace("Factory", "default".to_string());
    entity.set_attribute("capacity", json!(5000));
    assert_eq!(entity.get_attribute("capacity"), Some(&json!(5000)));
}

#[test]
fn test_entity_multiple_attributes() {
    let mut entity = Entity::new_with_namespace("Warehouse", "default".to_string());
    entity.set_attribute("capacity_sqft", json!(50000));
    entity.set_attribute("climate_controlled", json!(true));

    assert_eq!(entity.get_attribute("capacity_sqft"), Some(&json!(50000)));
    assert_eq!(
        entity.get_attribute("climate_controlled"),
        Some(&json!(true))
    );
}

#[test]
fn test_entity_get_nonexistent_attribute() {
    let entity = Entity::new_with_namespace("Entity", "default".to_string());
    assert_eq!(entity.get_attribute("missing"), None);
}

#[test]
fn test_entity_serializes_to_json() {
    let entity = Entity::new_with_namespace("Test", "default".to_string());
    let json = serde_json::to_string(&entity).unwrap();
    assert!(json.contains("Test"));
    assert!(json.contains("id"));
}

#[test]
fn test_entity_deserializes_from_json() {
    let entity = Entity::new_with_namespace("Original", "default".to_string());
    let json = serde_json::to_string(&entity).unwrap();
    let deserialized: Entity = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.name(), "Original");
}

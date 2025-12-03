use sea_core::primitives::{Entity, Resource, ResourceInstance};
use sea_core::units::unit_from_string;
use sea_core::ConceptId;

#[test]
fn test_instance_new_stores_references() {
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let camera =
        Resource::new_with_namespace("Camera", unit_from_string("units"), "default".to_string());

    let instance = ResourceInstance::new(camera.id().clone(), warehouse.id().clone());

    assert_eq!(instance.entity_id(), warehouse.id());
    assert_eq!(instance.resource_id(), camera.id());
}

#[test]
fn test_instance_references_are_valid_uuids() {
    let instance = ResourceInstance::new(
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
    );

    assert!(!instance.id().to_string().is_empty());
    assert!(!instance.resource_id().to_string().is_empty());
    assert!(!instance.entity_id().to_string().is_empty());
}

#[test]
fn test_instance_attributes() {
    let mut instance = ResourceInstance::new(
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
    );

    instance.set_attribute("serial_number", serde_json::json!("SN12345"));
    instance.set_attribute("manufacture_date", serde_json::json!("2025-01-15"));

    assert_eq!(
        instance.get_attribute("serial_number"),
        Some(&serde_json::json!("SN12345"))
    );
    assert_eq!(
        instance.get_attribute("manufacture_date"),
        Some(&serde_json::json!("2025-01-15"))
    );
    assert_eq!(instance.get_attribute("nonexistent"), None);
}

#[test]
fn test_instance_serialization() {
    let instance = ResourceInstance::new(
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
    );

    let json = serde_json::to_string(&instance).unwrap();
    let deserialized: ResourceInstance = serde_json::from_str(&json).unwrap();

    assert_eq!(instance, deserialized);
}

use rust_decimal::Decimal;
use sea_core::primitives::{Entity, Flow, Resource};
use sea_core::units::unit_from_string;
use sea_core::ConceptId;
use uuid::Uuid;

#[test]
fn test_flow_new_stores_references() {
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());
    let factory = Entity::new_with_namespace("Factory", "default".to_string());
    let product =
        Resource::new_with_namespace("Widget", unit_from_string("units"), "default".to_string());

    let flow = Flow::new(
        product.id().clone(),
        warehouse.id().clone(),
        factory.id().clone(),
        Decimal::from(100),
    );

    assert_eq!(flow.quantity(), Decimal::from(100));
    assert_eq!(flow.resource_id(), product.id());
    assert_eq!(flow.from_id(), warehouse.id());
    assert_eq!(flow.to_id(), factory.id());
}

#[test]
fn test_flow_references_are_valid_uuids() {
    let flow = Flow::new(
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        Decimal::from(50),
    );

    assert!(Uuid::parse_str(&flow.resource_id().to_string()).is_ok());
    assert!(Uuid::parse_str(&flow.from_id().to_string()).is_ok());
    assert!(Uuid::parse_str(&flow.to_id().to_string()).is_ok());
    assert!(Uuid::parse_str(&flow.id().to_string()).is_ok());
}

#[test]
fn test_flow_attributes() {
    let mut flow = Flow::new(
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        Decimal::from(100),
    );

    flow.set_attribute("priority", serde_json::json!("high"));
    flow.set_attribute("scheduled_date", serde_json::json!("2025-11-15"));

    assert_eq!(
        flow.get_attribute("priority"),
        Some(&serde_json::json!("high"))
    );
    assert_eq!(
        flow.get_attribute("scheduled_date"),
        Some(&serde_json::json!("2025-11-15"))
    );
    assert_eq!(flow.get_attribute("nonexistent"), None);
}

#[test]
fn test_flow_serialization() {
    let flow = Flow::new(
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        ConceptId::from_uuid(uuid::Uuid::new_v4()),
        Decimal::from(250),
    );

    let json = serde_json::to_string(&flow).unwrap();
    let deserialized: Flow = serde_json::from_str(&json).unwrap();

    assert_eq!(flow, deserialized);
}

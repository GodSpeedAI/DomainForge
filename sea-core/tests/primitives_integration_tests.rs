use rust_decimal::Decimal;
use sea_core::primitives::{Entity, Flow, Resource, ResourceInstance};
use sea_core::units::unit_from_string;

#[test]
fn test_complete_supply_chain_model() {
    let supplier = Entity::new_with_namespace("Supplier", "default".to_string());
    let warehouse = Entity::new_with_namespace("Warehouse", "default".to_string());

    let steel =
        Resource::new_with_namespace("Steel", unit_from_string("kg"), "default".to_string());
    let camera =
        Resource::new_with_namespace("Camera", unit_from_string("units"), "default".to_string());

    let steel_shipment = Flow::new(
        steel.id().clone(),
        supplier.id().clone(),
        warehouse.id().clone(),
        Decimal::from(5000),
    );

    let camera_instance = ResourceInstance::new(camera.id().clone(), warehouse.id().clone());

    assert!(steel_shipment.quantity() > Decimal::ZERO);
    assert_eq!(camera_instance.entity_id(), warehouse.id());
}

#[test]
fn test_multi_stage_flow() {
    let supplier = Entity::new_with_namespace("Steel Supplier", "default".to_string());
    let warehouse = Entity::new_with_namespace("Central Warehouse", "default".to_string());
    let manufacturer = Entity::new_with_namespace("Camera Manufacturer", "default".to_string());
    let retailer = Entity::new_with_namespace("Retail Store", "default".to_string());

    let steel =
        Resource::new_with_namespace("Steel", unit_from_string("kg"), "default".to_string());
    let camera_parts = Resource::new_with_namespace(
        "Camera Parts",
        unit_from_string("units"),
        "default".to_string(),
    );
    let finished_camera =
        Resource::new_with_namespace("Camera", unit_from_string("units"), "default".to_string());

    let flow1 = Flow::new(
        steel.id().clone(),
        supplier.id().clone(),
        warehouse.id().clone(),
        Decimal::from(1000),
    );

    let flow2 = Flow::new(
        camera_parts.id().clone(),
        warehouse.id().clone(),
        manufacturer.id().clone(),
        Decimal::from(500),
    );

    let flow3 = Flow::new(
        finished_camera.id().clone(),
        manufacturer.id().clone(),
        retailer.id().clone(),
        Decimal::from(100),
    );

    assert_eq!(flow1.from_id(), supplier.id());
    assert_eq!(flow1.to_id(), warehouse.id());
    assert_eq!(flow2.from_id(), warehouse.id());
    assert_eq!(flow2.to_id(), manufacturer.id());
    assert_eq!(flow3.from_id(), manufacturer.id());
    assert_eq!(flow3.to_id(), retailer.id());
}

#[test]
fn test_instance_tracking_across_entities() {
    let warehouse_a = Entity::new_with_namespace("Warehouse A", "default".to_string());
    let warehouse_b = Entity::new_with_namespace("Warehouse B", "default".to_string());
    let camera = Resource::new_with_namespace(
        "Camera Model X",
        unit_from_string("units"),
        "default".to_string(),
    );

    let instance1 = ResourceInstance::new(camera.id().clone(), warehouse_a.id().clone());

    let instance2 = ResourceInstance::new(camera.id().clone(), warehouse_b.id().clone());

    assert_eq!(instance1.resource_id(), camera.id());
    assert_eq!(instance2.resource_id(), camera.id());
    assert_ne!(instance1.entity_id(), instance2.entity_id());
    assert_ne!(instance1.id(), instance2.id());
}

#[test]
fn test_resource_flow_with_instances() {
    let origin = Entity::new_with_namespace("Manufacturing Plant", "default".to_string());
    let destination = Entity::new_with_namespace("Distribution Center", "default".to_string());
    let product = Resource::new_with_namespace(
        "Smartphone",
        unit_from_string("units"),
        "default".to_string(),
    );

    let transfer = Flow::new(
        product.id().clone(),
        origin.id().clone(),
        destination.id().clone(),
        Decimal::from(50),
    );

    let instance_at_origin = ResourceInstance::new(product.id().clone(), origin.id().clone());

    let instance_at_destination =
        ResourceInstance::new(product.id().clone(), destination.id().clone());

    assert_eq!(transfer.resource_id(), product.id());
    assert_eq!(transfer.from_id(), instance_at_origin.entity_id());
    assert_eq!(transfer.to_id(), instance_at_destination.entity_id());
}

#[test]
fn test_all_primitives_serialization() {
    let entity = Entity::new_with_namespace("Test Entity", "default".to_string());
    let resource = Resource::new_with_namespace(
        "Test Resource",
        unit_from_string("units"),
        "default".to_string(),
    );
    let flow = Flow::new(
        resource.id().clone(),
        entity.id().clone(),
        entity.id().clone(),
        Decimal::from(10),
    );
    let instance = ResourceInstance::new(resource.id().clone(), entity.id().clone());

    let entity_json = serde_json::to_string(&entity).unwrap();
    let resource_json = serde_json::to_string(&resource).unwrap();
    let flow_json = serde_json::to_string(&flow).unwrap();
    let instance_json = serde_json::to_string(&instance).unwrap();

    let entity_deserialized: Entity = serde_json::from_str(&entity_json).unwrap();
    let resource_deserialized: Resource = serde_json::from_str(&resource_json).unwrap();
    let flow_deserialized: Flow = serde_json::from_str(&flow_json).unwrap();
    let instance_deserialized: ResourceInstance = serde_json::from_str(&instance_json).unwrap();

    assert_eq!(entity, entity_deserialized);
    assert_eq!(resource, resource_deserialized);
    assert_eq!(flow, flow_deserialized);
    assert_eq!(instance, instance_deserialized);
}

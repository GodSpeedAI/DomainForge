use sea_core::primitives::{Entity, Resource, Flow, Instance};
use sea_core::units::unit_from_string;
use rust_decimal::Decimal;

#[test]
fn test_complete_supply_chain_model() {
    let supplier = Entity::new("Supplier");
    let warehouse = Entity::new("Warehouse");

    let steel = Resource::new("Steel", unit_from_string("kg"));
    let camera = Resource::new("Camera", unit_from_string("units"));

    let steel_shipment = Flow::new(
        steel.id().clone(),
        supplier.id().clone(),
        warehouse.id().clone(),
        Decimal::from(5000)
    );

    let camera_instance = Instance::new(
        camera.id().clone(),
        warehouse.id().clone()
    );

    assert!(steel_shipment.quantity() > Decimal::ZERO);
    assert_eq!(camera_instance.entity_id(), warehouse.id());
}

#[test]
fn test_multi_stage_flow() {
    let supplier = Entity::new("Steel Supplier");
    let warehouse = Entity::new("Central Warehouse");
    let manufacturer = Entity::new("Camera Manufacturer");
    let retailer = Entity::new("Retail Store");

    let steel = Resource::new("Steel", unit_from_string("kg"));
    let camera_parts = Resource::new("Camera Parts", unit_from_string("units"));
    let finished_camera = Resource::new("Camera", unit_from_string("units"));

    let flow1 = Flow::new(
        steel.id().clone(),
        supplier.id().clone(),
        warehouse.id().clone(),
        Decimal::from(1000)
    );

    let flow2 = Flow::new(
        camera_parts.id().clone(),
        warehouse.id().clone(),
        manufacturer.id().clone(),
        Decimal::from(500)
    );

    let flow3 = Flow::new(
        finished_camera.id().clone(),
        manufacturer.id().clone(),
        retailer.id().clone(),
        Decimal::from(100)
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
    let warehouse_a = Entity::new("Warehouse A");
    let warehouse_b = Entity::new("Warehouse B");
    let camera = Resource::new("Camera Model X", unit_from_string("units"));

    let instance1 = Instance::new(
        camera.id().clone(),
        warehouse_a.id().clone()
    );

    let instance2 = Instance::new(
        camera.id().clone(),
        warehouse_b.id().clone()
    );

    assert_eq!(instance1.resource_id(), camera.id());
    assert_eq!(instance2.resource_id(), camera.id());
    assert_ne!(instance1.entity_id(), instance2.entity_id());
    assert_ne!(instance1.id(), instance2.id());
}

#[test]
fn test_resource_flow_with_instances() {
    let origin = Entity::new("Manufacturing Plant");
    let destination = Entity::new("Distribution Center");
    let product = Resource::new("Smartphone", unit_from_string("units"));

    let transfer = Flow::new(
        product.id().clone(),
        origin.id().clone(),
        destination.id().clone(),
        Decimal::from(50)
    );

    let instance_at_origin = Instance::new(
        product.id().clone(),
        origin.id().clone()
    );

    let instance_at_destination = Instance::new(
        product.id().clone(),
        destination.id().clone()
    );

    assert_eq!(transfer.resource_id(), product.id());
    assert_eq!(transfer.from_id(), instance_at_origin.entity_id());
    assert_eq!(transfer.to_id(), instance_at_destination.entity_id());
}

#[test]
fn test_all_primitives_serialization() {
    let entity = Entity::new("Test Entity");
    let resource = Resource::new("Test Resource", unit_from_string("units"));
    let flow = Flow::new(
        resource.id().clone(),
        entity.id().clone(),
        entity.id().clone(),
        Decimal::from(10)
    );
    let instance = Instance::new(
        resource.id().clone(),
        entity.id().clone()
    );

    let entity_json = serde_json::to_string(&entity).unwrap();
    let resource_json = serde_json::to_string(&resource).unwrap();
    let flow_json = serde_json::to_string(&flow).unwrap();
    let instance_json = serde_json::to_string(&instance).unwrap();

    let entity_deserialized: Entity = serde_json::from_str(&entity_json).unwrap();
    let resource_deserialized: Resource = serde_json::from_str(&resource_json).unwrap();
    let flow_deserialized: Flow = serde_json::from_str(&flow_json).unwrap();
    let instance_deserialized: Instance = serde_json::from_str(&instance_json).unwrap();

    assert_eq!(entity, entity_deserialized);
    assert_eq!(resource, resource_deserialized);
    assert_eq!(flow, flow_deserialized);
    assert_eq!(instance, instance_deserialized);
}

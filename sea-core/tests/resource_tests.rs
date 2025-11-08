use sea_core::primitives::Resource;
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_resource_new_creates_valid_uuid() {
    let resource = Resource::new("Camera", "units");
    assert!(Uuid::parse_str(&resource.id().to_string()).is_ok());
}

#[test]
fn test_resource_name_and_unit_stored() {
    let resource = Resource::new("Steel Beam", "kg");
    assert_eq!(resource.name(), "Steel Beam");
    assert_eq!(resource.unit(), "kg");
}

#[test]
fn test_resource_with_namespace() {
    let resource = Resource::new_with_namespace("USD", "currency", "finance");
    assert_eq!(resource.namespace(), Some("finance"));
}

#[test]
fn test_resource_set_attribute() {
    let mut resource = Resource::new("Gold", "kg");
    resource.set_attribute("purity", json!(0.999));
    assert_eq!(resource.get_attribute("purity"), Some(&json!(0.999)));
}

#[test]
fn test_resource_serializes() {
    let resource = Resource::new("Silver", "oz");
    let json = serde_json::to_string(&resource).unwrap();
    assert!(json.contains("Silver"));
    assert!(json.contains("oz"));
}

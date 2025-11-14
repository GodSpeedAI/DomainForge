use rust_decimal::Decimal;
use sea_core::primitives::Resource;
use sea_core::units::{unit_from_string, Dimension, Unit};
use serde_json::json;
use uuid::Uuid;

#[test]
fn test_resource_new_creates_valid_uuid() {
    let units = unit_from_string("units");
    let resource = Resource::new_with_namespace("Camera", units, "default".to_string());
    assert!(Uuid::parse_str(&resource.id().to_string()).is_ok());
}

#[test]
fn test_resource_name_and_unit_stored() {
    let kg = unit_from_string("kg");
    let resource = Resource::new_with_namespace("Steel Beam", kg, "default".to_string());
    assert_eq!(resource.name(), "Steel Beam");
    assert_eq!(resource.unit().symbol(), "kg");
}

#[test]
fn test_resource_with_namespace() {
    let currency = unit_from_string("USD");
    let resource = Resource::new_with_namespace("USD", currency, "finance");
    assert_eq!(resource.namespace(), "finance");
}

#[test]
fn test_resource_set_attribute() {
    let kg = unit_from_string("kg");
    let mut resource = Resource::new_with_namespace("Gold", kg, "default".to_string());
    resource.set_attribute("purity", json!(0.999));
    assert_eq!(resource.get_attribute("purity"), Some(&json!(0.999)));
}

#[test]
fn test_resource_serializes() {
    let oz = Unit::new(
        "oz",
        "ounce",
        Dimension::Mass,
        Decimal::new(28349523, 9),
        "oz",
    );
    let resource = Resource::new_with_namespace("Silver", oz, "default".to_string());
    let json = serde_json::to_string(&resource).unwrap();
    assert!(json.contains("Silver"));
    assert!(json.contains("oz"));
}

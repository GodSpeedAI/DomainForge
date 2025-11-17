use rust_decimal::Decimal;
use sea_core::primitives::Resource;
use sea_core::units::{Dimension, Unit};

#[test]
fn test_resource_with_unit() {
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1), "kg");
    let gold = Resource::new_with_namespace("Gold", kg, "default".to_string());

    assert_eq!(gold.name(), "Gold");
    assert_eq!(gold.unit().symbol(), "kg");
    assert_eq!(gold.unit().dimension(), &Dimension::Mass);
}

#[test]
fn test_resource_unit_serialization() {
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1), "kg");
    let gold = Resource::new_with_namespace("Gold", kg, "default".to_string());

    let json = serde_json::to_string(&gold).unwrap();
    let deserialized: Resource = serde_json::from_str(&json).unwrap();

    assert_eq!(gold.unit().symbol(), deserialized.unit().symbol());
}

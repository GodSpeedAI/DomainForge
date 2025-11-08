use sea_core::primitives::Resource;
use sea_core::units::{Unit, Dimension};
use rust_decimal::Decimal;

#[test]
fn test_resource_with_unit() {
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1)).unwrap();
    let gold = Resource::new("Gold", kg);

    assert_eq!(gold.name(), "Gold");
    assert_eq!(gold.unit().symbol(), "kg");
    assert_eq!(gold.unit().dimension(), &Dimension::Mass);
}

#[test]
fn test_resource_unit_serialization() {
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1)).unwrap();
    let gold = Resource::new("Gold", kg);

    let json = serde_json::to_string(&gold).unwrap();
    let deserialized: Resource = serde_json::from_str(&json).unwrap();

    assert_eq!(gold.unit().symbol(), deserialized.unit().symbol());
}

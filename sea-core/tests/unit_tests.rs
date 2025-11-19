use rust_decimal::Decimal;
use sea_core::units::{Dimension, Unit, UnitError, UnitRegistry};

#[test]
fn test_dimension_equality() {
    assert_eq!(Dimension::Mass, Dimension::Mass);
    assert_ne!(Dimension::Mass, Dimension::Volume);
}

#[test]
fn test_unit_creation() {
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1), "kg");
    assert_eq!(kg.symbol(), "kg");
    assert_eq!(kg.dimension(), &Dimension::Mass);
    assert_eq!(kg.base_factor(), Decimal::from(1));
}

#[test]
fn test_unit_conversion() {
    let registry = UnitRegistry::default();
    let kg = registry.get_unit("kg").unwrap();
    let g = registry.get_unit("g").unwrap();

    // 1000g = 1kg
    let converted = registry.convert(Decimal::from(1000), g, kg).unwrap();
    assert_eq!(converted, Decimal::from(1));
}

#[test]
fn test_incompatible_unit_conversion() {
    let registry = UnitRegistry::default();
    let kg = registry.get_unit("kg").unwrap();
    let usd = registry.get_unit("USD").unwrap();

    let result = registry.convert(Decimal::from(100), kg, usd);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        UnitError::IncompatibleDimensions { .. }
    ));
}

#[test]
fn test_currency_no_conversion() {
    let registry = UnitRegistry::default();
    let usd = registry.get_unit("USD").unwrap();
    let eur = registry.get_unit("EUR").unwrap();

    // Currencies should not convert without exchange rates
    let result = registry.convert(Decimal::from(100), usd, eur);
    assert!(result.is_err());
}

// REGRESSION TESTS

#[test]
fn test_all_registered_units_have_base() {
    let registry = UnitRegistry::default();
    for (symbol, unit) in registry.units().iter() {
        let base_unit = registry.base_units().get(unit.dimension());
        assert!(
            base_unit.is_some(),
            "Unit {} dimension {:?} has no base unit",
            symbol,
            unit.dimension()
        );
    }
}

#[test]
fn test_conversion_is_reversible() {
    let registry = UnitRegistry::default();
    let kg = registry.get_unit("kg").unwrap();
    let g = registry.get_unit("g").unwrap();

    let original = Decimal::from(5);
    let converted = registry.convert(original, kg, g).unwrap();
    let back = registry.convert(converted, g, kg).unwrap();

    assert_eq!(original, back);
}

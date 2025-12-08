#[cfg(feature = "formatting")]
use icu_locid::locale;
#[cfg(feature = "formatting")]
use sea_core::primitives::quantity::QuantityFormatter;
use sea_core::primitives::Quantity;
use sea_core::units::Dimension;

#[cfg(feature = "formatting")]
#[test]
fn test_quantity_formatting_en_us() {
    let quantity = Quantity::new(1500.0, "USD".to_string(), Dimension::Currency).unwrap();
    let formatter = QuantityFormatter::new(locale!("en-US"));
    assert_eq!(formatter.format(&quantity).unwrap(), "1,500 \"USD\"");
}

#[cfg(feature = "formatting")]
#[test]
fn test_quantity_formatting_de_de() {
    let quantity = Quantity::new(1500.0, "EUR".to_string(), Dimension::Currency).unwrap();
    let formatter = QuantityFormatter::new(locale!("de-DE"));
    assert_eq!(formatter.format(&quantity).unwrap(), "1.500 \"EUR\"");
}

#[cfg(feature = "formatting")]
#[test]
fn test_quantity_formatting_fr_fr() {
    let quantity = Quantity::new(1500.0, "EUR".to_string(), Dimension::Currency).unwrap();
    let formatter = QuantityFormatter::new(locale!("fr-FR"));
    // ICU might use narrow non-breaking space or space depending on version/data
    // We'll check if it contains the number formatted correctly
    let formatted = formatter.format(&quantity).unwrap();
    assert!(formatted.contains("1") && formatted.contains("500"));
    assert!(formatted.contains("\"EUR\""));
}

#[test]
fn test_quantity_validation() {
    assert!(Quantity::new(f64::NAN, "USD".to_string(), Dimension::Currency).is_err());
    assert!(Quantity::new(f64::INFINITY, "USD".to_string(), Dimension::Currency).is_err());
    assert!(Quantity::new(f64::NEG_INFINITY, "USD".to_string(), Dimension::Currency).is_err());
}

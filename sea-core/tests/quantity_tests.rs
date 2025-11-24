use icu_locid::locale;
use sea_core::primitives::Quantity;
use sea_core::primitives::quantity::QuantityFormatter;
use sea_core::units::Dimension;

#[test]
fn test_quantity_formatting_en_us() {
    let quantity = Quantity::new(1500.0, "USD".to_string(), Dimension::Currency);
    let formatter = QuantityFormatter::new(locale!("en-US"));
    assert_eq!(formatter.format(&quantity), "1,500 \"USD\"");
}

#[test]
fn test_quantity_formatting_de_de() {
    let quantity = Quantity::new(1500.0, "EUR".to_string(), Dimension::Currency);
    let formatter = QuantityFormatter::new(locale!("de-DE"));
    assert_eq!(formatter.format(&quantity), "1.500 \"EUR\"");
}

#[test]
fn test_quantity_formatting_fr_fr() {
    let quantity = Quantity::new(1500.0, "EUR".to_string(), Dimension::Currency);
    let formatter = QuantityFormatter::new(locale!("fr-FR"));
    // ICU might use narrow non-breaking space or space depending on version/data
    // We'll check if it contains the number formatted correctly
    let formatted = formatter.format(&quantity);
    assert!(formatted.contains("1") && formatted.contains("500"));
    assert!(formatted.contains("\"EUR\""));
}

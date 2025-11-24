use crate::units::Dimension;
use icu_decimal::FixedDecimalFormatter;
use icu_locid::Locale;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    pub unit: String,
    pub dimension: Dimension,
}

impl Quantity {
    pub fn new(value: f64, unit: String, dimension: Dimension) -> Result<Self, String> {
        if !value.is_finite() {
            return Err(format!("Quantity value must be finite, got {}", value));
        }
        Ok(Self {
            value,
            unit,
            dimension,
        })
    }
}

pub struct QuantityFormatter {
    formatter: FixedDecimalFormatter,
    // locale is kept for future use (e.g. currency formatting)
    #[allow(dead_code)]
    locale: Locale,
}

impl QuantityFormatter {
    pub fn new(locale: Locale) -> Self {
        let formatter = FixedDecimalFormatter::try_new(&(&locale).into(), Default::default())
            .expect("Failed to create FixedDecimalFormatter");
        Self { formatter, locale }
    }

    pub fn format(&self, quantity: &Quantity) -> Result<String, String> {
        // Convert f64 to FixedDecimal for formatting
        // FixedDecimal::from_str handles float string representation
        let decimal = fixed_decimal::FixedDecimal::from_str(&quantity.value.to_string())
            .map_err(|e| format!("Failed to convert quantity value to decimal: {}", e))?;
        let formatted_value = self.formatter.format(&decimal).to_string();
        Ok(format!("{} \"{}\"", formatted_value, quantity.unit))
    }
}

use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Dimension {
    Mass,
    Length,
    Volume,
    Currency,
    Time,
    Temperature,
    Count,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    symbol: String,
    name: String,
    dimension: Dimension,
    base_factor: Decimal,
    base_unit: String,
}

impl Unit {
    pub fn new(
        symbol: impl Into<String>,
        name: impl Into<String>,
        dimension: Dimension,
        base_factor: Decimal,
    ) -> Result<Self, UnitError> {
        if base_factor.is_zero() {
            return Err(UnitError::ZeroBaseFactor);
        }
        let symbol = symbol.into();
        let base_unit = symbol.clone();
        Ok(Self {
            symbol,
            name: name.into(),
            dimension,
            base_factor,
            base_unit,
        })
    }

    pub fn with_base(mut self, base_unit: impl Into<String>) -> Self {
        self.base_unit = base_unit.into();
        self
    }

    pub fn symbol(&self) -> &str { &self.symbol }
    pub fn name(&self) -> &str { &self.name }
    pub fn dimension(&self) -> &Dimension { &self.dimension }
    pub fn base_factor(&self) -> Decimal { self.base_factor }
    pub fn base_unit(&self) -> &str { &self.base_unit }
}

pub trait UnitConversion {
    fn to_base(&self, value: Decimal) -> Decimal;
    fn from_base(&self, value: Decimal) -> Decimal;
}

impl UnitConversion for Unit {
    fn to_base(&self, value: Decimal) -> Decimal {
        value * self.base_factor
    }

    fn from_base(&self, value: Decimal) -> Decimal {
        value / self.base_factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitError {
    UnitNotFound(String),
    IncompatibleDimensions { from: Dimension, to: Dimension },
    ConversionNotDefined { from: String, to: String },
    ZeroBaseFactor,
}

impl std::fmt::Display for UnitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnitError::UnitNotFound(symbol) => write!(f, "Unit not found: {}", symbol),
            UnitError::IncompatibleDimensions { from, to } => {
                write!(f, "Cannot convert between {:?} and {:?}", from, to)
            }
            UnitError::ConversionNotDefined { from, to } => {
                write!(f, "Conversion not defined from {} to {}", from, to)
            }
            UnitError::ZeroBaseFactor => {
                write!(f, "Unit base_factor cannot be zero")
            }
        }
    }
}

impl std::error::Error for UnitError {}

#[derive(Debug, Clone)]
pub struct UnitRegistry {
    units: HashMap<String, Unit>,
    base_units: HashMap<Dimension, String>,
}

impl Default for UnitRegistry {
    fn default() -> Self {
        let mut registry = Self {
            units: HashMap::new(),
            base_units: HashMap::new(),
        };

        // Mass units
        registry.register_base(Dimension::Mass, "kg");
        registry.register(Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1)).expect("valid unit"));
        registry.register(Unit::new("g", "gram", Dimension::Mass, Decimal::new(1, 3)).expect("valid unit").with_base("kg"));
        registry.register(Unit::new("lb", "pound", Dimension::Mass, Decimal::new(45359237, 8)).expect("valid unit").with_base("kg"));

        // Length units
        registry.register_base(Dimension::Length, "m");
        registry.register(Unit::new("m", "meter", Dimension::Length, Decimal::from(1)).expect("valid unit"));
        registry.register(Unit::new("cm", "centimeter", Dimension::Length, Decimal::new(1, 2)).expect("valid unit").with_base("m"));
        registry.register(Unit::new("in", "inch", Dimension::Length, Decimal::new(254, 4)).expect("valid unit").with_base("m"));

        // Volume units
        registry.register_base(Dimension::Volume, "L");
        registry.register(Unit::new("L", "liter", Dimension::Volume, Decimal::from(1)).expect("valid unit"));
        registry.register(Unit::new("mL", "milliliter", Dimension::Volume, Decimal::new(1, 3)).expect("valid unit").with_base("L"));

        // Currency units (no conversion without exchange rates)
        registry.register_base(Dimension::Currency, "USD");
        registry.register(Unit::new("USD", "US Dollar", Dimension::Currency, Decimal::from(1)).expect("valid unit"));
        registry.register(Unit::new("EUR", "Euro", Dimension::Currency, Decimal::from(1)).expect("valid unit"));
        registry.register(Unit::new("GBP", "British Pound", Dimension::Currency, Decimal::from(1)).expect("valid unit"));

        // Time units
        registry.register_base(Dimension::Time, "s");
        registry.register(Unit::new("s", "second", Dimension::Time, Decimal::from(1)).expect("valid unit"));
        registry.register(Unit::new("min", "minute", Dimension::Time, Decimal::from(60)).expect("valid unit").with_base("s"));
        registry.register(Unit::new("h", "hour", Dimension::Time, Decimal::from(3600)).expect("valid unit").with_base("s"));

        // Count (dimensionless)
        registry.register_base(Dimension::Count, "units");
        registry.register(Unit::new("units", "units", Dimension::Count, Decimal::from(1)).expect("valid unit"));
        registry.register(Unit::new("items", "items", Dimension::Count, Decimal::from(1)).expect("valid unit"));

        registry
    }
}

impl UnitRegistry {
    pub fn new() -> Self {
        Self {
            units: HashMap::new(),
            base_units: HashMap::new(),
        }
    }

    pub fn register(&mut self, unit: Unit) {
        self.units.insert(unit.symbol.clone(), unit);
    }

    pub fn register_base(&mut self, dimension: Dimension, base_unit: impl Into<String>) {
        self.base_units.insert(dimension, base_unit.into());
    }

    pub fn get_unit(&self, symbol: &str) -> Result<&Unit, UnitError> {
        self.units.get(symbol)
            .ok_or_else(|| UnitError::UnitNotFound(symbol.to_string()))
    }

    pub fn units(&self) -> &HashMap<String, Unit> {
        &self.units
    }

    pub fn base_units(&self) -> &HashMap<Dimension, String> {
        &self.base_units
    }

    pub fn convert(&self, value: Decimal, from: &Unit, to: &Unit) -> Result<Decimal, UnitError> {
        // Check dimension compatibility
        if from.dimension != to.dimension {
            return Err(UnitError::IncompatibleDimensions {
                from: from.dimension.clone(),
                to: to.dimension.clone(),
            });
        }

        // Special case: Currency requires exchange rates
        if matches!(from.dimension, Dimension::Currency) && from.symbol != to.symbol {
            return Err(UnitError::ConversionNotDefined {
                from: from.symbol.clone(),
                to: to.symbol.clone(),
            });
        }

        // Convert to base unit, then to target using trait
        let in_base = from.to_base(value);
        let in_target = to.from_base(in_base);

        Ok(in_target)
    }
}

/// Helper function to get a Unit from a string symbol, using the default registry
/// Returns a Count-based unit if the symbol is not found
pub fn unit_from_string(symbol: impl Into<String>) -> Unit {
    let symbol = symbol.into();
    let registry = UnitRegistry::default();

    registry.get_unit(&symbol)
        .map(|u| u.clone())
        .unwrap_or_else(|_| {
            // Default to Count dimension for unknown units
            Unit::new(symbol.clone(), symbol.clone(), Dimension::Count, Decimal::from(1))
                .expect("default unit should have non-zero base_factor")
        })
}

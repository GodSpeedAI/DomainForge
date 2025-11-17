use rust_decimal::Decimal;
use rust_decimal::prelude::FromPrimitive;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;

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

impl std::fmt::Display for Unit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.symbol)
    }
}

impl Unit {
    pub fn new(
        symbol: impl Into<String>,
        name: impl Into<String>,
        dimension: Dimension,
        base_factor: Decimal,
        base_unit: impl Into<String>,
    ) -> Self {
        let symbol = symbol.into();
        Self {
            symbol,
            name: name.into(),
            dimension,
            base_factor,
            base_unit: base_unit.into(),
        }
    }

    pub fn new_base(
        symbol: impl Into<String>,
        name: impl Into<String>,
        dimension: Dimension,
    ) -> Result<Self, UnitError> {
        let symbol = symbol.into();
        let base_unit = symbol.clone();
        Ok(Self {
            symbol,
            name: name.into(),
            dimension,
            base_factor: Decimal::ONE,
            base_unit,
        })
    }

    pub fn with_base(mut self, base_unit: impl Into<String>) -> Self {
        self.base_unit = base_unit.into();
        self
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn dimension(&self) -> &Dimension {
        &self.dimension
    }
    pub fn base_factor(&self) -> Decimal {
        self.base_factor
    }
    pub fn base_unit(&self) -> &str {
        &self.base_unit
    }
}

pub trait UnitConversion {
    fn convert_to_base(&self, value: Decimal) -> Decimal;
    fn convert_from_base(&self, value: Decimal) -> Decimal;
}

impl UnitConversion for Unit {
    fn convert_to_base(&self, value: Decimal) -> Decimal {
        value * self.base_factor
    }

    fn convert_from_base(&self, value: Decimal) -> Decimal {
        value / self.base_factor
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum UnitError {
    UnitNotFound(String),
    IncompatibleDimensions { from: Dimension, to: Dimension },
    ConversionNotDefined { from: String, to: String },
    ZeroBaseFactor,
    DuplicateUnit(String),
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
            UnitError::DuplicateUnit(symbol) => {
                write!(f, "Unit already registered: {}", symbol)
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
        registry.register_builtin(Unit::new(
            "kg",
            "kilogram",
            Dimension::Mass,
            Decimal::from(1),
            "kg",
        ));
        registry.register_builtin(Unit::new(
            "g",
            "gram",
            Dimension::Mass,
            Decimal::new(1, 3),
            "kg",
        ));
        registry.register_builtin(Unit::new(
            "lb",
            "pound",
            Dimension::Mass,
            Decimal::new(45359237, 8),
            "kg",
        ));

        // Length units
        registry.register_base(Dimension::Length, "m");
        registry.register_builtin(Unit::new(
            "m",
            "meter",
            Dimension::Length,
            Decimal::from(1),
            "m",
        ));
        registry.register_builtin(Unit::new(
            "cm",
            "centimeter",
            Dimension::Length,
            Decimal::new(1, 2),
            "m",
        ));
        registry.register_builtin(Unit::new(
            "in",
            "inch",
            Dimension::Length,
            Decimal::new(254, 4),
            "m",
        ));

        // Volume units
        registry.register_base(Dimension::Volume, "L");
        registry.register_builtin(Unit::new(
            "L",
            "liter",
            Dimension::Volume,
            Decimal::from(1),
            "L",
        ));
        registry.register_builtin(Unit::new(
            "mL",
            "milliliter",
            Dimension::Volume,
            Decimal::new(1, 3),
            "L",
        ));

        // Currency units (no conversion without exchange rates)
        registry.register_base(Dimension::Currency, "USD");
        registry.register_builtin(Unit::new(
            "USD",
            "US Dollar",
            Dimension::Currency,
            Decimal::from(1),
            "USD",
        ));
        registry.register_builtin(Unit::new(
            "EUR",
            "Euro",
            Dimension::Currency,
            Decimal::from(1),
            "EUR",
        ));
        registry.register_builtin(Unit::new(
            "GBP",
            "British Pound",
            Dimension::Currency,
            Decimal::from(1),
            "GBP",
        ));

        // Time units
        registry.register_base(Dimension::Time, "s");
        registry.register_builtin(Unit::new(
            "s",
            "second",
            Dimension::Time,
            Decimal::from(1),
            "s",
        ));
        registry.register_builtin(Unit::new(
            "min",
            "minute",
            Dimension::Time,
            Decimal::from(60),
            "s",
        ));
        registry.register_builtin(Unit::new(
            "h",
            "hour",
            Dimension::Time,
            Decimal::from(3600),
            "s",
        ));

        // Count (dimensionless)
        registry.register_base(Dimension::Count, "units");
        registry.register_builtin(Unit::new(
            "units",
            "units",
            Dimension::Count,
            Decimal::from(1),
            "units",
        ));
        registry.register_builtin(Unit::new(
            "items",
            "items",
            Dimension::Count,
            Decimal::from(1),
            "items",
        ));

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

    pub fn register(&mut self, unit: Unit) -> Result<(), UnitError> {
        if self.units.contains_key(&unit.symbol) {
            return Err(UnitError::DuplicateUnit(unit.symbol.clone()));
        }
        self.units.insert(unit.symbol.clone(), unit);
        Ok(())
    }

    fn register_builtin(&mut self, unit: Unit) {
        let _ = self.register(unit);
    }

    pub fn register_dimension(&mut self, dimension: Dimension) {
        self.base_units.entry(dimension).or_default();
    }

    pub fn register_base(&mut self, dimension: Dimension, base_unit: impl Into<String>) {
        self.base_units.insert(dimension, base_unit.into());
    }

    pub fn get_unit(&self, symbol: &str) -> Result<&Unit, UnitError> {
        self.units
            .get(symbol)
            .ok_or_else(|| UnitError::UnitNotFound(symbol.to_string()))
    }

    pub fn units(&self) -> &HashMap<String, Unit> {
        &self.units
    }

    pub fn base_units(&self) -> &HashMap<Dimension, String> {
        &self.base_units
    }

    pub fn convert(&self, value: Decimal, from: &Unit, to: &Unit) -> Result<Decimal, UnitError> {
        if from.dimension != to.dimension {
            return Err(UnitError::IncompatibleDimensions {
                from: from.dimension.clone(),
                to: to.dimension.clone(),
            });
        }

        if matches!(from.dimension, Dimension::Currency) && from.symbol != to.symbol {
            return Err(UnitError::ConversionNotDefined {
                from: from.symbol.clone(),
                to: to.symbol.clone(),
            });
        }

        let in_base = from.convert_to_base(value);
        let in_target = to.convert_from_base(in_base);

        Ok(in_target)
    }

    pub fn global() -> &'static mut UnitRegistry {
        use std::sync::Mutex;
        static GLOBAL_REGISTRY: OnceLock<Mutex<UnitRegistry>> = OnceLock::new();

        let mutex = GLOBAL_REGISTRY.get_or_init(|| Mutex::new(UnitRegistry::default()));
        unsafe {
            let ptr = mutex as *const Mutex<UnitRegistry> as *mut Mutex<UnitRegistry>;
            (*ptr).get_mut().unwrap()
        }
    }

    /// Register units defined in a JSON string of the form:
    /// [{ "symbol": "X", "name": "Name", "dimension": "Currency", "base_factor": 1.0, "base_unit": "USD" }]
    pub fn register_from_json(&mut self, json: &str) -> Result<(), UnitError> {
        #[derive(Deserialize)]
        struct UnitConfig {
            symbol: String,
            name: String,
            dimension: String,
            base_factor: f64,
            base_unit: String,
        }

        let parsed: Vec<UnitConfig> = serde_json::from_str(json).map_err(|e| UnitError::ConversionNotDefined { from: "json".to_string(), to: e.to_string() })?;
        for cfg in parsed {
            let dim = match cfg.dimension.as_str() {
                "Mass" => Dimension::Mass,
                "Length" => Dimension::Length,
                "Volume" => Dimension::Volume,
                "Currency" => Dimension::Currency,
                "Time" => Dimension::Time,
                "Temperature" => Dimension::Temperature,
                "Count" => Dimension::Count,
                other => Dimension::Custom(other.to_string()),
            };
            let factor = Decimal::from_f64(cfg.base_factor).ok_or(UnitError::ZeroBaseFactor)?;
            if factor == Decimal::ZERO {
                return Err(UnitError::ZeroBaseFactor);
            }
            let unit = Unit::new(cfg.symbol, cfg.name, dim, factor, cfg.base_unit);
            self.register(unit)?;
        }
        Ok(())
    }
}

static DEFAULT_UNIT_REGISTRY: OnceLock<UnitRegistry> = OnceLock::new();

/// Helper function to get a Unit from a string symbol, using the default registry
/// Returns a Count-based unit if the symbol is not found
pub fn unit_from_string(symbol: impl Into<String>) -> Unit {
    let symbol = symbol.into();
    let registry = DEFAULT_UNIT_REGISTRY.get_or_init(UnitRegistry::default);

    registry.get_unit(&symbol).cloned().unwrap_or_else(|_| {
        // Default to Count dimension for unknown units
        Unit::new(
            symbol.clone(),
            symbol.clone(),
            Dimension::Count,
            Decimal::from(1),
            symbol.clone(),
        )
    })
}

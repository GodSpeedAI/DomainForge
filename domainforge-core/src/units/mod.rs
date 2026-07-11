use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

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

impl Dimension {
    /// Parse a dimension name in a case-insensitive way and map to builtin dimension
    pub fn parse(name: &str) -> Self {
        match name.to_ascii_lowercase().as_str() {
            "mass" => Dimension::Mass,
            "length" => Dimension::Length,
            "volume" => Dimension::Volume,
            "currency" => Dimension::Currency,
            "time" => Dimension::Time,
            "temperature" => Dimension::Temperature,
            "count" => Dimension::Count,
            other => Dimension::Custom(other.to_string()),
        }
    }
}

impl std::str::FromStr for Dimension {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Dimension::parse(s))
    }
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Dimension::Mass => write!(f, "Mass"),
            Dimension::Length => write!(f, "Length"),
            Dimension::Volume => write!(f, "Volume"),
            Dimension::Currency => write!(f, "Currency"),
            Dimension::Time => write!(f, "Time"),
            Dimension::Temperature => write!(f, "Temperature"),
            Dimension::Count => write!(f, "Count"),
            Dimension::Custom(s) => write!(f, "{}", s),
        }
    }
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
#[non_exhaustive]
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

        // Currency units (no conversion without exchange rates). U5: each
        // currency below is its own `base_unit` with `base_factor: 1`, so
        // `Unit::convert_to_base`/`convert_from_base` map every currency to
        // itself at 1.0. That's only correct because `UnitRegistry::convert`
        // special-cases `Dimension::Currency` and refuses cross-currency
        // conversion outright (see below) rather than trusting
        // `base_factor`. Any new code path that reads `base_factor`
        // directly instead of going through `convert` would silently treat
        // EUR/GBP as equal to USD at parity — keep the special-case in sync
        // if this ever needs real exchange rates.
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
        registry.register_builtin(Unit::new(
            "ms",
            "millisecond",
            Dimension::Time,
            Decimal::new(1, 3),
            "s",
        ));
        registry.register_builtin(Unit::new(
            "us",
            "microsecond",
            Dimension::Time,
            Decimal::new(1, 6),
            "s",
        ));
        registry.register_builtin(Unit::new(
            "ns",
            "nanosecond",
            Dimension::Time,
            Decimal::new(1, 9),
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
        // Common metric unit (M1): dimensionless, so it shares the Count
        // dimension rather than inventing a new one just for a label.
        registry.register_builtin(Unit::new(
            "percent",
            "percent",
            Dimension::Count,
            Decimal::from(1),
            "units",
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

    /// U3: previously this only checked for a duplicate symbol. It now also
    /// rejects a zero `base_factor` (division by zero in
    /// `convert_from_base`; the JSON-registration path already caught this,
    /// direct `register()` calls didn't) and a `base_unit` that references
    /// an already-registered unit of a *different* dimension (a copy-paste
    /// mistake that would silently cross-convert two unrelated dimensions).
    /// It deliberately does not require `base_unit` to already be
    /// registered — callers legitimately build a registry incrementally
    /// (e.g. `register_from_json` into a fresh `UnitRegistry::new()`, where
    /// the base symbol is asserted by convention, not by an existing
    /// entry).
    ///
    /// U2: Temperature is rejected outright. `Unit::convert` is a pure
    /// linear scale (`value * base_factor`), which cannot express the
    /// affine Celsius/Fahrenheit conversions real temperature units need;
    /// registering one here would silently produce wrong numbers rather
    /// than erroring, which is worse than not supporting it.
    pub fn register(&mut self, unit: Unit) -> Result<(), UnitError> {
        if self.units.contains_key(&unit.symbol) {
            return Err(UnitError::DuplicateUnit(unit.symbol.clone()));
        }
        if unit.base_factor == Decimal::ZERO {
            return Err(UnitError::ZeroBaseFactor);
        }
        if matches!(unit.dimension, Dimension::Temperature) {
            return Err(UnitError::ConversionNotDefined {
                from: unit.symbol.clone(),
                to: "Temperature (needs affine/offset conversion, unsupported by this linear unit model)"
                    .to_string(),
            });
        }
        if unit.base_unit != unit.symbol {
            if let Some(base) = self.units.get(&unit.base_unit) {
                if base.dimension != unit.dimension {
                    return Err(UnitError::IncompatibleDimensions {
                        from: unit.dimension.clone(),
                        to: base.dimension.clone(),
                    });
                }
            }
        }
        self.units.insert(unit.symbol.clone(), unit);
        Ok(())
    }

    fn register_builtin(&mut self, unit: Unit) {
        self.register(unit)
            .expect("builtin unit registration must be valid");
    }

    /// Records that a dimension is known, without asserting a base unit for
    /// it. Previously this inserted an empty-string placeholder into
    /// `base_units` when the dimension had none yet (via `or_default`),
    /// which poisoned any later `base_units()` consumer with a bogus ""
    /// base symbol (U3). `register_base` is the only source of truth for a
    /// dimension's actual base unit, so this is now a no-op until that's
    /// called.
    pub fn register_dimension(&mut self, _dimension: Dimension) {}

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

        // Cheap invariant check (debug builds only, zero release cost):
        // same-dimension units should always share a base_unit symbol.
        // `register()` enforces this at insertion time (U3), so tripping
        // this in a test/dev build means the registry was built
        // inconsistently — e.g. two units of the same dimension registered
        // against different `base_unit` strings.
        debug_assert_eq!(
            from.base_unit, to.base_unit,
            "same-dimension units '{}' and '{}' disagree on base_unit ('{}' vs '{}')",
            from.symbol, to.symbol, from.base_unit, to.base_unit
        );

        let in_base = from.convert_to_base(value);
        let in_target = to.convert_from_base(in_base);

        Ok(in_target)
    }

    /// U4: this is a process-wide mutable singleton. Any model's custom
    /// unit registrations (via the DSL's `Dimension`/`Unit` declarations,
    /// see `parser/ast.rs`) persist for the lifetime of the process and are
    /// visible to every other model parsed afterward in the same process —
    /// harmless for the one-shot CLI, but a cross-request leak risk for any
    /// long-lived host (an LSP server, a WASM/Python binding kept alive
    /// across calls). If that ever becomes a problem, thread an explicit
    /// `&UnitRegistry` through instead of reaching for this singleton.
    pub fn global() -> &'static RwLock<UnitRegistry> {
        static GLOBAL_REGISTRY: OnceLock<RwLock<UnitRegistry>> = OnceLock::new();
        GLOBAL_REGISTRY.get_or_init(|| RwLock::new(UnitRegistry::default()))
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

        let parsed: Vec<UnitConfig> =
            serde_json::from_str(json).map_err(|e| UnitError::ConversionNotDefined {
                from: "json".to_string(),
                to: e.to_string(),
            })?;
        for cfg in parsed {
            let dim = Dimension::parse(&cfg.dimension);
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

pub fn get_default_registry() -> &'static RwLock<UnitRegistry> {
    UnitRegistry::global()
}

/// Fallible unit lookup (U1). Returns `UnitError::UnitNotFound` for an
/// unregistered symbol instead of silently fabricating a Count-dimension
/// unit. Prefer this in new code; `unit_from_string` exists only for the
/// large pre-existing call-site surface that assumes an infallible lookup.
pub fn try_unit_from_string(symbol: &str) -> Result<Unit, UnitError> {
    let registry = get_default_registry();
    let registry = registry.read().unwrap_or_else(|e| e.into_inner());
    registry.get_unit(symbol).cloned()
}

/// Helper function to get a Unit from a string symbol, using the default
/// registry. Fabricates a unit for an unregistered symbol instead of
/// erroring (kept infallible for the large existing call-site surface —
/// see [`try_unit_from_string`] for the fallible alternative).
///
/// U1 fix: the fabricated unit's dimension is `Custom(symbol)`, not
/// `Count`. Previously every unregistered symbol landed in the shared
/// `Count` dimension, so two different typos (or two unrelated free-form
/// business units, e.g. "orders" vs "widgets") were treated as the same
/// dimension and converted 1:1 with no error — the actual bug this gap
/// caused. Keying the fabricated dimension by the symbol itself means
/// `convert()` now correctly reports `IncompatibleDimensions` between any
/// two distinct unregistered units, while a symbol converted against
/// itself (the only sane use of an ad-hoc unit) still works.
pub fn unit_from_string(symbol: impl Into<String>) -> Unit {
    let symbol = symbol.into();
    try_unit_from_string(&symbol).unwrap_or_else(|_| {
        Unit::new(
            symbol.clone(),
            symbol.clone(),
            Dimension::Custom(symbol.clone()),
            Decimal::from(1),
            symbol.clone(),
        )
    })
}

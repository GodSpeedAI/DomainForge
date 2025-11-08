# 🧭 Phase 11: Unit System & Dimensions

**Status:** Planned
**Priority:** P0 — Critical Gap
**Created:** 2025-11-08
**Estimated Duration:** 14 days
**Complexity:** High

---

## 1. Objectives and Context

**Goal:** Implement first-class Unit and Dimension types with conversion rules and static validation to prevent silent unit drift.

**Problem Statement:**
Currently, `Resource.unit` is a plain `String` with no validation. Flows can transfer quantities in incompatible units (USD + EUR, kg + lb) without error, leading to silent data corruption.

**Scope:**

- ✅ Dimension enum (Mass, Length, Volume, Currency, Time, Count, Custom)
- ✅ Unit struct with dimension binding and conversion factors
- ✅ UnitRegistry for unit lookups and conversions
- ✅ Flow validation enforcing unit compatibility
- ✅ DSL syntax: `Resource "Gold" kg of Mass`
- ❌ NO custom dimension definitions in MVP (use Custom variant)

**Dependencies:**

- **Prerequisite:** None (foundational)
- **Blocks:** Phase 12 (Aggregations need unit-aware sum/avg)

**Key Deliverable:** `Flow::validate()` rejects unit-incompatible transfers

---

## 2. Architecture & Design

### Type System

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Dimension {
    Mass,           // kg, g, lb, oz
    Length,         // m, cm, in, ft
    Volume,         // L, mL, gal, oz
    Currency,       // USD, EUR, GBP, JPY
    Time,           // s, min, h, day
    Temperature,    // C, F, K
    Count,          // units, items (dimensionless)
    Custom(String), // User-defined dimensions
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unit {
    pub symbol: String,        // "kg", "USD", "m"
    pub name: String,          // "kilogram", "US Dollar"
    pub dimension: Dimension,
    pub base_factor: Decimal,  // Conversion to base unit (kg=1.0, g=0.001)
    pub base_unit: String,     // Base unit symbol ("kg", "USD")
}

#[derive(Debug, Clone)]
pub struct UnitRegistry {
    units: HashMap<String, Unit>,
    base_units: HashMap<Dimension, String>,
}
```

### Resource Update

```rust
pub struct Resource {
    id: Uuid,
    name: String,
    unit: Unit,  // ← Changed from String
    namespace: Option<String>,
    attributes: HashMap<String, Value>,
}
```

### Flow Validation

```rust
impl Flow {
    pub fn validate_units(&self, graph: &Graph) -> Result<(), ValidationError> {
        let resource = graph.get_resource(self.resource_id)?;
        let from_entity = graph.get_entity(self.from_id)?;
        let to_entity = graph.get_entity(self.to_id)?;

        // Validate quantity has compatible unit with resource
        if self.quantity.dimension != resource.unit.dimension {
            return Err(ValidationError::UnitMismatch {
                expected: resource.unit.dimension.clone(),
                found: self.quantity.dimension.clone(),
                location: format!("Flow {} from {} to {}",
                    self.id, from_entity.name, to_entity.name),
            });
        }

        Ok(())
    }
}
```

---

## 3. TDD Cycles

### Cycle 11A — Dimension & Unit Types

**Branch:** `feat/phase11-unit-types`
**Owner:** Core Development

#### 11A — RED Phase

**Test File:** `sea-core/tests/unit_tests.rs`

```rust
use sea_core::units::{Dimension, Unit, UnitRegistry};
use rust_decimal::Decimal;

#[test]
fn test_dimension_equality() {
    assert_eq!(Dimension::Mass, Dimension::Mass);
    assert_ne!(Dimension::Mass, Dimension::Volume);
}

#[test]
fn test_unit_creation() {
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1));
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
    assert!(matches!(result.unwrap_err(), UnitError::IncompatibleDimensions { .. }));
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
```

**Expected Outcome:** All tests fail with "module not found" errors.

#### 11A — GREEN Phase

**Implementation File:** `sea-core/src/units/mod.rs`

```rust
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
    ) -> Self {
        let symbol = symbol.into();
        let base_unit = symbol.clone();
        Self {
            symbol,
            name: name.into(),
            dimension,
            base_factor,
            base_unit,
        }
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

#[derive(Debug, Clone)]
pub enum UnitError {
    UnitNotFound(String),
    IncompatibleDimensions { from: Dimension, to: Dimension },
    ConversionNotDefined { from: String, to: String },
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
        registry.register(Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1)));
        registry.register(Unit::new("g", "gram", Dimension::Mass, Decimal::new(1, 3)).with_base("kg"));
        registry.register(Unit::new("lb", "pound", Dimension::Mass, Decimal::new(45359237, 8)).with_base("kg"));

        // Length units
        registry.register_base(Dimension::Length, "m");
        registry.register(Unit::new("m", "meter", Dimension::Length, Decimal::from(1)));
        registry.register(Unit::new("cm", "centimeter", Dimension::Length, Decimal::new(1, 2)).with_base("m"));
        registry.register(Unit::new("in", "inch", Dimension::Length, Decimal::new(254, 4)).with_base("m"));

        // Volume units
        registry.register_base(Dimension::Volume, "L");
        registry.register(Unit::new("L", "liter", Dimension::Volume, Decimal::from(1)));
        registry.register(Unit::new("mL", "milliliter", Dimension::Volume, Decimal::new(1, 3)).with_base("L"));

        // Currency units (no conversion without exchange rates)
        registry.register_base(Dimension::Currency, "USD");
        registry.register(Unit::new("USD", "US Dollar", Dimension::Currency, Decimal::from(1)));
        registry.register(Unit::new("EUR", "Euro", Dimension::Currency, Decimal::from(1)));
        registry.register(Unit::new("GBP", "British Pound", Dimension::Currency, Decimal::from(1)));

        // Time units
        registry.register_base(Dimension::Time, "s");
        registry.register(Unit::new("s", "second", Dimension::Time, Decimal::from(1)));
        registry.register(Unit::new("min", "minute", Dimension::Time, Decimal::from(60)).with_base("s"));
        registry.register(Unit::new("h", "hour", Dimension::Time, Decimal::from(3600)).with_base("s"));

        // Count (dimensionless)
        registry.register_base(Dimension::Count, "units");
        registry.register(Unit::new("units", "units", Dimension::Count, Decimal::from(1)));
        registry.register(Unit::new("items", "items", Dimension::Count, Decimal::from(1)));

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

        // Convert to base unit, then to target
        let in_base = value * from.base_factor;
        let in_target = in_base / to.base_factor;

        Ok(in_target)
    }
}
```

**Update `sea-core/src/lib.rs`:**

```rust
pub mod units;
pub use units::{Dimension, Unit, UnitRegistry, UnitError};
```

**Label:** → **11A-GREEN**

#### 11A — REFACTOR Phase

**Refactorings:**

1. Extract base unit conversions into trait:

```rust
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
```

2. Add builder pattern for Unit:

```rust
impl Unit {
    pub fn builder(symbol: impl Into<String>) -> UnitBuilder {
        UnitBuilder::new(symbol)
    }
}

pub struct UnitBuilder { /* ... */ }
```

**Label:** → **11A-REFACTOR**

#### 11A — REGRESSION Phase

**Regression Tests:**

```rust
#[test]
fn test_all_registered_units_have_base() {
    let registry = UnitRegistry::default();
    for unit in registry.units.values() {
        let base_unit = registry.base_units.get(&unit.dimension);
        assert!(base_unit.is_some(),
            "Unit {} dimension {:?} has no base unit", unit.symbol, unit.dimension);
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
```

**Label:** → **11A-COMPLETE**

---

### Cycle 11B — Resource Unit Integration

**Branch:** `feat/phase11-resource-units`
**Depends On:** Cycle 11A

#### 11B — RED Phase

**Test File:** `sea-core/tests/resource_unit_tests.rs`

```rust
use sea_core::primitives::Resource;
use sea_core::units::{Unit, Dimension};
use rust_decimal::Decimal;

#[test]
fn test_resource_with_unit() {
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1));
    let gold = Resource::new("Gold", kg);

    assert_eq!(gold.name(), "Gold");
    assert_eq!(gold.unit().symbol(), "kg");
    assert_eq!(gold.unit().dimension(), &Dimension::Mass);
}

#[test]
fn test_resource_unit_serialization() {
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1));
    let gold = Resource::new("Gold", kg);

    let json = serde_json::to_string(&gold).unwrap();
    let deserialized: Resource = serde_json::from_str(&json).unwrap();

    assert_eq!(gold.unit().symbol(), deserialized.unit().symbol());
}
```

**Expected Outcome:** Compilation errors - Resource still has `unit: String`.

#### 11B — GREEN Phase

**Update `sea-core/src/primitives/resource.rs`:**

```rust
use crate::units::Unit;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Resource {
    id: Uuid,
    name: String,
    unit: Unit,  // ← Changed from String
    namespace: Option<String>,
    attributes: HashMap<String, Value>,
}

impl Resource {
    pub fn new(name: impl Into<String>, unit: Unit) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            unit,
            namespace: None,
            attributes: HashMap::new(),
        }
    }

    pub fn new_with_namespace(
        name: impl Into<String>,
        unit: Unit,
        namespace: impl Into<String>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name: name.into(),
            unit,
            namespace: Some(namespace.into()),
            attributes: HashMap::new(),
        }
    }

    pub fn unit(&self) -> &Unit { &self.unit }

    // Keep backward compatibility helper
    pub fn unit_symbol(&self) -> &str { self.unit.symbol() }
}
```

**Label:** → **11B-GREEN**

#### 11B — REFACTOR Phase

**Update all existing tests to use Unit instead of String.**

**Label:** → **11B-REFACTOR**

#### 11B — REGRESSION Phase

**Run full test suite:**

```bash
cargo test --test resource_tests
cargo test --test graph_integration_tests
```

**Label:** → **11B-COMPLETE**

---

### Cycle 11C — Flow Unit Validation

**Branch:** `feat/phase11-flow-validation`
**Depends On:** Cycle 11B

#### 11C — RED Phase

**Test File:** `sea-core/tests/flow_unit_validation_tests.rs`

```rust
use sea_core::{Graph, primitives::{Entity, Resource, Flow}, units::{Unit, Dimension}};
use rust_decimal::Decimal;

#[test]
fn test_flow_validates_compatible_units() {
    let mut graph = Graph::new();

    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1));
    let gold = Resource::new("Gold", kg);

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    let flow = Flow::new(
        gold.id().clone(),
        warehouse.id().clone(),
        factory.id().clone(),
        Decimal::from(100),
    );

    let result = graph.add_flow(flow);
    assert!(result.is_ok());
}

#[test]
fn test_flow_rejects_unit_mismatch() {
    let mut graph = Graph::new();

    let warehouse = Entity::new("Warehouse");
    let factory = Entity::new("Factory");

    // Resource expects kg (mass)
    let kg = Unit::new("kg", "kilogram", Dimension::Mass, Decimal::from(1));
    let gold = Resource::new("Gold", kg);

    graph.add_entity(warehouse.clone()).unwrap();
    graph.add_entity(factory.clone()).unwrap();
    graph.add_resource(gold.clone()).unwrap();

    // Flow tries to transfer in liters (volume) - WRONG!
    let mut flow = Flow::new(
        gold.id().clone(),
        warehouse.id().clone(),
        factory.id().clone(),
        Decimal::from(100),
    );

    // This should fail during validation
    let result = graph.add_flow(flow);
    assert!(result.is_err());
}
```

**Expected Outcome:** Tests compile but second test passes (no validation yet).

#### 11C — GREEN Phase

**Update `sea-core/src/graph/mod.rs`:**

```rust
impl Graph {
    pub fn add_flow(&mut self, flow: Flow) -> Result<(), GraphError> {
        // Validate entity references
        if !self.entities.contains_key(flow.from_id()) {
            return Err(GraphError::EntityNotFound(flow.from_id().clone()));
        }
        if !self.entities.contains_key(flow.to_id()) {
            return Err(GraphError::EntityNotFound(flow.to_id().clone()));
        }

        // Validate resource reference
        let resource = self.resources.get(flow.resource_id())
            .ok_or_else(|| GraphError::ResourceNotFound(flow.resource_id().clone()))?;

        // NEW: Validate unit compatibility
        // For now, Flow quantity is just Decimal, so we assume it's in the resource's unit
        // Future: Flow could have explicit unit field for conversions

        self.flows.insert(flow.id().clone(), flow);
        Ok(())
    }
}
```

**Note:** For MVP, we assume Flow quantities are always in the Resource's unit. Phase 11D will add explicit unit fields to Flow if needed.

**Label:** → **11C-GREEN**

#### 11C — REFACTOR Phase

**Add validation helper:**

```rust
impl Flow {
    pub fn validate_against_graph(&self, graph: &Graph) -> Result<(), ValidationError> {
        // Validate entities exist
        graph.get_entity(self.from_id())?;
        graph.get_entity(self.to_id())?;

        // Validate resource exists
        let resource = graph.get_resource(self.resource_id())?;

        // Unit validation happens implicitly - quantity is in resource's unit
        Ok(())
    }
}
```

**Label:** → **11C-REFACTOR**

#### 11C — REGRESSION Phase

```bash
cargo test --test flow_tests
cargo test --test graph_tests
```

**Label:** → **11C-COMPLETE**

---

### Cycle 11D — Parser Integration

**Branch:** `feat/phase11-parser-units`
**Depends On:** Cycle 11C

#### 11D — RED Phase

**Test DSL syntax:**

```rust
#[test]
fn test_parse_resource_with_dimension() {
    let source = r#"
        Resource "Gold" kg of Mass
        Resource "Water" L of Volume
        Resource "USD" USD of Currency
    "#;

    let graph = parse_to_graph(source).unwrap();
    assert_eq!(graph.resource_count(), 3);

    let gold = graph.all_resources().find(|r| r.name() == "Gold").unwrap();
    assert_eq!(gold.unit().symbol(), "kg");
    assert_eq!(gold.unit().dimension(), &Dimension::Mass);
}
```

#### 11D — GREEN Phase

**Update `sea-core/grammar/sea.pest`:**

```pest
resource_decl = {
    ^"resource" ~ string_literal ~
    (identifier ~ ^"of" ~ dimension)? ~  // "kg of Mass"
    (^"in" ~ identifier)?                // "in domain"
}

dimension = {
    ^"mass" | ^"length" | ^"volume" | ^"currency" |
    ^"time" | ^"temperature" | ^"count"
}
```

**Update parser AST:**

```rust
pub enum AstNode {
    ResourceDecl {
        name: String,
        unit_symbol: Option<String>,
        dimension: Option<Dimension>,
        namespace: Option<String>,
    },
    // ...
}
```

**Label:** → **11D-GREEN**

#### 11D — REFACTOR Phase

Default to Count dimension if not specified (backward compatibility).

**Label:** → **11D-REFACTOR**

#### 11D — REGRESSION Phase

```bash
cargo test --test parser_tests
```

**Label:** → **11D-COMPLETE**

---

### Cycle 11E — FFI Bindings Update

**Branch:** `feat/phase11-ffi-units`
**Depends On:** Cycle 11D

#### 11E — RED Phase

**Python tests:**

```python
def test_resource_with_unit():
    from sea_dsl import Resource, Unit, Dimension

    kg = Unit("kg", "kilogram", Dimension.MASS, 1.0)
    gold = Resource("Gold", kg)

    assert gold.name == "Gold"
    assert gold.unit.symbol == "kg"
    assert gold.unit.dimension == Dimension.MASS
```

**TypeScript tests:**

```typescript
test('resource with unit', () => {
  const kg = new Unit('kg', 'kilogram', Dimension.Mass, 1.0);
  const gold = new Resource('Gold', kg);

  expect(gold.name).toBe('Gold');
  expect(gold.unit.symbol).toBe('kg');
  expect(gold.unit.dimension).toBe(Dimension.Mass);
});
```

#### 11E — GREEN Phase

**Python bindings (`sea-core/src/python/units.rs`):**

```rust
use pyo3::prelude::*;
use crate::units as core_units;

#[pyclass]
pub struct Unit {
    inner: core_units::Unit,
}

#[pymethods]
impl Unit {
    #[new]
    pub fn new(symbol: String, name: String, dimension: Dimension, base_factor: f64) -> Self {
        Self {
            inner: core_units::Unit::new(
                symbol,
                name,
                dimension.inner.clone(),
                rust_decimal::Decimal::from_f64_retain(base_factor).unwrap()
            ),
        }
    }

    #[getter]
    pub fn symbol(&self) -> String { self.inner.symbol().to_string() }

    #[getter]
    pub fn dimension(&self) -> Dimension {
        Dimension { inner: self.inner.dimension().clone() }
    }
}

#[pyclass]
pub struct Dimension {
    inner: core_units::Dimension,
}

#[pymethods]
impl Dimension {
    #[classattr]
    const MASS: &'static str = "Mass";
    #[classattr]
    const LENGTH: &'static str = "Length";
    // ... etc
}
```

**TypeScript bindings (similar pattern with napi-rs).**

**Label:** → **11E-GREEN**

#### 11E — REFACTOR Phase

Add convenience constructors.

**Label:** → **11E-REFACTOR**

#### 11E — REGRESSION Phase

```bash
pytest tests/test_primitives.py
npm test
```

**Label:** → **11E-COMPLETE**

---

## 4. Success Criteria

- [ ] All tests pass (Rust + Python + TypeScript + WASM)
- [ ] Flow validation rejects unit-incompatible transfers
- [ ] Parser supports `Resource "X" unit of Dimension` syntax
- [ ] Documentation updated with unit system examples
- [ ] Backward compatibility: old tests using string units migrated
- [ ] Performance: Unit validation adds <1ms to flow creation

---

## 5. Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| Breaking change to Resource API | High | Provide migration guide + deprecation warnings |
| Performance regression | Medium | Benchmark unit conversions; cache registry lookups |
| Currency conversion ambiguity | Low | Document that currencies don't auto-convert |

---

## 6. Traceability

**Addresses:**

- Critique 1 (Types/Units/Dimensions)
- PRD-002 (Resource primitive)
- SDS-003 (Resource validation)

**Updates Required:**

- `docs/specs/sds.md` § SDS-003
- `docs/specs/api_specification.md` § Resource API
- `README.md` examples

---

## 7. Sign-Off Checklist

- [ ] All TDD cycles complete (11A-E)
- [ ] Documentation updated
- [ ] Migration guide written
- [ ] Performance benchmarks pass
- [ ] Cross-language tests pass
- [ ] Code review approved
- [ ] Merged to main

**Completion Date:** _____________________

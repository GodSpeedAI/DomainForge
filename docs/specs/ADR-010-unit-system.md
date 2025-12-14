# ADR-010: Unit System

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

SEA Resources have associated units of measurement (e.g., "units", "kg", "USD"). The system needs to:

1. Parse unit specifications from DSL source
2. Validate unit compatibility in operations
3. Support unit conversions where applicable
4. Enable dimensional analysis for type safety

## Decision

### Unit Representation

```rust
pub struct Unit {
    pub symbol: String,      // "kg", "USD", "units"
    pub name: String,        // "Kilogram", "US Dollar", "Units"
    pub dimension: Dimension, // Mass, Currency, Count
    pub scale: f64,          // Conversion factor to base unit
    pub base_symbol: String, // Base unit symbol
}

pub enum Dimension {
    Mass,
    Length,
    Time,
    Currency,
    Count,
    Temperature,
    Custom(String),
}
```

### Unit Parsing

```rust
pub fn unit_from_string(s: &str) -> Unit {
    match s.to_lowercase().as_str() {
        "units" | "unit" => Unit::count(),
        "kg" | "kilograms" => Unit::mass_kg(),
        "usd" | "dollars" => Unit::currency_usd(),
        // ... more built-in units
        custom => Unit::custom(custom),
    }
}
```

### Conversion Operator

The DSL supports explicit unit conversion via the `as` operator:

```sea
Resource "Response Time" 1000 'ms' as 's'  // Converts to 1 second
```

### Dimensional Analysis

Prevent invalid operations at parse time:

```sea
// Error: Cannot add Mass and Currency
Policy invalid as: Weight.value + Price.value > 100
```

### Unit Registry

Extensible registry for custom units:

```rust
pub struct UnitRegistry {
    units: HashMap<String, Unit>,
    conversions: HashMap<(String, String), f64>,
}

impl UnitRegistry {
    pub fn convert(&self, value: f64, from: &str, to: &str) -> Option<f64> {
        self.conversions.get(&(from.to_string(), to.to_string()))
            .map(|factor| value * factor)
    }
}
```

## Consequences

### Positive

- **Type safety**: Dimensional analysis catches unit errors
- **Expressiveness**: Natural unit syntax in DSL
- **Flexibility**: Custom units for domain-specific needs
- **Interoperability**: Standard units map to external systems

### Negative

- **Complexity**: Full dimensional analysis is non-trivial
- **Limitations**: Not all unit conversions are linear

## Related

- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)
- [PRD-003: DSL Core Capabilities](./PRD-003-dsl-core-capabilities.md)

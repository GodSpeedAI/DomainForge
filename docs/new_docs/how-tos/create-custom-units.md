# Create Custom Units

Goal: Define custom dimensions/units and validate quantities using them.

## Prerequisites

- SEA CLI installed (`cargo install --path sea-core --features cli`) or Python/TypeScript bindings.
- Basic understanding of Resource/Flow quantities.

## Declare Dimensions and Units (DSL)

Add declarations near the top of your `.sea` file:

```sea
Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"
Unit "EUR" of "Currency" factor 1.07 base "USD"
```

- `factor` is the multiplier relative to the base unit.
- Always provide a `base` to avoid ambiguous conversions.

## Use Units in Resources and Flows

```sea
Resource "Money" unit "USD"
Entity "Alice"
Entity "Bob"
Flow "Money" from "Alice" to "Bob" quantity 100 "EUR"
```

- Resources reference the canonical unit.
- Flows can use any declared unit; conversions happen via the base factor.

## Validate and Test

- CLI: `sea validate model.sea` (fails if the unit/dimension is unknown or factor is invalid).
- Python: `from sea_dsl import Unit; Unit.parse("USD")` to confirm registration; parse a graph and inspect `resource.unit`.
- TypeScript: `Unit.parse("EUR")` and `Graph.parse(...)` to ensure flows keep the correct unit strings.

## Edge Cases

- Avoid circular bases (e.g., EUR base USD, USD base EUR) — validation fails.
- Use decimal notation for `factor` to keep precision (e.g., `1.07`, not `1,07`).
- Namespaces: if you scope units by namespace, keep resource/unit declarations in the same namespace to prevent resolution errors.

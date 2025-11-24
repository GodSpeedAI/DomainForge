# Type System Specification

## Overview

The SEA DSL type system enforces semantic correctness in expressions, particularly distinguishing between dimensionless numbers and physical quantities with units.

## Types

The `ExprType` enum defines the supported types:

- `Quantity { dimension: Dimension }`: A value with a physical dimension (e.g., Mass, Length).
- `Numeric`: A dimensionless number.
- `String`: A text string.
- `Boolean`: True or False.
- `Collection(Box<ExprType>)`: A collection of values of a specific type.

## Type Inference

Type inference is performed by `sea-core/src/policy/type_inference.rs`. It analyzes expressions to determine their result type and checks for compatibility.

### Rules

1. **Comparisons**:
   - Quantities must be compared with Quantities of the same dimension.
   - Numerics must be compared with Numerics.
   - Mixing Quantity and Numeric in comparison is a type error.
2. **Arithmetic**:
   - Adding/Subtracting Quantities requires same dimension.
   - Multiplying/Dividing Quantities results in a new dimension (not fully implemented yet, currently results in Quantity or Numeric depending on operation).

## Error Handling

- `UnitMismatch`: Comparing quantities of different dimensions (e.g., Mass vs Length).
- `MixedQuantityNumeric`: Comparing Quantity with Numeric.
- `TypeMismatch`: General type incompatibility (e.g., Boolean vs String).

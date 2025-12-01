# Implementation Plan - Units & Dimensions (Roadmap Step 2)

This plan addresses **Step 2: Units & Dimensions (Refinement)** from the `dsl-completeness-roadmap.md`. The goal is to implement full arithmetic support with strict unit validation and automatic conversion.

## User Review Required

> [!IMPORTANT] > **Breaking Change Potential**: Strict unit validation might break existing policies that rely on implicit dimensionless assumptions. However, since the current evaluator errors on arithmetic, this is likely purely additive for arithmetic, but comparison logic will become stricter.

## Proposed Changes

### Core Semantics (`sea-core/src/policy/`)

#### [MODIFY] [core.rs](file:///home/sprime01/projects/domainforge/sea-core/src/policy/core.rs)

- **Update `get_runtime_value`**:
  - Currently, it returns an error for `Expression::Binary` (via fallthrough).
  - Add matching for `BinaryOp::Plus`, `Minus`, `Multiply`, `Divide`.
  - Call a new helper `evaluate_arithmetic_expression`.
- **Implement `evaluate_arithmetic_expression`**:
  - Recursively evaluate left and right operands.
  - Extract values and units (handling `__quantity_value` and `__quantity_unit` JSON structure).
  - Use `UnitRegistry::global()` to check compatibility and perform conversions.
  - **Rules**:
    - `Plus`/`Minus`: Operands must have compatible dimensions. Convert to left operand's unit (or base unit) before adding.
    - `Multiply`/`Divide`: (Future) Would create compound units. For now, we can restrict to:
      - `Quantity * Scalar` -> `Quantity`
      - `Quantity / Scalar` -> `Quantity`
      - `Quantity / Quantity` -> `Scalar` (if same dimension)
      - _Note_: The roadmap example `sum(...) <= 10_000 "USD"` implies we primarily need `Plus` for aggregations and comparisons.
- **Update `compare_values` / `compare_numeric`**:
  - Ensure they handle the JSON quantity structure correctly when comparing.

#### [MODIFY] [type_inference.rs](file:///home/sprime01/projects/domainforge/sea-core/src/policy/type_inference.rs)

- **Add `check_arithmetic(op, left, right)`**:
  - Validate dimension compatibility for `Plus`/`Minus`.
  - Validate scalar/quantity mixing for `Multiply`/`Divide`.
  - Return `TypeError` with helpful hints (e.g., "Cannot add 'Money' and 'Mass'").

### Unit Registry (`sea-core/src/units/`)

#### [MODIFY] [mod.rs](file:///home/sprime01/projects/domainforge/sea-core/src/units/mod.rs)

- Ensure `UnitRegistry` is accessible and thread-safe (already has `global()`).
- Verify `convert` handles all cases needed.

### Language Bindings

#### [NEW] `sea-core/src/python/units.rs`

- Expose `Dimension` enum and `Unit` struct to Python.
- Implement `__str__` and `__repr__`.
- Register in `sea-core/src/python/lib.rs`.

#### [NEW] `sea-core/src/typescript/units.rs`

- Expose `Dimension` enum and `Unit` struct to TypeScript (NAPI).
- Register in `sea-core/src/typescript/lib.rs`.

#### [NEW] `sea-core/src/wasm/units.rs`

- Expose `Dimension` enum and `Unit` struct to WASM.
- Register in `sea-core/src/wasm/lib.rs`.

### CLI Updates

#### [MODIFY] `sea-core/src/cli/validate.rs`

- Ensure validation command reports new `UnitError` types correctly.
- Add flag `--strict-units` (optional, if we want to toggle strictness).

## Verification Plan

### Automated Tests

We will add a new test file `tests/units_dimensions_v2_tests.rs` covering:

1.  **Arithmetic Evaluation**:
    - `100 "USD" + 50 "USD" == 150 "USD"`
    - `1 "kg" + 500 "g" == 1.5 "kg"` (Automatic conversion)
    - `10 "m" - 50 "cm" == 9.5 "m"`
2.  **Validation Errors**:
    - `100 "USD" + 10 "kg"` -> Should fail validation (Incompatible dimensions).
    - `100 "USD" + 5` -> Should fail (Mixed quantity/numeric).
3.  **Comparisons**:
    - `1 "km" > 500 "m"` -> Should be true.
    - `100 "USD" < 100 "EUR"` -> Should error (Conversion not defined) or require explicit exchange rate logic (out of scope, so error is correct).

### Binding Tests

- **Python**: Add `tests/test_units.py` to verify `Dimension` and `Unit` classes.
- **TypeScript**: Add `typescript-tests/units.test.ts`.
- **WASM**: Add `tests/wasm_units.rs`.

### Manual Verification

Run the new tests:

```bash
cargo test --test units_dimensions_v2_tests
# Run binding tests (commands to be confirmed based on project setup)
```

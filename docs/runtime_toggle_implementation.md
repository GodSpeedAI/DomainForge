# Runtime Toggle for Tri-State Semantics - Implementation Summary

## Overview

Successfully implemented a runtime toggle for tri-state (three-valued) logic in the SEA policy evaluation system. This allows users to switch between three-valued logic (True, False, NULL) and strict boolean logic (True, False) at runtime without rebuilding the crate.

## Changes Made

### 1. Removed Compile-Time Feature Flags

**Files Modified:**

- `sea-core/src/policy/mod.rs`
- `sea-core/src/policy/three_valued.rs`
- `sea-core/src/policy/core.rs`

**Changes:**

- Removed all `#[cfg(feature = "three_valued_logic")]` guards
- Made `ThreeValuedBool` and related functionality always available
- Removed conditional imports and module declarations

### 2. Added Runtime Configuration

**File:** `sea-core/src/graph/mod.rs`

**New Types:**

```rust
pub struct GraphConfig {
    pub use_three_valued_logic: bool,
}
```

**New Methods:**

```rust
pub fn set_evaluation_mode(&mut self, use_three_valued_logic: bool)
pub fn use_three_valued_logic(&self) -> bool
```

**Default Behavior:**

- Three-valued logic is **enabled by default** for backward compatibility
- Configuration is serializable with `#[serde(default)]`

### 3. Updated Policy Evaluation Logic

**File:** `sea-core/src/policy/core.rs`

**Changes:**

- Modified `Policy::evaluate()` to check `graph.config.use_three_valued_logic` at runtime
- Replaced compile-time `#[cfg]` blocks with runtime `if` statements
- Both evaluation paths (`evaluate_expression` and `evaluate_expression_three_valued`) are now always compiled

### 4. Documentation Updates

**File:** `docs/evaluate_policy.md`

**Added:**

- Runtime Configuration section with examples for Python, TypeScript, and WebAssembly
- Explanation of behavior differences between modes
- Code examples showing how to toggle and check the evaluation mode

### 5. Test Coverage

**New File:** `sea-core/tests/runtime_toggle_tests.rs`

**Tests:**

- `test_runtime_toggle_three_valued_logic` - Verifies both modes work correctly
- `test_runtime_toggle_default_is_three_valued` - Confirms default is three-valued
- `test_runtime_toggle_can_be_changed` - Tests mode switching

## API Usage

### Rust

```rust
use sea_core::graph::Graph;

let mut graph = Graph::new();

// Enable three-valued logic (default)
graph.set_evaluation_mode(true);

// Disable three-valued logic
graph.set_evaluation_mode(false);

// Check current mode
let is_tristate = graph.use_three_valued_logic();
```


## Behavior Differences

### Three-Valued Logic Enabled (Default)

- Comparisons involving `NULL` values yield `NULL` results
- Quantifiers (`ForAll`, `Exists`, `ExistsUnique`) propagate `NULL` appropriately
- Missing data results in `NULL` evaluation with a violation severity derived from the policy modality

### Three-Valued Logic Disabled

- Expressions are expanded before evaluation
- Missing data causes evaluation errors
- Strict boolean logic (True/False only)

## Backward Compatibility

- **Default behavior unchanged**: Three-valued logic is enabled by default
- **Serialization compatible**: New `config` field uses `#[serde(default)]`
- **Existing tests pass**: All 66 existing tests continue to pass
- **Feature flag removed**: The `three_valued_logic` feature flag is no longer needed

## Benefits

1. **No Rebuild Required**: Users can switch modes at runtime
2. **Cross-Language Support**: Works seamlessly with Python, TypeScript, and WASM bindings
3. **Flexible**: Projects can choose the evaluation mode that best fits their needs
4. **Backward Compatible**: Existing code continues to work without changes

## Testing

All tests pass successfully:

- ✅ 66 existing sea-core tests
- ✅ 4 three-valued quantifier tests
- ✅ 3 new runtime toggle tests

## Next Steps

✅ **COMPLETED**: Python and TypeScript bindings implemented and tested.

### Python Bindings ✅

**File:** `sea-core/src/python/graph.rs`

**Added Methods:**

```python
def set_evaluation_mode(self, use_three_valued_logic: bool) -> None:
    """Set the evaluation mode for policy evaluation."""

def use_three_valued_logic(self) -> bool:
    """Get the current evaluation mode."""
```

**Tests:** `tests/test_runtime_toggle.py`

- 6 comprehensive tests covering default behavior, toggling, and mode persistence

### TypeScript Bindings ✅

**File:** `sea-core/src/typescript/graph.rs`

**Added Methods:**

```typescript
setEvaluationMode(useThreeValuedLogic: boolean): void
useThreeValuedLogic(): boolean
```

**Tests:** `typescript-tests/runtime_toggle.test.ts`

- 6 comprehensive tests including instance independence verification

### WASM Bindings ✅

**File:** `sea-core/src/wasm/graph.rs`

**Added Methods:**

```javascript
setEvaluationMode(useThreeValuedLogic: boolean): void
useThreeValuedLogic(): boolean
```

**Naming Convention:** Uses camelCase for JavaScript compatibility (`setEvaluationMode`, `useThreeValuedLogic`)

## Build Verification

All bindings compile successfully:

- ✅ Python: `cargo build --features python`
- ✅ TypeScript: `cargo build --features typescript`
- ✅ WASM: `cargo build --features wasm --target wasm32-unknown-unknown`

## Best Practices Followed

1. **Consistent API Design**: All three bindings expose the same functionality with platform-appropriate naming conventions
2. **Comprehensive Documentation**: Each method includes doc comments explaining behavior
3. **No Code Duplication**: All bindings delegate to the core `Graph` implementation
4. **Error Handling**: Proper error propagation in all bindings
5. **Test Coverage**: Comprehensive tests for each binding platform
6. **Backward Compatibility**: Default behavior preserved (three-valued logic enabled)

## Complexity Assessment

- **Implementation Complexity**: Low (straightforward delegation to core)
- **API Surface Impact**: Minimal (2 new methods on `Graph`)
- **Cross-Language Impact**: Successfully implemented across all platforms
- **Technical Debt**: None - clean implementation following existing patterns

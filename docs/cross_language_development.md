# Cross-Language Development Guide

## Overview

The SEA DSL core library (`sea-core`) supports multiple language bindings through conditional compilation. This guide explains how to work with Python and TypeScript/JavaScript bindings simultaneously.

## Architecture

```
sea-core/
├── src/
│   ├── lib.rs           # Feature-gated module exports
│   ├── graph/           # Core Rust implementation
│   ├── policy/          # Core policy evaluation logic
│   ├── python/          # Python (PyO3) bindings
│   │   ├── graph.rs
│   │   └── policy.rs
│   └── typescript/      # TypeScript (NAPI) bindings
│       ├── graph.rs
│       └── policy.rs
└── Cargo.toml           # Feature definitions
```

## Feature Flags

The library uses Cargo features to conditionally compile language-specific code:

```toml
[features]
default = []
python = ["pyo3", "pythonize"]
typescript = ["napi", "napi-derive"]
three_valued_logic = []
```

### Feature Isolation

**Critical Rule**: Never enable both `python` and `typescript` features in the same build.

- Python builds use PyO3 and link against Python runtime
- TypeScript builds use NAPI-RS and link against Node.js runtime
- These are mutually exclusive and will cause symbol conflicts if mixed

## Building for Different Targets

### Python Bindings

```bash
# Development build (debug)
maturin develop --features python,three_valued_logic

# Production build (release)
maturin build --release --features python,three_valued_logic

# Run Python tests
pytest tests/
# or
just python-test
```

**Build artifacts:**

- `target/wheels/*.whl` - Python wheel package
- `.venv/lib/python*/site-packages/sea_dsl/` - Installed module

### TypeScript/Node.js Bindings

```bash
# Clean previous builds (important!)
cargo clean

# Development build (debug)
npm run build:debug

# Production build (release)
npm run build

# Run TypeScript tests
npm test
```

**Build artifacts:**

- `sea-core.linux-x64-gnu.node` - Native Node.js module
- `index.js` - JavaScript loader
- `index.d.ts` - TypeScript type definitions

### WASM Bindings (Future)

```bash
wasm-pack build sea-core --features wasm,three_valued_logic
```

## Workflow Best Practices

### Switching Between Languages

When switching from Python to TypeScript development (or vice versa):

1. **Clean all build artifacts:**

   ```bash
   # Workspace layout uses a single target/ directory at the repo root
   cargo clean
   ```

2. **Build for the target language:**

   ```bash
   # For Python
   maturin develop --features python,three_valued_logic

   # For TypeScript
   npm run build
   ```

3. **Verify the build:**

   ```bash
   # For Python
   python -c "from sea_dsl import Graph; print(Graph.__doc__)"

   # For TypeScript
   node -e "const {Graph} = require('./index.js'); console.log(typeof Graph)"
   ```

### Continuous Integration

For CI pipelines that test both languages:

```yaml
# Example GitHub Actions workflow
jobs:
  test-python:
    steps:
      - run: cargo clean
      - run: maturin develop --features python,three_valued_logic
      - run: pytest tests/

  test-typescript:
    steps:
      - run: cargo clean
      - run: npm run build
      - run: npm test
```

**Important**: Run each language in a separate job or ensure `cargo clean` between builds.

## Code Organization

### Core Logic (Rust)

All business logic lives in language-agnostic Rust code:

```rust
// sea-core/src/policy/core.rs
impl Policy {
    pub fn evaluate(&self, graph: &Graph) -> Result<EvaluationResult, String> {
        // Core evaluation logic (shared by all languages)
    }
}
```

### Language Bindings

Bindings are thin wrappers that:

1. Convert language-specific types to Rust types
2. Call core Rust functions
3. Convert Rust results back to language-specific types

**Python binding example:**

```rust
// sea-core/src/python/graph.rs
#[pymethods]
impl Graph {
    pub fn evaluate_policy(&self, policy_json: String) -> PyResult<EvaluationResult> {
        let policy: crate::policy::Policy = serde_json::from_str(&policy_json)
            .map_err(|e| PyErr::new::<PyValueError, _>(format!("Invalid JSON: {}", e)))?;

        let result = policy.evaluate(&self.inner)
            .map_err(|e| PyErr::new::<PyRuntimeError, _>(e))?;

        Ok(result.into())  // Convert to Python type
    }
}
```

**TypeScript binding example:**

```rust
// sea-core/src/typescript/graph.rs
#[napi]
impl Graph {
    #[napi]
    pub fn evaluate_policy(&self, policy_json: String) -> Result<EvaluationResult> {
        let policy: crate::policy::Policy = serde_json::from_str(&policy_json)
            .map_err(|e| Error::from_reason(format!("Invalid JSON: {}", e)))?;

        let result = policy.evaluate(&self.inner)
            .map_err(|e| Error::from_reason(e))?;

        Ok(result.into())  // Convert to JS type
    }
}
```

## Type Conversions

### Rust → Python

```rust
// Automatic conversion via From trait
impl From<crate::policy::EvaluationResult> for python::EvaluationResult {
    fn from(result: crate::policy::EvaluationResult) -> Self {
        Self {
            is_satisfied: result.is_satisfied,
            is_satisfied_tristate: result.is_satisfied_tristate,
            violations: result.violations.into_iter().map(|v| v.into()).collect(),
        }
    }
}
```

### Rust → TypeScript

```rust
// Automatic conversion via From trait + NAPI
impl From<crate::policy::EvaluationResult> for typescript::EvaluationResult {
    fn from(result: crate::policy::EvaluationResult) -> Self {
        Self {
            is_satisfied: result.is_satisfied,
            is_satisfied_tristate: result.is_satisfied_tristate,
            violations: result.violations.into_iter().map(|v| v.into()).collect(),
        }
    }
}
```

## Naming Conventions

Different languages have different conventions. The bindings respect these:

| Concept   | Rust              | Python            | TypeScript        |
| --------- | ----------------- | ----------------- | ----------------- |
| Functions | `snake_case`      | `snake_case`      | `camelCase`       |
| Structs   | `PascalCase`      | `PascalCase`      | `PascalCase`      |
| Fields    | `snake_case`      | `snake_case`      | `camelCase`       |
| Constants | `SCREAMING_SNAKE` | `SCREAMING_SNAKE` | `SCREAMING_SNAKE` |

**Example:**

```rust
// Rust
pub struct EvaluationResult {
    pub is_satisfied: bool,
    pub is_satisfied_tristate: Option<bool>,
}
```

```python
# Python (preserves snake_case)
result.is_satisfied
result.is_satisfied_tristate
```

```typescript
// TypeScript (converts to camelCase)
result.isSatisfied;
result.isSatisfiedTristate;
```

## Testing Strategy

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_evaluation() {
        // Test core Rust logic
    }
}
```

### Integration Tests (Python)

```python
# tests/test_three_valued_eval.py
def test_policy_evaluation():
    graph = Graph()
    # Test Python bindings
```

### Integration Tests (TypeScript)

```typescript
// typescript-tests/three_valued_eval.test.ts
describe("Policy Evaluation", () => {
  it("evaluates policies correctly", () => {
    const graph = new Graph();
    // Test TypeScript bindings
  });
});
```

### Parity Tests

Ensure both languages produce identical results:

```python
# Python
result = graph.evaluate_policy(policy_json)
assert result.is_satisfied == True
```

```typescript
// TypeScript
const result = graph.evaluatePolicy(policyJson);
expect(result.isSatisfied).toBe(true);
```

## Adding New Features

When adding a new feature that should be available in both languages:

1. **Implement in Rust core:**

   ```rust
   // sea-core/src/graph/mod.rs
   impl Graph {
       pub fn new_feature(&self) -> Result<SomeType, String> {
           // Implementation
       }
   }
   ```

2. **Add Python binding:**

   ```rust
   // sea-core/src/python/graph.rs
   #[pymethods]
   impl Graph {
       pub fn new_feature(&self) -> PyResult<SomeType> {
           self.inner.new_feature()
               .map(|r| r.into())
               .map_err(|e| PyErr::new::<PyRuntimeError, _>(e))
       }
   }
   ```

3. **Add TypeScript binding:**

   ```rust
   // sea-core/src/typescript/graph.rs
   #[napi]
   impl Graph {
       #[napi]
       pub fn new_feature(&self) -> Result<SomeType> {
           self.inner.new_feature()
               .map(|r| r.into())
               .map_err(Error::from_reason)
       }
   }
   ```

4. **Export in Python:**

   ```python
   # python/sea_dsl/__init__.py
   from .sea_dsl import Graph, SomeType
   __all__ = ["Graph", "SomeType"]
   ```

5. **Export in TypeScript:**

   ```javascript
   // index.js
   module.exports.SomeType = nativeBinding.SomeType;
   ```

6. **Add tests for both languages**

## Common Pitfalls

### ❌ Mixing Features

```bash
# DON'T DO THIS
cargo build --features python,typescript  # Will cause conflicts
```

### ❌ Forgetting to Clean

```bash
# DON'T DO THIS
maturin develop --features python
npm run build  # May pick up Python artifacts
```

### ❌ Inconsistent Naming

```rust
// DON'T DO THIS
#[napi]
pub fn evaluatePolicy(...) { }  // Use snake_case in Rust

// DO THIS
#[napi]
pub fn evaluate_policy(...) { }  // NAPI will convert to camelCase
```

### ❌ Missing Type Conversions

```rust
// DON'T DO THIS
#[napi]
pub fn get_result(&self) -> crate::policy::EvaluationResult {
    // Returns Rust type directly
}

// DO THIS
#[napi]
pub fn get_result(&self) -> typescript::EvaluationResult {
    self.inner.get_result().into()  // Convert to binding type
}
```

## Troubleshooting

See [NAPI Build Troubleshooting Guide](./napi_build_troubleshooting.md) for common issues and solutions.

## Resources

- [PyO3 User Guide](https://pyo3.rs/)
- [NAPI-RS Documentation](https://napi.rs/)
- [Maturin User Guide](https://www.maturin.rs/)
- [Cargo Features](https://doc.rust-lang.org/cargo/reference/features.html)

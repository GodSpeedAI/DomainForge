# NAPI Build Troubleshooting Guide

## Issue: `PyBaseObject_Type` Symbol Error

### Symptom

When running TypeScript tests or loading the Node.js native module, you encounter:

```
Error: /path/to/sea-core.linux-x64-gnu.node: undefined symbol: PyBaseObject_Type
```

### Root Cause

The Node.js native module (`.node` file) was built with the `python` feature enabled, causing Python/PyO3 symbols to be linked into the binary. This happens when:

1. The `python` feature is inadvertently enabled during the build
2. Cargo's feature resolution includes `python` as a default or transitive dependency
3. Build artifacts from a previous Python build are being reused

### Solution

#### Quick Fix: Manual Clean Build

1. **Clean all build artifacts:**

   ```bash
   cargo clean
   cd sea-core && cargo clean && cd ..
   rm -rf target sea-core/target
   ```

2. **Build the native library manually:**

   ```bash
   cd sea-core
   cargo build --lib --features typescript,three_valued_logic
   ```

3. **Copy the artifact to the Node.js module location:**

   ```bash
   cp target/debug/libsea_core.so ../sea-core.linux-x64-gnu.node
   ```

4. **Verify the build is clean:**

   ```bash
   nm -D ../sea-core.linux-x64-gnu.node | grep Py
   ```

   This should return **no results**. If you see `PyBaseObject_Type` or other `Py*` symbols, the build is contaminated.

5. **Run tests:**
   ```bash
   cd ..
   npm test
   ```

#### Proper Fix: Ensure Feature Isolation

**Verify `Cargo.toml` configuration:**

```toml
[features]
default = []  # Must be empty
python = ["pyo3", "pythonize"]
typescript = ["napi", "napi-derive"]
three_valued_logic = []
```

**Verify `package.json` build script:**

```json
{
  "scripts": {
    "build": "napi build --platform --cargo-cwd sea-core --features typescript,three_valued_logic",
    "build:debug": "napi build --platform --cargo-cwd sea-core --features typescript,three_valued_logic"
  }
}
```

**Verify `napi.config.js`:**

```javascript
module.exports = {
  cargo: {
    cargoManifestPath: "./sea-core/Cargo.toml",
    cargoFeatures: ["typescript", "three_valued_logic"],
  },
};
```

### Prevention

1. **Always use clean builds when switching between Python and TypeScript:**

   ```bash
   cargo clean
   ```

2. **Never enable both `python` and `typescript` features simultaneously**

3. **Use separate build commands:**

   ```bash
   # For Python
   maturin develop --features python,three_valued_logic

   # For TypeScript (after cargo clean)
   npm run build
   ```

## Issue: TypeScript Property Naming Mismatches

### Symptom

TypeScript tests fail with errors like:

```
Property 'is_satisfied' does not exist on type 'EvaluationResult'.
Did you mean 'isSatisfied'?
```

### Root Cause

NAPI automatically converts Rust `snake_case` identifiers to JavaScript `camelCase` conventions. This is standard behavior for `#[napi]` macros.

### Solution

**Use camelCase in TypeScript/JavaScript code:**

```typescript
// ✓ Correct
const result = graph.evaluatePolicy(policyJson);
console.log(result.isSatisfied);
console.log(result.isSatisfiedTristate);

// ✗ Incorrect
console.log(result.is_satisfied); // Property does not exist
console.log(result.is_satisfied_tristate); // Property does not exist
```

**Rust struct fields remain snake_case:**

```rust
#[napi(object)]
pub struct EvaluationResult {
    pub is_satisfied: bool,  // Becomes isSatisfied in JS
    pub is_satisfied_tristate: Option<bool>,  // Becomes isSatisfiedTristate in JS
    pub violations: Vec<Violation>,
}
```

### Naming Convention Reference

| Rust (snake_case)       | JavaScript (camelCase) |
| ----------------------- | ---------------------- |
| `evaluate_policy`       | `evaluatePolicy`       |
| `is_satisfied`          | `isSatisfied`          |
| `is_satisfied_tristate` | `isSatisfiedTristate`  |
| `from_file`             | `fromFile`             |
| `resolve_files`         | `resolveFiles`         |
| `namespace_for`         | `namespaceFor`         |

## Issue: `null` vs `undefined` for Indeterminate Results

### Symptom

Tests expecting `null` for indeterminate results fail:

```
AssertionError: expected undefined to be null
```

### Root Cause

NAPI maps Rust's `Option::None` to `undefined` (missing field) in JavaScript objects, not `null`.

### Solution

**Expect `undefined` for missing optional fields:**

```typescript
// ✓ Correct
expect(result.isSatisfiedTristate).toBeUndefined();

// ✗ Incorrect
expect(result.isSatisfiedTristate).toBeNull();
```

**Understanding the difference:**

```typescript
// When isSatisfiedTristate is None in Rust:
const result = {
  isSatisfied: false,
  violations: [...]
  // isSatisfiedTristate is not present in the object
};

console.log(result.isSatisfiedTristate);  // undefined
console.log('isSatisfiedTristate' in result);  // false
```

**In Python, `None` is used:**

```python
# Python uses None for null values
assert result.is_satisfied_tristate is None
```

## Issue: Missing Static Methods (e.g., `fromFile`)

### Symptom

```
TypeError: NamespaceRegistry.from_file is not a function
```

### Root Cause

NAPI renamed the static method from `from_file` to `fromFile` (camelCase).

### Solution

```typescript
// ✓ Correct
const registry = NamespaceRegistry.fromFile(path);

// ✗ Incorrect
const registry = NamespaceRegistry.from_file(path);
```

## Build Verification Checklist

Before committing or deploying, verify:

- [ ] `cargo clean` has been run
- [ ] Only appropriate features are enabled (`typescript,three_valued_logic` for Node.js)
- [ ] `nm -D sea-core.linux-x64-gnu.node | grep Py` returns no results
- [ ] `npm test` passes all tests
- [ ] TypeScript code uses camelCase for all NAPI-exposed APIs
- [ ] Tests expect `undefined` (not `null`) for `Option::None` values

## Debugging Commands

**Check which features are enabled:**

```bash
cargo tree --manifest-path sea-core/Cargo.toml --features typescript,three_valued_logic -e features | grep python
# Should return nothing
```

**Check for Python symbols in the binary:**

```bash
nm -D sea-core.linux-x64-gnu.node | grep Py
# Should return nothing
```

**Check linked libraries:**

```bash
ldd sea-core.linux-x64-gnu.node
# Should NOT include libpython
```

**Verify NAPI exports:**

```bash
nm -D sea-core.linux-x64-gnu.node | grep napi_register
# Should show napi_register_module_v1
```

## Related Files

- `sea-core/Cargo.toml` - Feature definitions
- `package.json` - Build scripts
- `napi.config.js` - NAPI configuration
- `sea-core/src/lib.rs` - Feature-gated module imports
- `sea-core/build.rs` - Build script

## Additional Resources

- [NAPI-RS Documentation](https://napi.rs/)
- [PyO3 User Guide](https://pyo3.rs/)
- [Cargo Features Documentation](https://doc.rust-lang.org/cargo/reference/features.html)

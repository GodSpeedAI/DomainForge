# ADR-005: Multi-Language Support Strategy

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

DomainForge needs to support multiple runtime environments to maximize adoption:

- **Python**: Data scientists, ML engineers, enterprise automation
- **TypeScript/JavaScript**: Web applications, serverless functions, Node.js backends
- **WebAssembly (WASM)**: Browser-based tools, edge computing, sandboxed execution
- **Rust**: High-performance systems, CLI tools, core library consumers

Maintaining separate implementations for each language would lead to:

- Feature drift between implementations
- Multiplicative testing burden
- Inconsistent behavior across platforms

## Decision

Implement a **single Rust core** (`sea-core`) that exposes bindings to other languages via FFI:

| Target             | Binding Technology | Feature Flag |
| ------------------ | ------------------ | ------------ |
| Python             | PyO3 + Maturin     | `python`     |
| TypeScript/Node.js | NAPI-RS            | `typescript` |
| WebAssembly        | wasm-bindgen       | `wasm`       |
| Rust               | Native crate       | (default)    |

### Binding Architecture

```
┌─────────────────────────────────────────────────────┐
│                    sea-core (Rust)                  │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌──────────┐  │
│  │ Parser  │ │  Graph  │ │ Policy  │ │Projection│  │
│  └─────────┘ └─────────┘ └─────────┘ └──────────┘  │
└─────────────────────────────────────────────────────┘
         │              │              │
    ┌────┴────┐    ┌────┴────┐    ┌────┴────┐
    │  PyO3   │    │ NAPI-RS │    │  WASM   │
    │ Binding │    │ Binding │    │ Binding │
    └────┬────┘    └────┬────┘    └────┬────┘
         │              │              │
    ┌────┴────┐    ┌────┴────┐    ┌────┴────┐
    │ Python  │    │  Node   │    │ Browser │
    │  sea_dsl│    │  @sea   │    │  @sea   │
    └─────────┘    └─────────┘    └─────────┘
```

### Binding Wrapper Pattern

Each binding module wraps core Rust types with language-idiomatic APIs:

```rust
// Python binding example (sea-core/src/python/primitives.rs)
#[pyclass]
pub struct Entity {
    inner: crate::primitives::Entity,
}

#[pymethods]
impl Entity {
    #[new]
    fn new(name: String, namespace: String) -> Self { /* ... */ }

    #[getter]
    fn name(&self) -> &str { self.inner.name() }
}
```

## Consequences

### Positive

- **Single source of truth**: All semantics defined once in Rust.
- **Consistent behavior**: All platforms use identical parsing, validation, and projection logic.
- **Performance**: Core operations run at native speed; only boundary crossing has overhead.
- **Reduced maintenance**: Bug fixes and features automatically propagate to all bindings.

### Negative

- **Rust expertise required**: Contributors must understand Rust for core changes.
- **FFI complexity**: Each binding layer introduces serialization/deserialization overhead.
- **Build complexity**: CI must build and test all target platforms.

## Implementation Notes

### Feature Flags

Bindings are conditionally compiled via Cargo features:

```toml
[features]
python = ["pyo3", "pythonize"]
typescript = ["napi", "napi-derive"]
wasm = ["wasm-bindgen", "serde-wasm-bindgen"]
```

### Data Serialization

- Python: `pythonize` for automatic serde ↔ Python dict conversion
- TypeScript: `napi` with serde for JSON-like object passing
- WASM: `serde-wasm-bindgen` for efficient JS object bridging

## Related

- [ADR-006: Error Handling Strategy](./ADR-006-error-handling-strategy.md)
- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)

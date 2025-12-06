# Cross-Language Binding Strategy

DomainForge is designed to be usable from Python, TypeScript, and WebAssembly while maintaining a single, high-performance Rust core. This document explains how we achieve this.

## The "Wrapper" Pattern

We do not rewrite business logic. We wrap Rust structs in language-specific containers.

### Python (PyO3)

- **Crate**: `pyo3`
- **Implementation**: `sea-core/src/python/`
- **Mechanism**: Rust structs are annotated with `#[pyclass]`. Methods are annotated with `#[pymethods]`.
- **Memory**: Python objects hold a reference to the underlying Rust data. When the Python object is garbage collected, the Rust memory is freed (if no other references exist).

### TypeScript (napi-rs)

- **Crate**: `napi`
- **Implementation**: `sea-core/src/typescript/`
- **Mechanism**: We define structs that mirror the core primitives but are compatible with N-API.
- **Async**: Heavy operations (like parsing large files) can be offloaded to the libuv thread pool to avoid blocking the Node.js event loop.

### WebAssembly (wasm-bindgen)

- **Crate**: `wasm-bindgen`
- **Implementation**: `sea-core/src/wasm/`
- **Mechanism**: Rust types are compiled to WASM. JavaScript "glue code" is generated to bridge the gap.
- **Constraint**: WASM runs in a sandbox. File I/O is not directly available; input must be passed as strings or byte arrays.

## Synchronization Rules

To maintain consistency, we follow strict rules:

1. **Core First**: Changes happen in `sea-core` first.
2. **Update All Bindings**: If a field is added to `Entity` in Rust, it *must* be exposed in Python and TypeScript immediately.
3. **Unified Testing**: `just all-tests` runs the test suites for all languages. A failure in Python bindings blocks the Rust PR.

## Error Handling

Rust errors (`Result<T, E>`) are mapped to language-specific exceptions:

- **Rust**: `sea_core::error::SeaError`
- **Python**: `sea.SeaError` (inherits from `Exception`)
- **TypeScript**: `Error` object with specific code properties.

## See Also

- [Architecture Overview](architecture-overview.md)
- [Adding a New Primitive](../playbooks/adding-new-primitive.md)

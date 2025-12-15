# ADR-006: Error Handling Strategy

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

SEA-DSL operations can fail in multiple ways:

- **Parse errors**: Invalid syntax in `.sea` files
- **Validation errors**: Semantic violations (e.g., referencing undefined entities)
- **Policy violations**: Business rule failures during evaluation
- **Runtime errors**: I/O failures, module resolution issues

Errors must:

1. Provide actionable diagnostics (file, line, column, suggestions)
2. Cross FFI boundaries cleanly (Rust → Python/TypeScript/WASM)
3. Support multiple output formats (human-readable, JSON, LSP)

## Decision

### Error Type Hierarchy

```rust
// Core validation error with rich diagnostics
pub struct ValidationError {
    pub code: ErrorCode,
    pub message: String,
    pub range: Option<SourceRange>,
    pub severity: Severity,
    pub suggestions: Vec<String>,
}

// Structured error codes for programmatic handling
pub enum ErrorCode {
    // Syntax errors (1xxx)
    SyntaxError = 1000,
    UnexpectedToken = 1001,

    // Semantic errors (2xxx)
    UndefinedEntity = 2000,
    DuplicateDeclaration = 2001,
    TypeMismatch = 2002,

    // Policy errors (3xxx)
    PolicyViolation = 3000,
    ConstraintFailed = 3001,

    // Module errors (4xxx)
    ModuleNotFound = 4000,
    CircularImport = 4001,
}
```

### Diagnostic Formatters

Multiple output formats via the `DiagnosticFormatter` trait:

| Formatter        | Use Case           | Output                       |
| ---------------- | ------------------ | ---------------------------- |
| `HumanFormatter` | CLI output         | Colored, contextual snippets |
| `JsonFormatter`  | CI/automation      | Structured JSON array        |
| `LspFormatter`   | Editor integration | LSP Diagnostic objects       |

### FFI Error Propagation

Errors are converted to native exceptions at language boundaries:

```rust
// Python: Convert to Python exception
#[pyfunction]
fn parse(source: &str) -> PyResult<Graph> {
    crate::parse_to_graph(source)
        .map_err(|e| PyValueError::new_err(e.to_string()))
}

// TypeScript: Return Result-like object or throw
#[napi]
fn parse(source: String) -> napi::Result<Graph> {
    crate::parse_to_graph(&source)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}
```

### "Did You Mean?" Suggestions

The `fuzzy` module provides Levenshtein-distance-based suggestions:

```rust
// When entity "Wharehouse" not found, suggest "Warehouse"
if let Some(suggestion) = fuzzy::did_you_mean("Wharehouse", known_entities) {
    error.suggestions.push(format!("Did you mean '{}'?", suggestion));
}
```

## Consequences

### Positive

- **Actionable errors**: Users get file locations and fix suggestions.
- **Programmatic handling**: Error codes enable automated error categorization.
- **IDE integration**: LSP formatter enables real-time editor diagnostics.
- **Cross-platform consistency**: Same error structure across all bindings.

### Negative

- **Complexity**: Multiple formatter implementations to maintain.
- **FFI overhead**: Error details may be lost when converting to simple strings.

## Related

- [ADR-005: Multi-Language Support Strategy](./ADR-005-multi-language-support-strategy.md)
- [SDS-002: SEA Core Architecture](./SDS-002-sea-core-architecture.md)

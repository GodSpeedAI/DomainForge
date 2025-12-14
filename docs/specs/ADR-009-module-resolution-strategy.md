# ADR-009: Module Resolution Strategy

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

Large SEA models need to be split across multiple files for maintainability. This requires:

1. Import syntax for referencing external SEA files
2. Namespace resolution for qualified references
3. Standard library of reusable types
4. Circular dependency detection

## Decision

### Import Syntax

```sea
// Relative file import
import "common/types.sea"

// Namespace binding
import namespace core from "std/core.sea"

// Selective import
import { Customer, Order } from "domain/entities.sea"
```

### Namespace Registry

A `.sea-registry.toml` file at the project root maps namespace prefixes to file paths:

```toml
[namespaces]
core = "sea-core/std/core.sea"
http = "sea-core/std/http.sea"
aws = "sea-core/std/aws.sea"
domain = "src/domain/index.sea"
```

### Resolution Algorithm

1. Check if namespace is in registry
2. Resolve relative to entry file if not found
3. Search standard library paths
4. Fail with helpful error message

```rust
pub struct ModuleResolver {
    registry: NamespaceRegistry,
    loaded: HashMap<PathBuf, Ast>,
}

impl ModuleResolver {
    pub fn resolve(&mut self, import: &ImportSpec, from: &Path) -> Result<Ast, ParseError> {
        // 1. Try registry lookup
        if let Some(path) = self.registry.resolve(&import.namespace) {
            return self.load(path);
        }

        // 2. Try relative resolution
        let relative = from.parent().unwrap().join(&import.path);
        if relative.exists() {
            return self.load(&relative);
        }

        // 3. Fail with suggestion
        Err(ParseError::ModuleNotFound { /* ... */ })
    }
}
```

### Circular Dependency Detection

Track loading state to detect cycles:

```rust
enum LoadState {
    Loading,    // Currently being parsed
    Loaded(Ast), // Successfully loaded
}

// If we encounter Loading while resolving, it's circular
```

### Standard Library

Built-in types available via `std:` prefix:

| Module     | Contents                       |
| ---------- | ------------------------------ |
| `std:core` | Basic types (UUID, Email, URL) |
| `std:http` | HTTP request/response patterns |
| `std:aws`  | AWS resource types             |

## Consequences

### Positive

- **Modularity**: Large models split into maintainable files
- **Reusability**: Standard library provides common patterns
- **Clarity**: Explicit namespace bindings reduce ambiguity
- **Safety**: Circular dependencies caught at parse time

### Negative

- **Configuration**: Requires `.sea-registry.toml` setup
- **Complexity**: Resolution logic adds parsing overhead

## Related

- [SDS-003: Parser and Semantic Graph](./SDS-003-parser-semantic-graph.md)
- [ADR-001: SEA-DSL as Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)

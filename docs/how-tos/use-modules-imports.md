# Use SEA Modules and Imports

This guide explains how to organize large SEA models across multiple files using imports and the namespace registry.

## Overview

SEA supports modular organization through:

- **Imports**: Reference types from other SEA files
- **Namespace Registry**: Map namespace prefixes to file paths
- **Standard Library**: Built-in types via `std:` prefix

## Prerequisites

- `sea` CLI installed
- A project with multiple `.sea` files

## Quick Start

### 1. Create Multiple Files

**`common/types.sea`**

```sea
Namespace "common"

Entity "BaseEntity" {
    id: UUID
    created_at: DateTime
    updated_at: DateTime
}
```

**`domain/customer.sea`**

```sea
Namespace "domain.customer"

import "common/types.sea"

Entity "Customer" {
    @extends "BaseEntity"
    name: String
    email: String
}
```

### 2. Configure the Registry

Create `.sea-registry.toml` at your project root:

```toml
[namespaces]
common = "common/types.sea"
domain = "domain/index.sea"
```

### 3. Use Imports

```bash
sea validate domain/customer.sea --registry .sea-registry.toml
```

## Import Syntax

### Relative File Import

Import all types from a file:

```sea
import "common/types.sea"
```

### Namespace Binding

Import with namespace alias:

```sea
import namespace core from "std/core.sea"

Entity "User" {
    email: core.Email
}
```

### Selective Import

Import specific types:

```sea
import { Customer, Order } from "domain/entities.sea"
```

### Wildcard Import (with alias)

```sea
import * as billing from "billing/types.sea"

Flow from Customer to billing.Invoice
```

## Namespace Registry

The `.sea-registry.toml` file maps namespace prefixes to file paths:

```toml
[registry]
paths = ["./src", "./vendor"]

[namespaces]
core = "sea-core/std/core.sea"
http = "sea-core/std/http.sea"
aws = "sea-core/std/aws.sea"
domain = "src/domain/index.sea"
```

### Resolution Priority

1. **Registry lookup**: Check `[namespaces]` table
2. **Path search**: Search directories in `[registry].paths`
3. **Relative resolution**: Resolve relative to the importing file
4. **Standard library**: Check built-in `std:` modules

## Standard Library

Built-in types available via `std:` prefix (no registry needed):

| Module     | Contents                              |
| ---------- | ------------------------------------- |
| `std:core` | UUID, Email, URL, DateTime            |
| `std:http` | Request, Response, Headers            |
| `std:aws`  | S3Bucket, LambdaFunction, DynamoTable |

### Usage

```sea
import namespace core from "std:core"

Entity "User" {
    id: core.UUID
    email: core.Email
    website: core.URL
}
```

## Index Files

Create `index.sea` files to re-export from a namespace:

**`domain/index.sea`**

```sea
Namespace "domain"

// Re-export from sub-modules
export * from "domain/customer.sea"
export * from "domain/order.sea"
export * from "domain/payment.sea"
```

Then import the namespace:

```sea
import namespace domain from "domain/index.sea"

Flow from domain.Customer to domain.Order
```

## Circular Dependency Detection

The parser detects circular imports:

```
Error[E010]: Circular module dependency detected
  --> domain/order.sea:3
  |
3 | import "domain/customer.sea"
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^
  |
  = note: dependency chain: customer.sea -> order.sea -> customer.sea
```

### Resolution

Break cycles by:

1. Moving shared types to a `common` module
2. Using interface types instead of concrete imports
3. Restructuring module boundaries

## Project Structure Best Practices

```
my-project/
├── .sea-registry.toml
├── common/
│   ├── types.sea         # Base types
│   └── units.sea         # Custom units
├── domain/
│   ├── index.sea         # Re-exports
│   ├── customer.sea
│   ├── order.sea
│   └── payment.sea
├── policies/
│   ├── compliance.sea
│   └── business-rules.sea
└── main.sea              # Entry point
```

## CLI Options

### Validate with Registry

```bash
sea validate main.sea --registry .sea-registry.toml
```

### Specify Additional Paths

```bash
sea validate main.sea --include ./vendor --include ./lib
```

### Verbose Module Loading

```bash
SEA_LOG=debug sea validate main.sea
```

## Programmatic Usage

### Rust

```rust
use sea_core::registry::NamespaceRegistry;
use sea_core::parser::parse_with_registry;

let registry = NamespaceRegistry::from_file(".sea-registry.toml")?;
let graph = parse_with_registry("main.sea", &registry)?;
```

### Python

```python
from sea_dsl import Graph, NamespaceRegistry

registry = NamespaceRegistry.from_file(".sea-registry.toml")
graph = Graph.parse_with_registry(open("main.sea").read(), registry)
```

## Troubleshooting

### Module Not Found

```
Error[E009]: Module not found: 'common/types.sea'
  --> domain/customer.sea:3
  |
  = hint: Check .sea-registry.toml or use --include flag
```

**Fix**: Ensure the file path is correct or add to registry.

### Namespace Conflict

```
Error[E011]: Namespace conflict: 'Customer' defined in multiple modules
```

**Fix**: Use namespace aliases or rename conflicting types.

### Registry Not Found

```
Warning: No .sea-registry.toml found, using default resolution
```

**Fix**: Create `.sea-registry.toml` in project root.

## See Also

- [ADR-009: Module Resolution Strategy](../specs/ADR-009-module-resolution-strategy.md)
- [Configuration Reference](../reference/configuration.md)
- [Registry Reference](../reference/registry.md)
- [CLI Commands](../reference/cli-commands.md)

# How-To: Configure Multi-File Projects & Module Resolution

This guide explains how to split large enterprise models across multiple `.sea` files using import declarations and the `.sea-registry.toml` namespace registry.

---

## Goal
Structure a multi-file domain model with clean namespace separation, exported concepts, and transitive module resolution.

---

## 1. Create a `.sea-registry.toml` Manifest

In your repository or project root, create `.sea-registry.toml`:

```toml
[namespaces.procurement]
path = "models/procurement/**/*.sea"
version = "1.0.0"

[namespaces.shared]
path = "models/shared/**/*.sea"
version = "1.0.0"
```

---

## 2. Author Modular SEA Files

### Shared Definitions (`models/shared/types.sea`)
Use the `export` keyword to expose concepts to other modules:
```sea
@namespace "shared"
@version "1.0.0"

export Entity "Auditor" in shared

export Resource "AuditLog" units in shared
```

### Downstream Module (`models/procurement/orders.sea`)
Import exported symbols using named imports:
```sea
@namespace "procurement"
@version "1.0.0"

import { Auditor, AuditLog } from "../shared/types.sea"

Entity "Buyer" in procurement

Flow "AuditLog" from "Buyer" to "Auditor" quantity 1
```

You can also use aliases to avoid name collisions:
```sea
import { Auditor as ExternalAuditor } from "../shared/types.sea"
```

---

## 3. Validate the Modular Project

Run `domainforge validate` passing the entry file and registry:

```bash
domainforge validate models/procurement/orders.sea --registry .sea-registry.toml
```

The `ModuleResolver` will automatically:
1. Parse the entry file.
2. Recursively load and parse all imported files.
3. Verify that all imported symbols were explicitly marked `export`.
4. Construct a unified, cross-namespace semantic graph.

---

## Troubleshooting Common Resolution Errors

| Error Code | Symptom | Fix |
|---|---|---|
| `APP014_IMPORT_CYCLE` | Circular import detected between modules (e.g. A imports B, B imports A). | Move shared definitions into a third independent module (e.g. `shared.sea`) that both modules import. |
| `APP014_NOT_EXPORTED` | Symbol X is imported but not exported in target file. | Add `export` keyword before declaration in the target file. |
| `APP014_UNRESOLVED_SPECIFIER` | Import path cannot be found on disk. | Verify relative file path or check registry pattern. |

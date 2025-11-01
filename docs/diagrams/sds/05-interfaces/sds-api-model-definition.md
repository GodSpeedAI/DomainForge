---
sds_section: "5. Interface & API Design"
diagram_type: "Sequence"
component_ids: ["BIND-API-PyO3", "CORE-API-RustCore", "CORE-SVC-NamespaceResolver", "CORE-DB-GraphStore", "MODEL-COMP-EntityPrimitive"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-005"]
satisfies_requirements: ["REQ-002", "REQ-005", "REQ-006", "REQ-010", "REQ-011", "REQ-017"]
related_diagrams:
  - ../04-components/sds-component-namespace-resolver.md
  - ../04-components/sds-component-graph-runtime.md
  - ../03-architecture/sds-architecture-e2e-flow.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Illustrates the API call sequence for defining an Entity primitive via Python bindings and Rust core."
---

## API Sequence: Model Definition

```mermaid
sequenceDiagram
    %% Source: docs/specs/sds.md SDS-002, SDS-008, SDS-009
    %% Implements: ADR-001, ADR-002, ADR-005
    %% Satisfies: REQ-002, REQ-005, REQ-006, REQ-010, REQ-011, REQ-017
    %% Components: BIND-API-PyO3, CORE-API-RustCore, CORE-SVC-NamespaceResolver, CORE-DB-GraphStore, MODEL-COMP-EntityPrimitive

    participant Client as BIND-API-PyO3
    participant FFI as PyO3 FFI Layer
    participant Core as CORE-API-RustCore
    participant Namespace as CORE-SVC-NamespaceResolver
    participant Graph as CORE-DB-GraphStore

    Client->>FFI: Model.define_entity("Customer", attrs)
    FFI->>Core: define_entity(name, attrs)
    Core->>Namespace: resolve_namespace("finance")
    Namespace-->>Core: Ok(namespace_id)
    Core->>Graph: insert_entity(Entity{namespace_id, attrs})
    Graph-->>Core: Ok(EntityHandle)
    Core-->>FFI: Ok(EntityHandle)
    FFI-->>Client: Entity(id, attrs)

    alt Duplicate name
        Graph-->>Core: Err(DuplicatePrimitive)
        Core-->>FFI: Err(Error::DuplicatePrimitive)
        FFI-->>Client: raise ValueError
    end
```

### Design Rationale
- Ensures namespace resolution occurs before graph mutation to prevent drift.
- Error handling path emphasises mapping to Python exceptions (REQ-017).

### Related Components
- Graph internals documented in [sds-component-graph-runtime](../04-components/sds-component-graph-runtime.md).

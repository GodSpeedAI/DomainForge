---
sds_section: "3. Architectural Design"
diagram_type: "Sequence"
component_ids: ["BIND-API-PyO3", "CORE-API-RustCore", "CORE-SVC-NamespaceResolver", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "MODEL-COMP-FlowPrimitive"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-003", "ADR-004", "ADR-005"]
satisfies_requirements: ["REQ-002", "REQ-003", "REQ-004", "REQ-005", "REQ-006", "REQ-012", "REQ-013", "REQ-015", "REQ-017"]
related_diagrams:
  - sds-architecture-container-overview.md
  - ../05-interfaces/sds-api-model-definition.md
  - ../06-data/sds-data-schema-complete.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Captures end-to-end request flow from API client through Rust core, namespace resolver, graph store, and policy evaluator."
---

## End-to-End Modelling Sequence

Demonstrates full request lifecycle for defining a flow and validating policies, covering happy and failure paths per validation checklist.

```mermaid
sequenceDiagram
    %% Source: docs/specs/sds.md Sections SDS-001..SDS-006
    %% Implements: ADR-001, ADR-002, ADR-003, ADR-004, ADR-005
    %% Satisfies: REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-012, REQ-013, REQ-015, REQ-017
    %% Components: BIND-API-PyO3, CORE-API-RustCore, CORE-SVC-NamespaceResolver, CORE-DB-GraphStore, VAL-SVC-PolicyEvaluator, MODEL-COMP-FlowPrimitive

    participant Client as BIND-API-PyO3
    participant Core as CORE-API-RustCore
    participant Namespace as CORE-SVC-NamespaceResolver
    participant Graph as CORE-DB-GraphStore
    participant Policy as VAL-SVC-PolicyEvaluator

    Client->>Core: define_flow(name, resource, from, to, qty)
    Core->>Namespace: resolve("finance::Transfer")
    Namespace-->>Core: UUID success
    Core->>Graph: upsert Flow primitive
    Graph-->>Core: Persisted MODEL-COMP-FlowPrimitive
    Core->>Policy: validate(graph_state)
    Policy->>Graph: traverse relation indexes
    Graph-->>Policy: adjacency set
    Policy-->>Core: ValidationResult {violations: []}
    Core-->>Client: Ok(FlowHandle)

    alt Error: missing namespace
        Namespace-->>Core: Error::UndefinedNamespace
        Core-->>Client: Err(CardinalityViolation?)
    end

    rect rgb(255, 240, 240)
        Note over Policy,Client: Implements REQ-017 diagnostics via structured errors
        Policy->>Client: stream_violation(Violation)
    end
```

### Design Rationale
- Combined synchronous + streaming interactions show compliance with REQ-013.
- Error alternate path ensures diagnostics remain explicit (REQ-017).

### Related Components
- Namespace resolution internals defined in [sds-component-namespace-resolver](../04-components/sds-component-namespace-resolver.md).
- Graph schema referenced in [sds-data-schema-complete](../06-data/sds-data-schema-complete.md).

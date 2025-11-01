---
sds_section: "5. Interface & API Design"
diagram_type: "Sequence"
component_ids: ["BIND-API-PyO3", "CORE-API-RustCore", "VAL-SVC-PolicyEvaluator", "CORE-DB-GraphStore"]
implements_adrs: ["ADR-002", "ADR-003", "ADR-004"]
satisfies_requirements: ["REQ-003", "REQ-005", "REQ-012", "REQ-013", "REQ-015"]
related_diagrams:
  - ../03-architecture/sds-architecture-service-interactions.md
  - ../04-components/sds-component-policy-engine.md
  - sds-api-ffi-error-handling.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Describes synchronous and streaming validation flows, including backpressure handling."
---

## API Sequence: Validation with Streaming

```mermaid
sequenceDiagram
    %% Source: docs/specs/sds.md SDS-006, SDS-008, SDS-009
    %% Implements: ADR-002, ADR-003, ADR-004
    %% Satisfies: REQ-003, REQ-005, REQ-012, REQ-013, REQ-015
    %% Components: BIND-API-PyO3, CORE-API-RustCore, VAL-SVC-PolicyEvaluator, CORE-DB-GraphStore

    participant Client as BIND-API-PyO3
    participant Core as CORE-API-RustCore
    participant Policy as VAL-SVC-PolicyEvaluator
    participant Graph as CORE-DB-GraphStore
    participant Stream as AsyncStream

    Client->>Core: validate_streaming(callback)
    Core->>Policy: spawn_validation(graph_snapshot)
    Policy->>Graph: iterate_policies()
    Graph-->>Policy: policy_iter
    loop policy
        Policy->>Graph: expand_quantifier(policy)
        Graph-->>Policy: contexts
        alt violation detected
            Policy->>Stream: send(Violation)
            Stream-->>Client: callback(violation)
        else compliant
            Policy-->>Policy: memoize result
        end
    end
    Policy-->>Core: ValidationSummary
    Core-->>Client: summary

    rect rgb(255, 240, 240)
        Note over Policy,Client: Backpressure - Stream uses bounded channel (ADR-004)
    end
```

### Design Rationale
- Async stream ensures first violation delivered before evaluation completes (REQ-013).
- Bounded channel prevents memory explosion (security/performance mitigation).

### Related Components
- Error propagation for callbacks detailed in [sds-api-ffi-error-handling](sds-api-ffi-error-handling.md).

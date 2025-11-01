---
sds_section: "6. Data Design"
diagram_type: "State"
component_ids: ["MODEL-COMP-FlowPrimitive", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator"]
implements_adrs: ["ADR-003", "ADR-004", "ADR-005"]
satisfies_requirements: ["REQ-002", "REQ-004", "REQ-012"]
related_diagrams:
  - sds-data-schema-complete.md
  - ../03-architecture/sds-architecture-error-handling.md
  - ../08-deployment/sds-deployment-observability.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Models flow lifecycle states and consistency guarantees."
---

## Flow Consistency State Machine

```mermaid
stateDiagram-v2
    %% Source: docs/specs/sds.md SDS-004, SDS-006
    %% Implements: ADR-003, ADR-004, ADR-005
    %% Satisfies: REQ-002, REQ-004, REQ-012
    %% Components: MODEL-COMP-FlowPrimitive, CORE-DB-GraphStore, VAL-SVC-PolicyEvaluator

    [*] --> Draft
    Draft --> PendingValidation: save()
    PendingValidation --> Valid: validation_pass()
    PendingValidation --> Invalid: violation_detected()
    Invalid --> Draft: fix_and_retry()
    Valid --> Completed: set_completed_at()
    Completed --> Archived: archive()
    Archived --> [*]

    state PendingValidation {
        [*] --> Traversal
        Traversal --> QuantifierExpansion
        QuantifierExpansion --> StreamingEmit
        StreamingEmit --> Traversal: next_context
    }
```

### Design Rationale
- Captures interplay between graph persistence and policy evaluation states.
- Nested state machine summarises validation processing steps (ensures coverage for streaming).

### Related Components
- Observability metrics for state transitions defined in [sds-deployment-observability](../08-deployment/sds-deployment-observability.md).

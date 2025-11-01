---
sds_section: "6. Data Design"
diagram_type: "ER"
component_ids: ["MODEL-COMP-EntityPrimitive", "MODEL-COMP-ResourcePrimitive", "MODEL-COMP-FlowPrimitive", "MODEL-COMP-InstancePrimitive", "VAL-SVC-PolicyEvaluator", "CORE-DB-GraphStore"]
implements_adrs: ["ADR-003", "ADR-005"]
satisfies_requirements: ["REQ-002", "REQ-004", "REQ-011", "REQ-012"]
related_diagrams:
  - ../04-components/sds-component-graph-runtime.md
  - sds-data-consistency-model.md
  - sds-data-pipeline-events.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Documents full SEA DSL data schema including primitives, indexes, and relationships."
---

## ER Diagram: SEA DSL Primitives

```mermaid
erDiagram
    %% Source: docs/specs/sds.md SDS-002..SDS-005
    %% Implements: ADR-003, ADR-005
    %% Satisfies: REQ-002, REQ-004, REQ-011, REQ-012
    %% Components: MODEL-COMP-EntityPrimitive, MODEL-COMP-ResourcePrimitive, MODEL-COMP-FlowPrimitive, MODEL-COMP-InstancePrimitive, VAL-SVC-PolicyEvaluator, CORE-DB-GraphStore

    Entity {
        uuid id PK
        string name
        string namespace
    }

    Resource {
        uuid id PK
        string name
        string namespace
        string unit
    }

    Flow {
        uuid id PK
        string name
        uuid resource_id FK
        uuid from_entity_id FK
        uuid to_entity_id FK
        decimal quantity
        uuid status_enum_id
        datetime initiated_at
        datetime completed_at
    }

    Instance {
        uuid id PK
        uuid resource_id FK
        uuid entity_id FK
        jsonb attributes
    }

    Policy {
        uuid id PK
        string name
        bool active
        string severity
        text message
        text rule_expression
    }

    Flow }o--|| Entity : from_entity
    Flow }o--|| Entity : to_entity
    Flow }o--|| Resource : moves
    Instance }o--|| Resource : of
    Instance }o--|| Entity : located_at
    Policy }o--o{ Flow : applies_to
    Policy }o--o{ Entity : applies_to
    Policy }o--o{ Resource : applies_to
```

### Design Rationale
- Captures essential relationships for validation queries (REQ-012).
- Fields such as `status_enum_id` allow linking to enumerations defined elsewhere.

### Related Components
- Consistency constraints elaborated in [sds-data-consistency-model](sds-data-consistency-model.md).

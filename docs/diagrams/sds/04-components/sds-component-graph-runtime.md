---
sds_section: "4. Component Design"
diagram_type: "C4Component"
component_ids: ["CORE-DB-GraphStore", "MODEL-COMP-EntityPrimitive", "MODEL-COMP-ResourcePrimitive", "MODEL-COMP-FlowPrimitive", "MODEL-COMP-InstancePrimitive", "VAL-SVC-PolicyEvaluator"]
implements_adrs: ["ADR-003", "ADR-005"]
satisfies_requirements: ["REQ-002", "REQ-004", "REQ-011", "REQ-012"]
related_diagrams:
  - ../03-architecture/sds-architecture-container-overview.md
  - ../06-data/sds-data-schema-complete.md
  - sds-component-policy-engine.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Breaks down graph storage and index components supporting traversal and validation."
---

## Component Design: Graph Runtime

Captures hash maps and indexes enabling O(1) lookups and efficient policy evaluation.

```mermaid
C4Component
    %% Source: docs/specs/sds.md - SDS-005 Graph Data Structure
    %% Implements: ADR-003, ADR-005
    %% Satisfies: REQ-002, REQ-004, REQ-011, REQ-012
    %% Components: CORE-DB-GraphStore, MODEL-COMP-EntityPrimitive, MODEL-COMP-ResourcePrimitive, MODEL-COMP-FlowPrimitive, MODEL-COMP-InstancePrimitive, VAL-SVC-PolicyEvaluator

    AddElementTag("Storage", $bgColor="#ede9fe", $borderColor="#7c3aed", $fontColor="#5b21b6")
    AddElementTag("Index", $bgColor="#e0f2fe", $borderColor="#0284c7", $fontColor="#0369a1")

    Container_Boundary(graphBoundary, "CORE-DB-GraphStore", "Rust struct", $tags="Storage") {
        Component(entityMap, "EntityMap", "HashMap<Uuid, Entity>", "Stores entities", $tags="Storage")
        Component(resourceMap, "ResourceMap", "HashMap<Uuid, Resource>", "Stores resources", $tags="Storage")
        Component(flowMap, "FlowMap", "HashMap<Uuid, Flow>", "Stores flows", $tags="Storage")
        Component(instanceMap, "InstanceMap", "HashMap<Uuid, Instance>", "Stores instances", $tags="Storage")

        Component(fromIndex, "FromIndex", "HashMap<Uuid, Uuid>", "Flow → source entity", $tags="Index")
        Component(toIndex, "ToIndex", "HashMap<Uuid, Uuid>", "Flow → dest entity", $tags="Index")
        Component(resourceIndex, "MovesIndex", "HashMap<Uuid, Uuid>", "Flow → resource", $tags="Index")
        Component(entityIncoming, "EntityIncoming", "HashMap<Uuid, Vec<Uuid>>", "Entity → incoming flows", $tags="Index")
        Component(entityOutgoing, "EntityOutgoing", "HashMap<Uuid, Vec<Uuid>>", "Entity → outgoing flows", $tags="Index")
        Component(resourceFlows, "ResourceFlows", "HashMap<Uuid, Vec<Uuid>>", "Resource → flows", $tags="Index")
    }

    Component(policyEngine, "VAL-SVC-PolicyEvaluator", "Uses indexes for traversal")

    Rel(entityMap, policyEngine, "provides entity nodes")
    Rel(flowMap, policyEngine, "provides flow edges")
    Rel(resourceIndex, policyEngine, "enables resource lookups")
    Rel(entityIncoming, policyEngine, "optimises forall quantifiers")
```

### Traversal Sequence

```mermaid
sequenceDiagram
    %% Traversal for upstream query

    participant API as CORE-API-RustCore
    participant Graph as CORE-DB-GraphStore
    participant Index as EntityOutgoing

    API->>Graph: flow_upstream(flow_id, hops=3)
    Graph->>Index: outgoing = entity_outgoing(entity_id)
    Index-->>Graph: [flow_id1, flow_id2]
    Graph->>API: Vec<Flow>
```

### Design Rationale
- Index structures align with REQ-004 performance requirements (<100ms validation).
- HashMap-based storage ensures constant-time lookups for primitives.

### Related Components
- Policy evaluator’s use of indexes detailed in [sds-component-policy-engine](sds-component-policy-engine.md).

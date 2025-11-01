---
sds_section: "3. Architectural Design"
diagram_type: "Flowchart"
component_ids: ["CORE-API-RustCore", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "PARSE-SVC-GrammarParser", "BIND-API-PyO3", "BIND-API-TypeScript"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-003", "ADR-004", "ADR-007"]
satisfies_requirements: ["REQ-003", "REQ-004", "REQ-005", "REQ-006", "REQ-007", "REQ-012", "REQ-013", "REQ-015", "REQ-017"]
related_diagrams:
  - sds-architecture-container-overview.md
  - ../04-components/sds-component-policy-engine.md
  - ../05-interfaces/sds-api-model-definition.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Illustrates collaboration between parser, core API, graph store, and policy engine during model lifecycle operations."
---

## Service Interaction Flow

Depicts synchronous interactions for model definition and validation, including error pathways mandated by validation matrix.

```mermaid
graph TD
    %% Source: docs/specs/sds.md Sections SDS-006, SDS-007, SDS-008
    %% Implements: ADR-001, ADR-002, ADR-003, ADR-004, ADR-007
    %% Satisfies: REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-012, REQ-013, REQ-015, REQ-017
    %% Components: PARSE-SVC-GrammarParser, CORE-API-RustCore, CORE-DB-GraphStore, VAL-SVC-PolicyEvaluator, BIND-API-PyO3, BIND-API-TypeScript

    Client["BIND-API-* Client"]
    Parser["PARSE-SVC-GrammarParser"]
    CoreAPI["CORE-API-RustCore"]
    Graph["CORE-DB-GraphStore"]
    Policy["VAL-SVC-PolicyEvaluator"]
    Stream["Streaming Channel"]

    Client -->|1. parse(source)| Parser
    Parser -->|2. AST| CoreAPI
    CoreAPI -->|3. resolve namespaces| Graph
    Graph -->|4. namespace ok?| CoreAPI
    CoreAPI -->|5. apply primitive mutators| Graph
    CoreAPI -->|6. run validation| Policy
    Policy -->|7. traverses graph| Graph
    Policy -->|8. violations?| CoreAPI
    CoreAPI -->|9a. sync response| Client
    Policy -->|9b. stream_violation| Stream
    Stream -->|10. deliver events| Client

    CoreAPI -.->|error| Client
    Policy -.->|diagnostic errors| Client
    Graph -.->|resolution failure| CoreAPI
```

### Design Rationale
- Highlights interplay of parser and graph store before validation.
- Shows dual path for synchronous result and streaming events to satisfy REQ-013.

### Related Components
- Sequence detail captured in [sds-api-validation-streaming](../05-interfaces/sds-api-validation-streaming.md).

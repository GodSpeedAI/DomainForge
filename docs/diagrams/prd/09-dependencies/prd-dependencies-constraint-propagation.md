---
prd_section: "9. Dependencies & Constraints"
diagram_type: "Flowchart"
ears_requirements: ["PRD-001", "PRD-002", "PRD-004", "PRD-012", "PRD-013", "PRD-015", "PRD-016"]
updated: "2025-11-01"
purpose: "Shows how changes ripple through SEA DSL subsystems, surfacing dependency risks identified in the PRD."
---

## PRD Section 9: Constraint Propagation Flow

```mermaid
graph TD
    %% Source: docs/specs/prd.md - Dependencies & success metrics
    A[Primitive schema change]
    B[Graph indexes rebuild]
    C[Policy evaluation update]
    D[Streaming API latency]
    E[Parity regression checks]
    F[Lockstep release decision]
    G[Docs & migration guides]

    A --> B
    B --> C
    C --> D
    D --> E
    E --> F
    F --> G

    %% Feedback loops
    E -->|Failure| C
    F -->|Release blocked| A

    %% WARNING: Insufficient detail on SLA for index rebuild; capture during implementation planning.
```

---
prd_section: "7. Requirements (EARS)"
diagram_type: "Flowchart"
ears_requirements: ["PRD-001", "PRD-002", "PRD-003", "PRD-004", "PRD-005", "PRD-006", "PRD-007", "PRD-008", "PRD-009", "PRD-010", "PRD-011", "PRD-012", "PRD-013", "PRD-014", "PRD-015", "PRD-016", "PRD-017", "PRD-018", "PRD-019"]
updated: "2025-11-01"
purpose: "Categories all PRD requirements by capability pillar and highlights their EARS patterns with mandated color coding."
---

## PRD Section 7: Requirement Category Map

```mermaid
graph TD
    %% Source: docs/specs/prd.md - PRD-001..PRD-019
    R0[Requirement Portfolio]
    R0 --> R1[Core Modeling\n(PRD-001, PRD-002, PRD-003, PRD-004)]
    R0 --> R2[Runtime & Engine\n(PRD-005, PRD-012, PRD-017)]
    R0 --> R3[Language Bindings\n(PRD-006, PRD-007, PRD-008, PRD-015)]
    R0 --> R4[Interoperability & Extensibility\n(PRD-009, PRD-010, PRD-011, PRD-014)]
    R0 --> R5[Operations & Quality\n(PRD-016, PRD-018, PRD-019)]
    R0 --> R6[Event-Driven Streaming\n(PRD-013)]

    %% EARS pattern color coding
    classDef ubiquitous fill:#e1f5ff,stroke:#1976d2,color:#0d47a1;
    classDef event fill:#fff4e1,stroke:#ff9800,color:#e65100;

    class R1,R2,R3,R4,R5 ubiquitous;
    class R6 event;
```

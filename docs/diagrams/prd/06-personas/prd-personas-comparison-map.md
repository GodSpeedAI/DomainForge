---
prd_section: "6. User Personas & Scenarios"
diagram_type: "Flowchart"
ears_requirements: ["PRD-001", "PRD-002", "PRD-003", "PRD-006", "PRD-007", "PRD-012", "PRD-015", "PRD-017", "PRD-019"]
updated: "2025-11-01"
purpose: "Contrasts core personas, highlighting distinct needs and shared pain points to validate coverage of PRD user stories." 
---

## PRD Section 6: Persona Comparison Map

```mermaid
graph LR
    %% Source: docs/specs/prd.md - User stories for PRD-001..PRD-019
    BA[Business Analyst\nGoal: editable vocabulary]
    CO[Compliance Officer\nGoal: enforce policies]
    PE[Platform Engineer\nGoal: performant releases]

    N1[Need: Layered modeling
(PRD-001)]
    N2[Need: SBVR-aligned expressions
(PRD-003)]
    N3[Need: Streaming validation
(PRD-013)]
    N4[Need: Rust performance core
(PRD-005)]
    N5[Need: Idiomatic bindings
(PRD-006/7/8)]
    N6[Pain Point: Diagnostics clarity
(PRD-017)]
    N7[Pain Point: Documentation depth
(PRD-019)]

    BA --> N1
    BA --> N5
    BA --> N6
    CO --> N2
    CO --> N3
    CO --> N6
    PE --> N4
    PE --> N5
    PE --> N7

    %% NOTE: Persona satisfaction scores recorded in accompanying journey diagram.
```

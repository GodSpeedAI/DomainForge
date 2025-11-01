---
prd_section: "5. Scope & Out-of-Scope"
diagram_type: "C4Context"
ears_requirements: ["PRD-001", "PRD-002", "PRD-003", "PRD-004", "PRD-005", "PRD-006", "PRD-007", "PRD-008", "PRD-009", "PRD-014", "PRD-016", "PRD-018", "PRD-019"]
updated: "2025-11-01"
purpose: "Clarifies SEA DSL release scope based on PRD coverage and highlights explicitly out-of-scope capabilities."
---

## PRD Section 5: Scope Boundary

```mermaid
C4Context
    title SEA DSL Scope vs External Responsibilities (PRD Section 5)
    %% Source: docs/specs/prd.md - Requirements portfolio

    AddElementTag("InScope", $bgColor="#e8f5e9", $borderColor="#2e7d32", $fontColor="#1b5e20")
    AddElementTag("OutScope", $bgColor="#ffe1e1", $borderColor="#c62828", $fontColor="#b71c1c")

    System_Boundary(scope, "SEA DSL Release v1", "InScope") {
        System(layering, "Layered Model Manager", "Implements PRD-001")
        System(primitives, "Primitive Registry", "Implements PRD-002 & PRD-004")
        System(validation, "Policy Validation Engine", "Implements PRD-003 & PRD-012")
        System(bindings, "Language Bindings Suite", "Python/TS/WASM (PRD-006/7/8/15)")
        System(interops, "Interoperability Toolkit", "ERP5 (PRD-009) & CALM (PRD-014)")
        System(ops, "Release & Benchmark Ops", "PRD-016 & PRD-018")
        System(docs, "Documentation Portal", "PRD-019")
    }

    System_Ext(uiBuilder, "Drag-and-drop UI Builder", "Not defined in PRD", "OutScope")
    System_Ext(identity, "Enterprise Identity/SSO", "Assumed external", "OutScope")
    System_Ext(erpExecution, "ERP Transaction Execution", "Remains in ERP5", "OutScope")
    System_Ext(dataWarehouse, "Enterprise Data Warehouse", "Consumes SEA outputs", "OutScope")

    Rel(bindings, dataWarehouse, "Exports validated datasets")
    Rel(interops, erpExecution, "Read-only migration", "No transactional control")
    Rel(uiBuilder, bindings, "%% WARNING: Insufficient detail on UI authoring (confirm if future roadmap)")
    Rel(identity, bindings, "Leverage host app authentication")
```

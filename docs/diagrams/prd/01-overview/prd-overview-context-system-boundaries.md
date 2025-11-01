---
prd_section: "1. Overview & Context"
diagram_type: "C4Context"
ears_requirements: ["PRD-001", "PRD-002", "PRD-003", "PRD-004", "PRD-005", "PRD-006", "PRD-007", "PRD-008", "PRD-009", "PRD-015"]
updated: "2025-11-01"
purpose: "Maps SEA DSL platform boundaries, primary personas, and external integrations derived from the PRD overview."
---

## PRD Section 1: Overview Context Diagram

The diagram highlights how personas interface with the SEA DSL platform and where key external systems (ERP5 UBM and FINOS CALM) connect, ensuring all interfaces cited in docs/specs/prd.md Section "Overview" and traceability summary are represented.

```mermaid
C4Context
    title SEA DSL Platform Context (PRD Section 1)
    %% Source: docs/specs/prd.md - Overview & Traceability Summary
    %% Personas sourced from target users and user stories

    AddElementTag("InScope", $bgColor="#f0f8ff", $borderColor="#0d47a1", $fontColor="#0d47a1")
    AddElementTag("External", $bgColor="#fff4e1", $borderColor="#ff8f00", $fontColor="#6d4c41")

    Person(businessAnalyst, "Business Analyst", "Models vocabulary, fact model, rules")
    Person(enterpriseArchitect, "Enterprise Architect", "Aligns SEA with enterprise architecture")
    Person(complianceOfficer, "Compliance Officer", "Defines policies and monitors violations")
    Person(platformEngineer, "Platform Engineer", "Operates Rust core & release tooling")
    Person(dataScientist, "Data Scientist", "Analyses resource flows via Python tooling")

    System_Ext(erp5, "ERP5 UBM", "Legacy domain models & migrations")
    System_Ext(calm, "FINOS CALM", "Architecture documentation ecosystem")
    System_Ext(ci, "CI/CD Pipeline", "Cross-language regression harness")
    System_Ext(analytics, "Analytics Stack", "Pandas, BI tools consuming SEA outputs")
    System_Ext(nextjs, "Next.js Applications", "Serverless + edge validation consumers")

    System_Boundary(sea, "SEA DSL Platform", "InScope") {
        System(seaCore, "Rust Core Engine", "Parsing, graph runtime, validation")
        Container(validation, "Policy Validation Engine", "Executes SBVR-aligned rules")
        Container(bindings, "Language Bindings", "Python (PyO3), TypeScript (N-API), WASM")
        Container(migration, "Interoperability Toolkit", "ERP5 mapping, CALM adapters")
    }

    Rel(businessAnalyst, bindings, "Define & validate models via Python UI")
    Rel(dataScientist, bindings, "Stream analytics-ready data", "Async generators")
    Rel(complianceOfficer, validation, "Author policies & review violations")
    Rel(enterpriseArchitect, migration, "Synchronise enterprise models", "Bidirectional mapping")
    Rel(platformEngineer, seaCore, "Operates releases & performance benchmarks")

    Rel(erp5, migration, "Import/export ERP5 UBM primitives", "PRD-009")
    Rel(migration, calm, "Round-trip CALM JSON", "PRD-014")
    Rel(bindings, ci, "Semantic consistency regression", "PRD-015")
    Rel(validation, analytics, "Expose violation datasets for BI")
    Rel(bindings, nextjs, "Deploy validation to edge runtimes", "PRD-007/PRD-008")
``` 

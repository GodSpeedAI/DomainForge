---
sds_section: "6. Data Design"
diagram_type: "Flowchart"
component_ids: ["CORE-API-RustCore", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer"]
implements_adrs: ["ADR-003", "ADR-005", "ADR-006"]
satisfies_requirements: ["REQ-004", "REQ-009", "REQ-012", "REQ-014"]
related_diagrams:
  - sds-data-schema-complete.md
  - sds-data-migration-strategy.md
  - ../03-architecture/sds-architecture-e2e-flow.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Visualises data flow from ingestion to validation and export."
---

## Data Pipeline Overview

```mermaid
graph LR
    %% Source: docs/specs/sds.md SDS-004..SDS-013
    %% Implements: ADR-003, ADR-005, ADR-006
    %% Satisfies: REQ-004, REQ-009, REQ-012, REQ-014
    %% Components: CORE-API-RustCore, CORE-DB-GraphStore, VAL-SVC-PolicyEvaluator, MIG-ADPT-ERP5Adapter, MIG-ADPT-CALMSerializer

    ERP5[ERP5 Data]
    Migration["MIG-ADPT-ERP5Adapter"]
    Core["CORE-API-RustCore"]
    Graph["CORE-DB-GraphStore"]
    Policy["VAL-SVC-PolicyEvaluator"]
    Violations["Violation Stream"]
    CALM["MIG-ADPT-CALMSerializer"]
    CALMOut["CALM JSON"]
    Export["External Consumers"]

    ERP5 -->|Node/Movement payloads| Migration
    Migration -->|SEA primitives| Core
    Core -->|persist| Graph
    Core -->|trigger validation| Policy
    Policy -->|stream| Violations
    Policy -->|summary| Core
    Violations -->|BI analytics| Export
    Core -->|serialize| CALM
    CALM --> CALMOut
    CALMOut --> Export
```

### Design Rationale
- Ensures imported data passes through validation before export.
- Exposes violation stream for analytics/regulatory reporting.

### Related Components
- Migration details by ERP5/CALM workflows in [sds-data-migration-strategy](sds-data-migration-strategy.md).

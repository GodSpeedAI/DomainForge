---
sds_section: "6. Data Design"
diagram_type: "Flowchart"
component_ids: ["MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer", "CORE-API-RustCore", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-005", "ADR-006", "ADR-007"]
satisfies_requirements: ["REQ-009", "REQ-014", "REQ-019"]
related_diagrams:
  - ../02-context/sds-context-external-integrations.md
  - sds-data-pipeline-events.md
  - ../08-deployment/sds-deployment-cicd-pipeline.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Explains ERP5 migration workflow, CALM export fallback, and documentation hooks."
---

## Migration Strategy

```mermaid
graph TD
    %% Source: docs/specs/sds.md SDS-012, SDS-013
    %% Implements: ADR-005, ADR-006, ADR-007
    %% Satisfies: REQ-009, REQ-014, REQ-019
    %% Components: MIG-ADPT-ERP5Adapter, MIG-ADPT-CALMSerializer, CORE-API-RustCore, DOC-PIPE-DocGenerator

    Start([Start ERP5 Migration])
    Extract[Extract ERP5 Nodes/Movements]
    Normalize[Normalize to SEA primitives]
    Import[Import via CORE-API-RustCore]
    Validate[Run policy validation]
    Violations{Violations?}
    ExportCALM[Export CALM snapshot]
    PublishDocs[Update migration guide]
    Finish([Migration Complete])

    Start --> Extract
    Extract --> Normalize
    Normalize --> Import
    Import --> Validate
    Validate --> Violations
    Violations -- Yes --> PublishDocs
    PublishDocs --> Finish
    Violations -- No --> ExportCALM
    ExportCALM --> PublishDocs
    PublishDocs --> Finish
```

### Design Rationale
- Ensures validation occurs before exporting CALM artifacts.
- Documentation update step ensures migration guides stay current (REQ-019).

### Related Components
- CI automation for migration regression runs described in [sds-deployment-cicd-pipeline](../08-deployment/sds-deployment-cicd-pipeline.md).

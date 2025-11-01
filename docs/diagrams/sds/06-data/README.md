---
sds_section: "6. Data Design"
diagram_type: "Documentation"
component_ids: ["CORE-DB-GraphStore", "MODEL-COMP-EntityPrimitive", "MODEL-COMP-ResourcePrimitive", "MODEL-COMP-FlowPrimitive", "MODEL-COMP-InstancePrimitive", "MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer"]
implements_adrs: ["ADR-003", "ADR-005", "ADR-006"]
satisfies_requirements: ["REQ-002", "REQ-004", "REQ-009", "REQ-011", "REQ-012", "REQ-014"]
related_diagrams:
  - sds-data-schema-complete.md
  - sds-data-pipeline-events.md
  - sds-data-consistency-model.md
  - sds-data-migration-strategy.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Introduces data schema, pipeline, consistency, and migration diagrams."
---

## Data Assets

- `sds-data-schema-complete.md`: ER diagram for primitives and indexes.
- `sds-data-pipeline-events.md`: Flowchart of data ingestion/export flows.
- `sds-data-consistency-model.md`: State diagram modelling flow lifecycle and consistency guarantees.
- `sds-data-migration-strategy.md`: Flowchart for ERP5/CALM migration paths and validation.

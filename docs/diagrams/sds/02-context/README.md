---
sds_section: "2. System Overview & Context"
diagram_type: "Documentation"
component_ids: ["CORE-API-RustCore", "MODEL-COMP-EntityPrimitive", "BIND-API-PyO3", "BIND-API-TypeScript", "MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-005", "ADR-006", "ADR-007"]
satisfies_requirements: ["REQ-001", "REQ-002", "REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-009", "REQ-014", "REQ-015"]
related_diagrams:
  - sds-context-system-boundaries.md
  - sds-context-external-integrations.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Summarises provided context diagrams detailing SEA DSL system boundaries and integration points."
---

## Context Assets

- `sds-context-system-boundaries.md`: C4Context diagram mapping SEA DSL core, personas, and primary components to external systems.
- `sds-context-external-integrations.md`: Flowchart highlighting integration patterns (ERP5, CALM, package registries) and requirements they satisfy.

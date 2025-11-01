---
sds_section: "3. Architectural Design"
diagram_type: "Documentation"
component_ids: ["CORE-API-RustCore", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "PARSE-SVC-GrammarParser", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-003", "ADR-004", "ADR-005", "ADR-007"]
satisfies_requirements: ["REQ-001", "REQ-002", "REQ-003", "REQ-004", "REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-012", "REQ-013", "REQ-015"]
related_diagrams:
  - sds-architecture-container-overview.md
  - sds-architecture-service-interactions.md
  - sds-architecture-e2e-flow.md
  - sds-architecture-error-handling.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Overviews container-level and end-to-end architectural diagrams for the SEA DSL platform."
---

## Architecture Assets

- `sds-architecture-container-overview.md`: C4Container showing services, bindings, and data stores.
- `sds-architecture-service-interactions.md`: Flowchart of synchronous vs asynchronous collaboration between core modules.
- `sds-architecture-e2e-flow.md`: Sequence capturing user-to-database round trip across components.
- `sds-architecture-error-handling.md`: Flowchart detailing error propagation and mitigation strategies.

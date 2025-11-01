---
sds_section: "9. Testing & QA"
diagram_type: "Documentation"
component_ids: ["VAL-SVC-PolicyEvaluator", "CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "OPS-PIPE-ReleaseAutomation"]
implements_adrs: ["ADR-002", "ADR-004", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-003", "REQ-005", "REQ-006", "REQ-007", "REQ-012", "REQ-013", "REQ-015", "REQ-017", "REQ-018"]
related_diagrams:
  - sds-testing-strategy.md
  - sds-testing-integration-scenarios.md
updated: "2025-11-01"
reviewed_by: "QA Team"
purpose: "Outlines QA diagrams covering strategy and integration scenarios."
---

## Testing Assets

- `sds-testing-strategy.md`: Flowchart of QA layers and responsibilities.
- `sds-testing-integration-scenarios.md`: Sequence diagram for integration test coverage across bindings and migration pipelines.

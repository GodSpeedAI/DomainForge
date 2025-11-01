---
sds_section: "4. Component Design"
diagram_type: "Documentation"
component_ids: ["CORE-SVC-NamespaceResolver", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "BIND-API-PyO3", "BIND-API-TypeScript"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-003", "ADR-004", "ADR-007"]
satisfies_requirements: ["REQ-001", "REQ-003", "REQ-004", "REQ-005", "REQ-006", "REQ-007", "REQ-010", "REQ-012", "REQ-015", "REQ-017"]
related_diagrams:
  - sds-component-namespace-resolver.md
  - sds-component-graph-runtime.md
  - sds-component-policy-engine.md
  - sds-component-language-parity.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Introduces component-level drill-down diagrams and interaction sequences."
---

## Component Assets

- `sds-component-namespace-resolver.md`: C4Component + rationale for namespace resolution system.
- `sds-component-graph-runtime.md`: Component view of graph storage and indexes.
- `sds-component-policy-engine.md`: Component diagram plus sequence for policy evaluation internals.
- `sds-component-language-parity.md`: Cross-language parity harness component map ensuring REQ-015 coverage.

---
sds_section: "7. Security & Compliance"
diagram_type: "Documentation"
component_ids: ["CORE-API-RustCore", "VAL-SVC-PolicyEvaluator", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "OPS-PIPE-ReleaseAutomation"]
implements_adrs: ["ADR-002", "ADR-004", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-003", "REQ-005", "REQ-012", "REQ-013", "REQ-015", "REQ-017", "REQ-018"]
related_diagrams:
  - sds-security-boundaries.md
  - sds-security-auth-flow.md
  - sds-security-threat-model.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Overview of security diagrams covering trust boundaries, authentication flows, and threat modelling."
---

## Security Assets

- `sds-security-boundaries.md`: Context diagram of trust zones and data classification.
- `sds-security-auth-flow.md`: Flowchart of release credential handling and validation access.
- `sds-security-threat-model.md`: Sequence diagram enumerating primary threat scenarios and mitigations.

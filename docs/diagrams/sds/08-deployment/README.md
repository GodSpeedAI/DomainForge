---
sds_section: "8. Deployment & Operations"
diagram_type: "Documentation"
component_ids: ["CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "OPS-PIPE-ReleaseAutomation", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-002", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-015", "REQ-016", "REQ-018", "REQ-019"]
related_diagrams:
  - sds-deployment-topology.md
  - sds-deployment-cicd-pipeline.md
  - sds-deployment-scaling-strategy.md
  - sds-deployment-observability.md
updated: "2025-11-01"
reviewed_by: "DevOps Team"
purpose: "Describes deployment topology, CI/CD pipeline, scaling, and observability."
---

## Deployment Assets

- `sds-deployment-topology.md`: C4Deployment of build/test/publish infrastructure and runtime environments.
- `sds-deployment-cicd-pipeline.md`: Flowchart of lockstep release workflow.
- `sds-deployment-scaling-strategy.md`: Flowchart summarising scaling/failover strategy.
- `sds-deployment-observability.md`: C4Container view for observability tooling and metrics.

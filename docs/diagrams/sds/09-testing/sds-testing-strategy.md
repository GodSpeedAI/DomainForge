---
sds_section: "9. Testing & QA"
diagram_type: "Flowchart"
component_ids: ["VAL-SVC-PolicyEvaluator", "CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "OPS-PIPE-ReleaseAutomation"]
implements_adrs: ["ADR-002", "ADR-004", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-003", "REQ-005", "REQ-006", "REQ-007", "REQ-012", "REQ-013", "REQ-015", "REQ-017", "REQ-018"]
related_diagrams:
  - sds-testing-integration-scenarios.md
  - ../08-deployment/sds-deployment-cicd-pipeline.md
  - ../00-traceability/sds-diagram-validation-report.md
updated: "2025-11-01"
reviewed_by: "QA Team"
purpose: "Summarises layered testing strategy and responsibilities."
---

## Testing Strategy Flow

```mermaid
graph TD
    %% Source: docs/specs/sds.md SDS-006..SDS-015
    %% Implements: ADR-002, ADR-004, ADR-007, ADR-008
    %% Satisfies: REQ-003, REQ-005, REQ-006, REQ-007, REQ-012, REQ-013, REQ-015, REQ-017, REQ-018
    %% Components: VAL-SVC-PolicyEvaluator, CORE-API-RustCore, BIND-API-PyO3, BIND-API-TypeScript, OPS-PIPE-ReleaseAutomation

    Unit["Unit Tests (Rust/Python/TS)"]
    Integration["Integration Tests (Cross-component)"]
    Parity["Parity Harness"]
    Streaming["Streaming Validation Benchmarks"]
    Migration["ERP5/CALM Regression Suite"]
    Perf["Performance Benchmarks"]
    Docs["Documentation Example Tests"]
    ReleaseGate["Release Gate"]

    Unit --> Integration
    Integration --> Parity
    Parity --> Streaming
    Streaming --> Migration
    Migration --> Perf
    Perf --> Docs
    Docs --> ReleaseGate
```

### Design Rationale
- Emphasises parity harness gating prior to streaming/performance tests.
- Documentation examples executed to prevent drift (REQ-019 via docs pipeline).

### Related Components
- Integration scenario specifics in [sds-testing-integration-scenarios](sds-testing-integration-scenarios.md).

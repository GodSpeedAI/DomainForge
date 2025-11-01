---
sds_section: "7. Security & Compliance"
diagram_type: "Flowchart"
component_ids: ["BIND-API-PyO3", "BIND-API-TypeScript", "CORE-API-RustCore", "OPS-PIPE-ReleaseAutomation"]
implements_adrs: ["ADR-002", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-005", "REQ-006", "REQ-007", "REQ-012", "REQ-017", "REQ-018"]
related_diagrams:
  - sds-security-boundaries.md
  - ../05-interfaces/sds-api-ffi-error-handling.md
  - ../08-deployment/sds-deployment-cicd-pipeline.md
updated: "2025-11-01"
reviewed_by: "Security Team"
purpose: "Shows authentication and authorisation flow for CLI/API usage and release automation."
---

## Authentication & Authorisation Flow

```mermaid
graph TD
    %% Source: docs/specs/sds.md SDS-008..SDS-010, SDS-014
    %% Implements: ADR-002, ADR-007, ADR-008
    %% Satisfies: REQ-005, REQ-006, REQ-007, REQ-012, REQ-017, REQ-018
    %% Components: BIND-API-PyO3, BIND-API-TypeScript, CORE-API-RustCore, OPS-PIPE-ReleaseAutomation

    UserToken[Developer Access Token]
    PythonCLI["BIND-API-PyO3 CLI"]
    TSCLI["BIND-API-TypeScript CLI"]
    AuthGateway["FFI Auth Guard"]
    Core["CORE-API-RustCore"]
    Audit["Audit Logger"]
    CI["OPS-PIPE-ReleaseAutomation"]
    Secrets["OIDC / Secret Store"]

    UserToken --> PythonCLI
    UserToken --> TSCLI
    PythonCLI --> AuthGateway
    TSCLI --> AuthGateway
    AuthGateway -->|Validate token| Secrets
    Secrets -->|JWT claims| AuthGateway
    AuthGateway -->|Scoped context| Core
    Core -->|Policy evaluation| Audit
    Audit -->|Append logs| CI

    CI -->|assume-role| Secrets
    CI -->|publish packages| Core
    CI -->|rotate tokens| Secrets
```

### Design Rationale
- FFI guard ensures tokens validated before core invocation.
- CI release pipeline uses OIDC to avoid long-lived credentials.

### Related Components
- CI/CD specifics described in [sds-deployment-cicd-pipeline](../08-deployment/sds-deployment-cicd-pipeline.md).

---
sds_section: "7. Security & Compliance"
diagram_type: "Sequence"
component_ids: ["BIND-API-TypeScript", "CORE-API-RustCore", "VAL-SVC-PolicyEvaluator", "OPS-PIPE-ReleaseAutomation"]
implements_adrs: ["ADR-002", "ADR-004", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-003", "REQ-005", "REQ-007", "REQ-012", "REQ-013", "REQ-015", "REQ-017", "REQ-018"]
related_diagrams:
  - sds-security-boundaries.md
  - ../03-architecture/sds-architecture-error-handling.md
  - ../09-testing/sds-testing-integration-scenarios.md
updated: "2025-11-01"
reviewed_by: "Security Team"
purpose: "Sequence-based threat model highlighting key attack vectors and mitigations."
---

## Threat Scenario Sequence

```mermaid
sequenceDiagram
    %% Source: docs/specs/sds.md SDS-006..SDS-010, SDS-014
    %% Implements: ADR-002, ADR-004, ADR-007, ADR-008
    %% Satisfies: REQ-003, REQ-005, REQ-007, REQ-012, REQ-013, REQ-015, REQ-017, REQ-018
    %% Components: BIND-API-TypeScript, CORE-API-RustCore, VAL-SVC-PolicyEvaluator, OPS-PIPE-ReleaseAutomation

    participant Attacker as Untrusted TS Client
    participant TS as BIND-API-TypeScript
    participant Core as CORE-API-RustCore
    participant Policy as VAL-SVC-PolicyEvaluator
    participant CI as OPS-PIPE-ReleaseAutomation

    Attacker->>TS: define_flow(payload with oversized attributes)
    TS->>Core: define_flow(...)
    Core->>Core: validate payload size (Mitigation: bounds check)
    alt Payload too large
        Core-->>TS: Err(Error::PayloadTooLarge)
        TS-->>Attacker: throw Error 413
    else Acceptable
        Core->>Policy: validate()
        Policy->>Policy: enforce recursion depth (Mitigation: depth limit)
        Policy-->>Core: Violations
    end

    Attacker->>CI: attempt publish via forged token
    CI->>CI: OIDC attestation (Mitigation: short-lived tokens)
    alt Invalid attestation
        CI-->>Attacker: 401 Unauthorized
    else Valid release
        CI->>Core: run integration suite
        Core-->>CI: success
    end
```

### Design Rationale
- Captures input validation (size limits) and evaluation recursion guard.
- Shows release pipeline authentication mitigations (OIDC).

### Related Components
- Detailed error mapping in [sds-architecture-error-handling](../03-architecture/sds-architecture-error-handling.md).

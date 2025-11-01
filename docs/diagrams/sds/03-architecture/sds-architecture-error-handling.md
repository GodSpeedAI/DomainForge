---
sds_section: "3. Architectural Design"
diagram_type: "Flowchart"
component_ids: ["CORE-API-RustCore", "CORE-SVC-NamespaceResolver", "VAL-SVC-PolicyEvaluator", "BIND-API-TypeScript"]
implements_adrs: ["ADR-002", "ADR-004", "ADR-007"]
satisfies_requirements: ["REQ-003", "REQ-005", "REQ-007", "REQ-012", "REQ-013", "REQ-015", "REQ-017"]
related_diagrams:
  - sds-architecture-e2e-flow.md
  - ../05-interfaces/sds-api-ffi-error-handling.md
  - ../07-security/sds-security-threat-model.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Describes error propagation strategy across core API, bindings, and validation engine, ensuring diagnostics and resilience."
---

## Error Handling Strategy

Outlines detection, classification, and propagation of modelling and validation errors, emphasising REQ-017 diagnostics and streaming compatibility.

```mermaid
graph TD
    %% Source: docs/specs/sds.md Sections SDS-001, SDS-006, SDS-008, SDS-009, SDS-010
    %% Implements: ADR-002, ADR-004, ADR-007
    %% Satisfies: REQ-003, REQ-005, REQ-007, REQ-012, REQ-013, REQ-015, REQ-017
    %% Components: CORE-API-RustCore, CORE-SVC-NamespaceResolver, VAL-SVC-PolicyEvaluator, BIND-API-TypeScript

    Start((Error Detected))
    NamespaceErr{Namespace?}
    CoreErr{Core API Result?}
    PolicyErr{Policy Severity?}
    BindingLayer["BIND-API-* Adapter"]
    Log["Structured Diagnostics"]
    Stream["Streaming Channel"]
    Retry["Client Retry / Remediation"]

    Start --> NamespaceErr
    NamespaceErr -- Missing -->|Error::UndefinedNamespace| CoreErr
    NamespaceErr -- Resolved --> PolicyErr

    PolicyErr -- Error -->|Violation(severity=error)| CoreErr
    PolicyErr -- Warning --> Stream
    PolicyErr -- Info --> Stream

    CoreErr -- Err --> BindingLayer
    CoreErr -- Ok --> Retry

    BindingLayer -->|Map to native Error| Retry
    Stream -->|push events| Retry
    BindingLayer --> Log
    Stream --> Log
```

### Design Rationale
- Differentiates namespace resolution errors from policy violations.
- Confirms parity in binding-layer error mapping (TypeScript/Python).

### Related Components
- Detailed binding error mapping shown in [sds-api-ffi-error-handling](../05-interfaces/sds-api-ffi-error-handling.md).
- Threat modelling for malicious inputs captured in [sds-security-threat-model](../07-security/sds-security-threat-model.md).

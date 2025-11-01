---
sds_section: "7. Security & Compliance"
diagram_type: "C4Context"
component_ids: ["CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "OPS-PIPE-ReleaseAutomation", "MIG-ADPT-ERP5Adapter"]
implements_adrs: ["ADR-002", "ADR-004", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-003", "REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-012", "REQ-013", "REQ-015", "REQ-017", "REQ-018"]
related_diagrams:
  - ../02-context/sds-context-system-boundaries.md
  - ../08-deployment/sds-deployment-topology.md
  - sds-security-threat-model.md
updated: "2025-11-01"
reviewed_by: "Security Team"
purpose: "Defines trust boundaries for SEA DSL runtime, bindings, and release infrastructure."
---

## Trust Boundary Diagram

```mermaid
C4Context
    %% Source: docs/specs/sds.md SDS-006..SDS-011, SDS-014
    %% Implements: ADR-002, ADR-004, ADR-007, ADR-008
    %% Satisfies: REQ-003, REQ-005, REQ-006, REQ-007, REQ-008, REQ-012, REQ-013, REQ-015, REQ-017, REQ-018
    %% Components: CORE-API-RustCore, BIND-API-PyO3, BIND-API-TypeScript, BIND-API-WebAssembly, OPS-PIPE-ReleaseAutomation, MIG-ADPT-ERP5Adapter

    AddBoundary("TrustedCore", $bgColor="#ecfdf5", $borderColor="#047857", $fontColor="#065f46")
    AddBoundary("FFIClients", $bgColor="#f0f9ff", $borderColor="#0ea5e9", $fontColor="#075985")
    AddBoundary("CIInfra", $bgColor="#fff7ed", $borderColor="#f97316", $fontColor="#7c2d12")

    Boundary(TrustedCore, "Trusted Core (Rust sandbox)") {
        System(core, "CORE-API-RustCore", "Rust crate", "Executes validated operations")
        System(policy, "VAL-SVC-PolicyEvaluator", "Rust module", "Runs policies with bounded recursion")
    }

    Boundary(FFIClients, "FFI Clients") {
        System(python, "BIND-API-PyO3", "Python extension", "Untrusted inputs, memory-safe wrappers")
        System(node, "BIND-API-TypeScript", "Node addon", "Untrusted runtime, uses napi-rs checks")
        System(wasm, "BIND-API-WebAssembly", "WASM module", "Runs in browser/edge sandbox")
    }

    Boundary(CIInfra, "Release & Migration Infrastructure") {
        System(ci, "OPS-PIPE-ReleaseAutomation", "GitHub Actions", "Secrets-scoped, publishes artifacts")
        System(migration, "MIG-ADPT-ERP5Adapter", "Migration jobs", "Access to ERP5 credentials")
    }

    Person(user, "Model Author", "Interacts via CLI/API")
    Person(devops, "Release Engineer", "Maintains CI secrets")

    Rel(user, python, "Define models", "TLS + API keys")
    Rel(user, node, "Validate models", "TLS + Token")
    Rel(user, wasm, "Browser validation", "Secure context")
    Rel(python, core, "FFI calls", "PyO3 boundary checks")
    Rel(node, core, "FFI calls", "napi-rs type checks")
    Rel(wasm, core, "WASM interface", "wit-bindgen interface types")
    Rel(core, policy, "In-process call", "Rust safe API")
    Rel(migration, core, "ERP5 migrations", "OAuth2 tokens")
    Rel(devops, ci, "Manage secrets", "OIDC")
```

### Design Rationale
- Distinguishes trusted Rust core from untrusted language bindings for threat analysis.
- Highlights credentials stored within CI boundary.

### Related Components
- Deployment specifics for compartments in [sds-deployment-topology](../08-deployment/sds-deployment-topology.md).

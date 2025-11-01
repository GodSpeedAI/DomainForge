---
sds_section: "5. Interface & API Design"
diagram_type: "Sequence"
component_ids: ["CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly"]
implements_adrs: ["ADR-002", "ADR-007"]
satisfies_requirements: ["REQ-006", "REQ-007", "REQ-008", "REQ-015", "REQ-017"]
related_diagrams:
  - ../04-components/sds-component-language-parity.md
  - ../03-architecture/sds-architecture-error-handling.md
  - ../07-security/sds-security-threat-model.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Shows how Rust errors and violations map to Python, TypeScript, and WebAssembly error types."
---

## API Sequence: FFI Error Propagation

```mermaid
sequenceDiagram
    %% Source: docs/specs/sds.md SDS-008..SDS-011
    %% Implements: ADR-002, ADR-007
    %% Satisfies: REQ-006, REQ-007, REQ-008, REQ-015, REQ-017
    %% Components: CORE-API-RustCore, BIND-API-PyO3, BIND-API-TypeScript, BIND-API-WebAssembly

    participant Core as CORE-API-RustCore
    participant Py as BIND-API-PyO3
    participant Ts as BIND-API-TypeScript
    participant Wasm as BIND-API-WebAssembly

    Core->>Core: execute operation
    alt Result::Err(Error::CardinalityViolation)
        Core-->>Py: Err(CardinalityViolation)
        Py-->>Py: map_err(PyValueError)
        Py-->>Client: raise ValueError(message, context)

        Core-->>Ts: Err(CardinalityViolation)
        Ts-->>Ts: map_err(NapiError)
        Ts-->>Client: throw Error(message)

        Core-->>Wasm: Err(CardinalityViolation)
        Wasm-->>Client: reject Promise(Error)
    else Result::Err(Error::UndefinedNamespace)
        Core-->>Py: Err(UndefinedNamespace)
        Py-->>Client: raise NamespaceError
        Core-->>Ts: Err(UndefinedNamespace)
        Ts-->>Client: throw NamespaceError
        Core-->>Wasm: Err(UndefinedNamespace)
        Wasm-->>Client: reject Promise(NamespaceError)
    end

    rect rgb(240, 253, 244)
        Note over Py,Ts: Includes stack traces + location metadata (REQ-017)
    end
```

### Design Rationale
- Normalises diagnostics to host language idioms while preserving context.
- Ensures parity harness can compare errors across bindings (REQ-015).

### Related Components
- Parity harness verification described in [sds-component-language-parity](../04-components/sds-component-language-parity.md).

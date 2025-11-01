---
sds_section: "8. Deployment & Operations"
diagram_type: "Flowchart"
component_ids: ["OPS-PIPE-ReleaseAutomation", "CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-002", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-015", "REQ-016", "REQ-018", "REQ-019"]
related_diagrams:
  - sds-deployment-topology.md
  - sds-component-language-parity.md
  - sds-testing-strategy.md
updated: "2025-11-01"
reviewed_by: "DevOps Team"
purpose: "Displays lockstep release pipeline stages, gating checks, and artifact publication."
---

## CI/CD Pipeline Flow

```mermaid
graph TD
    %% Source: docs/specs/sds.md SDS-014
    %% Implements: ADR-002, ADR-007, ADR-008
    %% Satisfies: REQ-005, REQ-006, REQ-007, REQ-008, REQ-015, REQ-016, REQ-018, REQ-019
    %% Components: OPS-PIPE-ReleaseAutomation, CORE-API-RustCore, BIND-API-PyO3, BIND-API-TypeScript, BIND-API-WebAssembly, DOC-PIPE-DocGenerator

    Start([Tag vX.Y.Z])
    Matrix["Matrix Tests (OS × Arch)"]
    Parity["Cross-language Parity Harness"]
    Bench["Performance Benchmarks"]
    Docs["Generate Docs"]
    Approve{"Release gate?"}
    PublishCrate["Publish to crates.io"]
    PublishPyPI["Publish to PyPI"]
    PublishNPM["Publish to npm"]
    PublishWASM["Upload WASM bundle"]
    ReleaseNotes["Publish Release Notes + Docs"]
    End([Release Complete])

    Start --> Matrix
    Matrix --> Parity
    Parity --> Bench
    Bench --> Docs
    Docs --> Approve
    Approve -- Pass --> PublishCrate
    Approve -- Pass --> PublishPyPI
    Approve -- Pass --> PublishNPM
    Approve -- Pass --> PublishWASM
    PublishCrate --> ReleaseNotes
    PublishPyPI --> ReleaseNotes
    PublishNPM --> ReleaseNotes
    PublishWASM --> ReleaseNotes
    ReleaseNotes --> End

    Approve -- Fail --> End
```

### Design Rationale
- Parity harness gating ensures REQ-015 compliance before releasing.
- Docs generation step enforces REQ-019 documentation update.

### Related Components
- Performance metrics tracked in [sds-deployment-scaling-strategy](sds-deployment-scaling-strategy.md).

---
sds_section: "2. System Overview & Context"
diagram_type: "Flowchart"
component_ids: ["MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer", "OPS-PIPE-ReleaseAutomation", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-002", "ADR-005", "ADR-006", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-006", "REQ-007", "REQ-008", "REQ-009", "REQ-014", "REQ-016", "REQ-019"]
related_diagrams:
  - sds-context-system-boundaries.md
  - ../06-data/sds-data-migration-strategy.md
  - ../08-deployment/sds-deployment-cicd-pipeline.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Details integration paths between SEA DSL and external ecosystems, noting ADR constraints and requirement coverage."
---

## External Integration Flow

Illustrates how migration adapters, documentation tooling, and release automation interact with ERP5, CALM, and package registries, ensuring ADR-006 interoperability and ADR-008 release discipline.

```mermaid
graph LR
    %% Source: docs/specs/sds.md Sections SDS-012, SDS-013, SDS-014, SDS-015
    %% Implements: ADR-002, ADR-005, ADR-006, ADR-007, ADR-008
    %% Satisfies: REQ-006, REQ-007, REQ-008, REQ-009, REQ-014, REQ-016, REQ-019
    %% Components: MIG-ADPT-ERP5Adapter, MIG-ADPT-CALMSerializer, OPS-PIPE-ReleaseAutomation, DOC-PIPE-DocGenerator

    subgraph Migration["Migration Hub"]
        ERP5Adapter["MIG-ADPT-ERP5Adapter"]
        CALMSerializer["MIG-ADPT-CALMSerializer"]
    end

    ERP5Adapter -->|Imports nodes, movements| ERP5((ERP5 UBM))
    ERP5Adapter -->|Exports SEA models| ERP5
    CALMSerializer -->|Exports CALM JSON| CALM((FINOS CALM))
    CALM -->|Feeds architecture repos| DocsPortal[(Architecture Portal)]

    subgraph ReleaseOps["OPS-PIPE-ReleaseAutomation"]
        CI["CI/CD Workflow"]
        Bench["Benchmark Suite"]
        Publish["Publish Packages"]
    end

    CI --> ERP5Adapter
    CI --> CALMSerializer
    CI --> DocsGen["DOC-PIPE-DocGenerator"]
    DocsGen --> DocsPortal

    Publish --> PyPI[(PyPI)]
    Publish --> npm[(npm)]
    Publish --> Crates[(crates.io)]

    Bench --> CI
    Bench --> Publish

    DocsGen --> DevPortal[(SEA Docs Portal)]
```

### Design Rationale
- **Migration Hub**: Splits ERP5 and CALM adapters with distinct flows while sharing CI triggers.
- **ReleaseOps**: Ensures lockstep releases run benchmarks before publishing, satisfying ADR-008.

### Related Components
- Data mapping logic detailed in [sds-data-migration-strategy](../06-data/sds-data-migration-strategy.md).
- CI job structure expanded in [sds-deployment-cicd-pipeline](../08-deployment/sds-deployment-cicd-pipeline.md).

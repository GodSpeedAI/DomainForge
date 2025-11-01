---
sds_section: "1. Introduction"
diagram_type: "Flowchart"
component_ids: ["DOC-PIPE-DocGenerator", "CORE-API-RustCore", "OPS-PIPE-ReleaseAutomation"]
implements_adrs: ["ADR-002", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-005", "REQ-016", "REQ-019"]
related_diagrams:
  - ../00-traceability/sds-traceability-req-mapping.md
  - ../03-architecture/sds-architecture-container-overview.md
  - ../08-deployment/sds-deployment-cicd-pipeline.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Shows documentation flow across PRD, ADR, and SDS assets to enforce traceability and onboarding guidance."
---

## Introduction: Documentation Relationship Map

Establishes how product strategy (PRD), architectural decisions (ADR), and technical implementation (SDS) feed documentation pipelines and release governance. Ensures documentation automation (REQ-019) remains grounded in technical reality (ADR-002, ADR-008).

```mermaid
graph TD
    %% Source: docs/specs/sds.md - System Overview
    %% Implements: ADR-002, ADR-007, ADR-008
    %% Satisfies: REQ-005, REQ-016, REQ-019
    %% Components: DOC-PIPE-DocGenerator, CORE-API-RustCore, OPS-PIPE-ReleaseAutomation

    PRD[PRD Requirements\n(PRD-001..PRD-019)]
    ADR[ADR Ledger\n(ADR-001..ADR-008)]
    SDS[SDS Components\n(SDS-001..SDS-015)]

    subgraph DocumentationOps["DOC-PIPE-DocGenerator"]
        guide[Guides & References]
        apiDocs[API Docs (cargo doc, Sphinx, TypeDoc)]
    end

    subgraph ReleaseOps["OPS-PIPE-ReleaseAutomation"]
        cicd[CI/CD Pipelines]
        parity[Cross-Language Parity Reports]
    end

    PRD -->|Defines scope & KPIs| ADR
    ADR -->|Constrains design choices| SDS
    SDS -->|Generates Markdown diagrams| guide
    SDS -->|Generates API comments| apiDocs
    guide -->|Feeds onboarding| parity
    apiDocs -->|Supports| parity
    cicd -->|Publishes docs| guide
    cicd -->|Publishes packages| parity
    PRD -->|Traceability inputs| cicd
    ADR -->|Version gates| cicd
```

### Design Rationale
- **Traceability Flow**: Aligns PRD → ADR → SDS pipeline, surfacing export into docs and releases.
- **Automation Hooks**: Highlights where documentation automation taps into CI/CD to avoid drift.

### Related Components
- `DOC-PIPE-DocGenerator` consumes metadata from SDS diagrams to regenerate platform guides.
- `OPS-PIPE-ReleaseAutomation` triggers rebuild of docs during lockstep releases.

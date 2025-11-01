---
sds_section: "2. System Overview & Context"
diagram_type: "C4Context"
component_ids: ["CORE-API-RustCore", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "PARSE-SVC-GrammarParser", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer", "OPS-PIPE-ReleaseAutomation"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-003", "ADR-004", "ADR-005", "ADR-006", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-001", "REQ-002", "REQ-003", "REQ-004", "REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-009", "REQ-012", "REQ-014", "REQ-015", "REQ-016"]
related_diagrams:
  - ../03-architecture/sds-architecture-container-overview.md
  - ../03-architecture/sds-architecture-e2e-flow.md
  - ../07-security/sds-security-boundaries.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Depicts SEA DSL system boundaries, personas, and external systems referenced in SDS and PRD."
---

## System Context Diagram

Establishes SEA DSL boundaries, aligning with ADR-001/ADR-002 layering and referencing integrations mandated by PRD requirements. Ensures all external actors mentioned in SDS (ERP5, CALM, package registries) appear.

```mermaid
C4Context
    %% Source: docs/specs/sds.md - System Overview
    %% Implements: ADR-001, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008
    %% Satisfies: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-012, REQ-014, REQ-015, REQ-016
    %% Components: CORE-API-RustCore, CORE-DB-GraphStore, VAL-SVC-PolicyEvaluator, PARSE-SVC-GrammarParser, BIND-API-PyO3, BIND-API-TypeScript, BIND-API-WebAssembly, MIG-ADPT-ERP5Adapter, MIG-ADPT-CALMSerializer, OPS-PIPE-ReleaseAutomation

    AddElementTag("Persona", $bgColor="#f7fbff", $borderColor="#1d4ed8", $fontColor="#1d4ed8")
    AddElementTag("External", $bgColor="#fff7ed", $borderColor="#f97316", $fontColor="#9a3412")
    AddElementTag("Core", $bgColor="#ecfdf5", $borderColor="#047857", $fontColor="#065f46")

    Person(businessAnalyst, "Business Analyst", "Models vocabulary, fact model, and policies", $tags="Persona")
    Person(engineer, "Platform Engineer", "Maintains Rust core and release automation", $tags="Persona")
    Person(compliance, "Compliance Officer", "Authorises policies and inspects violations", $tags="Persona")
    Person(devRel, "Developer Advocate", "Publishes docs and SDK samples", $tags="Persona")

    System_Ext(erp5, "ERP5 UBM", "Legacy enterprise models", $tags="External")
    System_Ext(calm, "FINOS CALM", "Architecture-as-code repository", $tags="External")
    System_Ext(pypi, "PyPI Registry", "Python package distribution", $tags="External")
    System_Ext(npmjs, "npm Registry", "TypeScript package distribution", $tags="External")
    System_Ext(crates, "crates.io", "Rust crate registry", $tags="External")
    System_Ext(ci, "CI/CD Infrastructure", "GitHub Actions, Benchmark runners", $tags="External")
    System_Ext(edge, "Edge Runtimes", "Browser, Deno, Cloudflare Workers", $tags="External")

    System_Boundary(sea, "SEA DSL Platform", "Core") {
        System(core, "CORE-API-RustCore", "Rust API surface for modelling", $tags="Core")
        Container(graph, "CORE-DB-GraphStore", "Graph storage and indexes", $tags="Core")
        Container(policy, "VAL-SVC-PolicyEvaluator", "Policy evaluation engine", $tags="Core")
        Container(parser, "PARSE-SVC-GrammarParser", "SBVR grammar parser", $tags="Core")
        Container(python, "BIND-API-PyO3", "Python bindings", $tags="Core")
        Container(ts, "BIND-API-TypeScript", "TypeScript bindings", $tags="Core")
        Container(wasm, "BIND-API-WebAssembly", "WASM bindings", $tags="Core")
        Container(adapter, "MIG-ADPT-ERP5Adapter", "ERP5 migration tooling", $tags="Core")
        Container(calmAdapter, "MIG-ADPT-CALMSerializer", "CALM serialization", $tags="Core")
        Container(release, "OPS-PIPE-ReleaseAutomation", "Lockstep releases & benchmarks", $tags="Core")
    }

    Rel(businessAnalyst, python, "Author models via Python API", "REQ-006/REQ-015")
    Rel(compliance, policy, "Reviews policy violations", "REQ-003/REQ-012")
    Rel(engineer, release, "Operates releases and benchmarks", "REQ-016/REQ-018")
    Rel(devRel, calmAdapter, "Publishes CALM exports", "REQ-014")
    Rel(python, core, "Invokes modelling API", "FFI/PyO3")
    Rel(ts, core, "Invokes modelling API", "N-API")
    Rel(wasm, core, "Invokes modelling API", "wit-bindgen")
    Rel(core, graph, "Reads/Writes primitives", "In-memory + indexes")
    Rel(core, policy, "Triggers validation", "Synchronous & streaming")
    Rel(parser, core, "Provides AST", "Rust module")
    Rel(adapter, core, "Imports ERP5 models", "Migration pipeline")
    Rel(adapter, erp5, "Pulls ERP5 UBM data", "REST/XMLRPC")
    Rel(calmAdapter, calm, "Exports CALM JSON", "HTTPS")
    Rel(release, pypi, "Publishes Python wheels", "GitHub Actions")
    Rel(release, npmjs, "Publishes npm packages", "GitHub Actions")
    Rel(release, crates, "Publishes Rust crates", "GitHub Actions")
    Rel(release, ci, "Executes tests & benchmarks", "Workflow jobs")
    Rel(wasm, edge, "Deploys validation to edge", "WebAssembly")
```

### Design Rationale
- Captures all external dependencies mandated in SDS/PRD to satisfy validation matrix.
- Highlights cross-language bindings fulfilling REQ-015 parity.

### Related Components
- Container drill-down provided in [sds-architecture-container-overview](../03-architecture/sds-architecture-container-overview.md).
- Trust boundaries refined in [sds-security-boundaries](../07-security/sds-security-boundaries.md).

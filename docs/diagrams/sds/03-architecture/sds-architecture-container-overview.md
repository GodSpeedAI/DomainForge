---
sds_section: "3. Architectural Design"
diagram_type: "C4Container"
component_ids: ["CORE-API-RustCore", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "PARSE-SVC-GrammarParser", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer", "OPS-PIPE-ReleaseAutomation", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-003", "ADR-004", "ADR-005", "ADR-006", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-001", "REQ-002", "REQ-003", "REQ-004", "REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-009", "REQ-012", "REQ-013", "REQ-014", "REQ-015", "REQ-016", "REQ-017", "REQ-018", "REQ-019"]
related_diagrams:
  - sds-architecture-service-interactions.md
  - ../04-components/sds-component-graph-runtime.md
  - ../08-deployment/sds-deployment-topology.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Container-level overview of SEA DSL platform aligning components with ADR decisions and PRD requirements."
---

## Container Architecture

Shows runtime containers and their collaborations, connecting SDS component specifications to ADR constraints. Highlights runtime boundaries (Rust core, bindings, adapters, ops) and data store interactions.

```mermaid
C4Container
    %% Source: docs/specs/sds.md Sections SDS-001..SDS-015
    %% Implements: ADR-001, ADR-002, ADR-003, ADR-004, ADR-005, ADR-006, ADR-007, ADR-008
    %% Satisfies: REQ-001, REQ-002, REQ-003, REQ-004, REQ-005, REQ-006, REQ-007, REQ-008, REQ-009, REQ-012, REQ-013, REQ-014, REQ-015, REQ-016, REQ-017, REQ-018, REQ-019
    %% Components: CORE-API-RustCore, CORE-DB-GraphStore, VAL-SVC-PolicyEvaluator, PARSE-SVC-GrammarParser, BIND-API-PyO3, BIND-API-TypeScript, BIND-API-WebAssembly, MIG-ADPT-ERP5Adapter, MIG-ADPT-CALMSerializer, OPS-PIPE-ReleaseAutomation, DOC-PIPE-DocGenerator

    AddElementTag("Core", $bgColor="#ecfdf5", $borderColor="#047857", $fontColor="#065f46")
    AddElementTag("Binding", $bgColor="#f0f9ff", $borderColor="#0ea5e9", $fontColor="#075985")
    AddElementTag("Adapter", $bgColor="#fdf2f8", $borderColor="#db2777", $fontColor="#831843")
    AddElementTag("Ops", $bgColor="#fff7ed", $borderColor="#f97316", $fontColor="#7c2d12")

    Container_Boundary(core_boundary, "SEA DSL Core", "Rust + async runtime", $tags="Core") {
        Container(core_api, "CORE-API-RustCore", "Rust crate", "Model API facade", $tags="Core")
        Container(graph_store, "CORE-DB-GraphStore", "Rust module", "Graph data store & indexes", $tags="Core")
        Container(policy_engine, "VAL-SVC-PolicyEvaluator", "Rust module", "Policy evaluation & streaming", $tags="Core")
        Container(parser, "PARSE-SVC-GrammarParser", "Rust module", "SBVR grammar parser", $tags="Core")
    }

    Container(pyo3, "BIND-API-PyO3", "PyO3 extension", "Python bindings", $tags="Binding")
    Container(ts_api, "BIND-API-TypeScript", "napi-rs addon", "TypeScript/Node bindings", $tags="Binding")
    Container(wasm_api, "BIND-API-WebAssembly", "wit-bindgen module", "WebAssembly bindings", $tags="Binding")

    Container(erp5_adapter, "MIG-ADPT-ERP5Adapter", "Rust/Python adapter", "ERP5 migration", $tags="Adapter")
    Container(calm_adapter, "MIG-ADPT-CALMSerializer", "Rust adapter", "CALM serialization", $tags="Adapter")

    Container(release_ops, "OPS-PIPE-ReleaseAutomation", "GitHub Actions workflows", "Lockstep releases & benchmarks", $tags="Ops")
    Container(doc_ops, "DOC-PIPE-DocGenerator", "Sphinx/TypeDoc pipelines", "Documentation generation", $tags="Ops")

    ContainerDb(graph_db, "Graph Memory", "In-memory + optional persistence", "Stores primitives & indexes", $tags="Core")

    Rel(core_api, graph_store, "Persists primitives", "Internal API")
    Rel(graph_store, graph_db, "Materializes data", "RwLock-backed")
    Rel(core_api, policy_engine, "Runs validation", "Sync/Async Rust calls")
    Rel(policy_engine, graph_store, "Traverses graph", "Query API")
    Rel(parser, core_api, "Provides AST", "Internal API")

    Rel(pyo3, core_api, "FFI invocations", "PyO3, REQ-006/REQ-015")
    Rel(ts_api, core_api, "FFI invocations", "napi-rs, REQ-007/REQ-015")
    Rel(wasm_api, core_api, "FFI invocations", "wit-bindgen, REQ-008/REQ-015")

    Rel(erp5_adapter, core_api, "Imports models", "Migration API")
    Rel(erp5_adapter, pyo3, "Python-based migration UX", "ERP5 scripts")
    Rel(calm_adapter, core_api, "Serializes models", "CALM exporter")

    Rel(release_ops, pyo3, "Build & publish wheels", "CI jobs")
    Rel(release_ops, ts_api, "Build & publish npm package", "CI jobs")
    Rel(release_ops, wasm_api, "Publish WASM artifacts", "CI jobs")
    Rel(release_ops, core_api, "Run tests & benchmarks", "Cargo/test matrix")
    Rel(doc_ops, core_api, "Generate Rust docs", "cargo doc")
    Rel(doc_ops, pyo3, "Generate Python docs", "Sphinx")
    Rel(doc_ops, ts_api, "Generate TypeDoc", "TypeDoc CLI")
    Rel(doc_ops, release_ops, "Publishes docs", "CI pipeline")
```

### Design Rationale
- Aligns with ADR layering; ensures each container references relevant ADR and PRD items.
- Shows operations containers to highlight compliance with REQ-016/REQ-019.

### Related Components
- Component-level drill-down provided in Section 4 diagrams.
- Deployment environment specifics described in [sds-deployment-topology](../08-deployment/sds-deployment-topology.md).

---
sds_section: "10. Traceability"
diagram_type: "Matrix"
component_ids: ["CORE-SVC-NamespaceResolver", "MODEL-COMP-EntityPrimitive", "MODEL-COMP-ResourcePrimitive", "MODEL-COMP-FlowPrimitive", "MODEL-COMP-InstancePrimitive", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "PARSE-SVC-GrammarParser", "CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer", "OPS-PIPE-ReleaseAutomation", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-003", "ADR-004", "ADR-005", "ADR-006", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-001", "REQ-002", "REQ-003", "REQ-004", "REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-009", "REQ-010", "REQ-011", "REQ-012", "REQ-013", "REQ-014", "REQ-015", "REQ-016", "REQ-017", "REQ-018", "REQ-019"]
related_diagrams:
  - ../03-architecture/sds-architecture-container-overview.md
  - ../04-components/sds-component-graph-runtime.md
  - ../08-deployment/sds-deployment-topology.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Provides canonical component identifiers, dependency edges, and diagram coverage for cross-diagram navigation."
---

| Component ID | Type | Description | Depends On | Consumed By | Diagram References |
|--------------|------|-------------|------------|-------------|--------------------|
| CORE-SVC-NamespaceResolver | Service | Manages namespaces, imports, lazy reference resolution (SDS-001) | — | CORE-API-RustCore; MODEL-COMP-* | [sds-component-namespace-resolver](../04-components/sds-component-namespace-resolver.md) |
| MODEL-COMP-EntityPrimitive | Component | Entity primitive with attributes/relations (SDS-002) | CORE-SVC-NamespaceResolver | CORE-DB-GraphStore; VAL-SVC-PolicyEvaluator | [sds-component-graph-runtime](../04-components/sds-component-graph-runtime.md) |
| MODEL-COMP-ResourcePrimitive | Component | Resource primitive with unit of measure (SDS-003) | CORE-SVC-NamespaceResolver | CORE-DB-GraphStore; VAL-SVC-PolicyEvaluator | [sds-data-schema-complete](../06-data/sds-data-schema-complete.md) |
| MODEL-COMP-FlowPrimitive | Component | Flow primitive linking entities and resources (SDS-004) | MODEL-COMP-EntityPrimitive; MODEL-COMP-ResourcePrimitive | CORE-DB-GraphStore; VAL-SVC-PolicyEvaluator | [sds-data-schema-complete](../06-data/sds-data-schema-complete.md) |
| MODEL-COMP-InstancePrimitive | Component | Instance primitive for physical resource tracking (SDS-004) | MODEL-COMP-ResourcePrimitive | CORE-DB-GraphStore | [sds-data-schema-complete](../06-data/sds-data-schema-complete.md) |
| CORE-DB-GraphStore | Database | Graph storage and indexes (SDS-005) | MODEL-COMP-* | VAL-SVC-PolicyEvaluator; CORE-API-RustCore | [sds-component-graph-runtime](../04-components/sds-component-graph-runtime.md) |
| VAL-SVC-PolicyEvaluator | Service | Policy evaluation engine with streaming (SDS-006) | CORE-DB-GraphStore; PARSE-SVC-GrammarParser | CORE-API-RustCore; BIND-API-* | [sds-component-policy-engine](../04-components/sds-component-policy-engine.md) |
| PARSE-SVC-GrammarParser | Service | SBVR-aligned parser (SDS-007) | — | VAL-SVC-PolicyEvaluator; CORE-API-RustCore | [sds-architecture-service-interactions](../03-architecture/sds-architecture-service-interactions.md) |
| CORE-API-RustCore | API | Rust core facade exposing modelling API (SDS-008) | CORE-SVC-NamespaceResolver; MODEL-COMP-*; CORE-DB-GraphStore; VAL-SVC-PolicyEvaluator | BIND-API-PyO3; BIND-API-TypeScript; BIND-API-WebAssembly | [sds-api-model-definition](../05-interfaces/sds-api-model-definition.md) |
| BIND-API-PyO3 | API | Python bindings (SDS-009) | CORE-API-RustCore | QA flows; Documentation examples | [sds-component-language-parity](../04-components/sds-component-language-parity.md) |
| BIND-API-TypeScript | API | TypeScript bindings via N-API (SDS-010) | CORE-API-RustCore | JS/Edge clients; QA parity | [sds-component-language-parity](../04-components/sds-component-language-parity.md) |
| BIND-API-WebAssembly | API | WebAssembly bindings (SDS-011) | CORE-API-RustCore | Browser/edge runtimes | [sds-component-language-parity](../04-components/sds-component-language-parity.md) |
| MIG-ADPT-ERP5Adapter | Adapter | ERP5 UBM migration tooling (SDS-012) | CORE-API-RustCore; MODEL-COMP-* | ERP5 pipelines; CALM serializer | [sds-data-migration-strategy](../06-data/sds-data-migration-strategy.md) |
| MIG-ADPT-CALMSerializer | Adapter | FINOS CALM integration (SDS-013) | CORE-API-RustCore; MIG-ADPT-ERP5Adapter | External CALM systems | [sds-data-migration-strategy](../06-data/sds-data-migration-strategy.md) |
| OPS-PIPE-ReleaseAutomation | Pipeline | CI/CD and benchmarking pipeline (SDS-014) | CORE-API-RustCore; BIND-API-* | Package registries; QA dashboards | [sds-deployment-cicd-pipeline](../08-deployment/sds-deployment-cicd-pipeline.md) |
| DOC-PIPE-DocGenerator | Pipeline | Documentation generation and publishing (SDS-015) | CORE-API-RustCore; BIND-API-* | Documentation portal | [sds-introduction-artifact-flow](../01-introduction/sds-introduction-artifact-flow.md) |

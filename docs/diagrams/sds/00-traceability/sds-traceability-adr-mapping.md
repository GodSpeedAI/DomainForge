---
sds_section: "10. Traceability"
diagram_type: "Matrix"
component_ids: ["CORE-SVC-NamespaceResolver", "MODEL-COMP-EntityPrimitive", "MODEL-COMP-ResourcePrimitive", "MODEL-COMP-FlowPrimitive", "CORE-DB-GraphStore", "VAL-SVC-PolicyEvaluator", "PARSE-SVC-GrammarParser", "CORE-API-RustCore", "BIND-API-PyO3", "BIND-API-TypeScript", "BIND-API-WebAssembly", "MIG-ADPT-ERP5Adapter", "MIG-ADPT-CALMSerializer", "OPS-PIPE-ReleaseAutomation", "DOC-PIPE-DocGenerator"]
implements_adrs: ["ADR-001", "ADR-002", "ADR-003", "ADR-004", "ADR-005", "ADR-006", "ADR-007", "ADR-008"]
satisfies_requirements: ["REQ-001", "REQ-002", "REQ-003", "REQ-004", "REQ-005", "REQ-006", "REQ-007", "REQ-008", "REQ-009", "REQ-010", "REQ-011", "REQ-012", "REQ-013", "REQ-014", "REQ-015", "REQ-016", "REQ-017", "REQ-018", "REQ-019"]
related_diagrams:
  - ../03-architecture/sds-architecture-container-overview.md
  - ../04-components/sds-component-graph-runtime.md
  - ../05-interfaces/sds-api-validation-streaming.md
  - ../08-deployment/sds-deployment-topology.md
updated: "2025-11-01"
reviewed_by: "Architecture Team"
purpose: "Maps each ADR to the SDS sections, component implementations, and diagrams where the decision is realised."
---

| ADR ID | Decision | SDS Sections | Implemented By | Diagram References |
|--------|----------|--------------|----------------|--------------------|
| ADR-001 | Layered Architecture: Vocabulary, Facts, Rules | Sections 2, 3, 4, 6 | CORE-SVC-NamespaceResolver; MODEL-COMP-EntityPrimitive; MODEL-COMP-FlowPrimitive | [sds-context-system-boundaries](../02-context/sds-context-system-boundaries.md), [sds-component-namespace-resolver](../04-components/sds-component-namespace-resolver.md), [sds-data-schema-complete](../06-data/sds-data-schema-complete.md) |
| ADR-002 | Rust Core with Multi-Language FFI Bindings | Sections 3, 5, 8 | CORE-API-RustCore; BIND-API-PyO3; BIND-API-TypeScript; BIND-API-WebAssembly | [sds-architecture-container-overview](../03-architecture/sds-architecture-container-overview.md), [sds-api-model-definition](../05-interfaces/sds-api-model-definition.md), [sds-deployment-topology](../08-deployment/sds-deployment-topology.md) |
| ADR-003 | Graph-Based Domain Model Representation | Sections 3, 4, 6 | CORE-DB-GraphStore; VAL-SVC-PolicyEvaluator | [sds-component-graph-runtime](../04-components/sds-component-graph-runtime.md), [sds-data-schema-complete](../06-data/sds-data-schema-complete.md), [sds-architecture-e2e-flow](../03-architecture/sds-architecture-e2e-flow.md) |
| ADR-004 | SBVR-Aligned Rule Expression Language | Sections 4, 5, 7 | VAL-SVC-PolicyEvaluator; PARSE-SVC-GrammarParser | [sds-component-policy-engine](../04-components/sds-component-policy-engine.md), [sds-api-validation-streaming](../05-interfaces/sds-api-validation-streaming.md), [sds-security-auth-flow](../07-security/sds-security-auth-flow.md) |
| ADR-005 | Five Core Primitives Design Pattern | Sections 2, 3, 6 | MODEL-COMP-EntityPrimitive; MODEL-COMP-ResourcePrimitive; MODEL-COMP-FlowPrimitive | [sds-context-system-boundaries](../02-context/sds-context-system-boundaries.md), [sds-data-schema-complete](../06-data/sds-data-schema-complete.md) |
| ADR-006 | CALM Interoperability for Architecture-as-Code | Sections 2, 6, 8 | MIG-ADPT-CALMSerializer | [sds-context-external-integrations](../02-context/sds-context-external-integrations.md), [sds-data-migration-strategy](../06-data/sds-data-migration-strategy.md), [sds-deployment-observability](../08-deployment/sds-deployment-observability.md) |
| ADR-007 | Idiomatic Language-Specific Bindings | Sections 4, 5, 8, 9 | BIND-API-PyO3; BIND-API-TypeScript; BIND-API-WebAssembly; DOC-PIPE-DocGenerator | [sds-component-language-parity](../04-components/sds-component-language-parity.md), [sds-api-ffi-error-handling](../05-interfaces/sds-api-ffi-error-handling.md), [sds-testing-integration-scenarios](../09-testing/sds-testing-integration-scenarios.md) |
| ADR-008 | Lockstep Versioning Strategy | Sections 8, 9, 10 | OPS-PIPE-ReleaseAutomation | [sds-deployment-cicd-pipeline](../08-deployment/sds-deployment-cicd-pipeline.md), [sds-deployment-scaling-strategy](../08-deployment/sds-deployment-scaling-strategy.md), [sds-testing-strategy](../09-testing/sds-testing-strategy.md) |

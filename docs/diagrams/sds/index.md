---
updated: "2025-11-01"
purpose: "Navigation hub for SDS diagram set, including section coverage, component IDs, and traceability assets."
reviewed_by: "Architecture Team"
---

# SDS Diagram Index

| Section | Diagrams | Source SDS Items | Key Requirements |
|---------|----------|------------------|------------------|
| 0. Traceability | [ADR mapping](00-traceability/sds-traceability-adr-mapping.md), [Requirement coverage](00-traceability/sds-traceability-req-mapping.md), [Component index](00-traceability/sds-traceability-component-index.md), [Validation report](00-traceability/sds-diagram-validation-report.md) | SDS Summary | REQ-001…REQ-019 |
| 1. Introduction | [Document relationship map](01-introduction/sds-introduction-artifact-flow.md) | SDS Overview | REQ-019 |
| 2. Context | [System boundaries](02-context/sds-context-system-boundaries.md), [External integrations](02-context/sds-context-external-integrations.md) | SDS Overview, SDS-012, SDS-013 | REQ-002, REQ-009, REQ-014 |
| 3. Architecture | [Container overview](03-architecture/sds-architecture-container-overview.md), [Service collaboration](03-architecture/sds-architecture-service-interactions.md), [End-to-end flow](03-architecture/sds-architecture-e2e-flow.md), [Error handling](03-architecture/sds-architecture-error-handling.md), [Observability](08-deployment/sds-deployment-observability.md) | SDS-001…SDS-015 | REQ-002, REQ-004, REQ-012, REQ-015 |
| 4. Components | [Namespace resolver](04-components/sds-component-namespace-resolver.md), [Graph runtime](04-components/sds-component-graph-runtime.md), [Policy engine](04-components/sds-component-policy-engine.md), [Language parity harness](04-components/sds-component-language-parity.md) | SDS-001, SDS-005, SDS-006, SDS-008…SDS-011 | REQ-003, REQ-004, REQ-010, REQ-012, REQ-015 |
| 5. Interfaces & APIs | [Model definition sequence](05-interfaces/sds-api-model-definition.md), [Validation streaming](05-interfaces/sds-api-validation-streaming.md), [FFI error propagation](05-interfaces/sds-api-ffi-error-handling.md) | SDS-006…SDS-011 | REQ-003, REQ-005, REQ-006, REQ-007, REQ-008, REQ-013 |
| 6. Data Design | [Graph schema](06-data/sds-data-schema-complete.md), [Data pipeline](06-data/sds-data-pipeline-events.md), [Consistency model](06-data/sds-data-consistency-model.md), [Migration strategy](06-data/sds-data-migration-strategy.md) | SDS-002…SDS-005, SDS-012, SDS-013 | REQ-002, REQ-004, REQ-009, REQ-014 |
| 7. Security & Compliance | [Trust boundaries](07-security/sds-security-boundaries.md), [Authentication flow](07-security/sds-security-auth-flow.md), [Threat model](07-security/sds-security-threat-model.md) | SDS-001, SDS-006, SDS-008…SDS-011 | REQ-003, REQ-005, REQ-012, REQ-015, REQ-017 |
| 8. Deployment & Operations | [Deployment topology](08-deployment/sds-deployment-topology.md), [CI/CD pipeline](08-deployment/sds-deployment-cicd-pipeline.md), [Scaling strategy](08-deployment/sds-deployment-scaling-strategy.md), [Observability](08-deployment/sds-deployment-observability.md) | SDS-008…SDS-015 | REQ-005, REQ-016, REQ-018 |
| 9. Testing & QA | [Testing strategy](09-testing/sds-testing-strategy.md), [Integration scenarios](09-testing/sds-testing-integration-scenarios.md) | SDS-006, SDS-008…SDS-015 | REQ-012, REQ-013, REQ-015, REQ-017, REQ-018 |
| 10. Traceability | [Coverage summary](00-traceability/sds-requirement-coverage-report.md), [ADR impact analysis](00-traceability/sds-adr-impact-report.md) | SDS-001…SDS-015 | REQ-001…REQ-019 |

> **Requirement Alias Note:** `REQ-###` identifiers map one-to-one with `PRD-###` entries from `docs/specs/prd.md`. The alias aligns SDS diagrams with the traceability controls mandated in the prompt.

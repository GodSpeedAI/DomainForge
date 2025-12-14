# DomainForge Specifications

This directory contains the architectural documentation for DomainForge and the SEA Domain-Specific Language, including core architecture, projection framework, integrations, and tooling.

## Document Overview

### Architecture Decision Records (ADRs)

| Document                                                   | Status   | Summary                                         |
| ---------------------------------------------------------- | -------- | ----------------------------------------------- |
| [ADR-001](./ADR-001-sea-dsl-semantic-source-of-truth.md)   | Accepted | SEA-DSL remains the semantic source of truth    |
| [ADR-002](./ADR-002-projection-first-class-construct.md)   | Accepted | Projection as a first-class DSL construct       |
| [ADR-003](./ADR-003-protobuf-projection-target.md)         | Accepted | Protobuf as a projection target                 |
| [ADR-004](./ADR-004-projection-compatibility-semantics.md) | Accepted | Compatibility enforcement semantics             |
| [ADR-005](./ADR-005-multi-language-support-strategy.md)    | Accepted | Multi-language support via Rust core + bindings |
| [ADR-006](./ADR-006-error-handling-strategy.md)            | Accepted | Rich error handling with FFI propagation        |
| [ADR-007](./ADR-007-policy-evaluation-engine.md)           | Accepted | Three-valued logic and quantifier support       |
| [ADR-008](./ADR-008-knowledge-graph-integration.md)        | Accepted | RDF/Turtle export and SHACL validation          |
| [ADR-009](./ADR-009-module-resolution-strategy.md)         | Accepted | Import syntax and namespace registry            |
| [ADR-010](./ADR-010-unit-system.md)                        | Accepted | Dimensional analysis and unit conversions       |

### Product Requirements Documents (PRDs)

| Document                                         | Status      | Summary                                     |
| ------------------------------------------------ | ----------- | ------------------------------------------- |
| [PRD-001](./PRD-001-sea-projection-framework.md) | Draft       | SEA Projection Framework requirements       |
| [PRD-002](./PRD-002-sea-cli-tooling.md)          | Implemented | SEA CLI commands and options                |
| [PRD-003](./PRD-003-dsl-core-capabilities.md)    | Implemented | Core DSL primitives and expression language |

### Software Design Specifications (SDSs)

| Document                                           | Status      | Summary                                   |
| -------------------------------------------------- | ----------- | ----------------------------------------- |
| [SDS-001](./SDS-001-protobuf-projection-engine.md) | Draft       | Protobuf Projection Engine design         |
| [SDS-002](./SDS-002-sea-core-architecture.md)      | Implemented | Core library architecture and modules     |
| [SDS-003](./SDS-003-parser-semantic-graph.md)      | Implemented | Parser pipeline and graph construction    |
| [SDS-004](./SDS-004-policy-engine-design.md)       | Implemented | Policy evaluation with three-valued logic |
| [SDS-005](./SDS-005-knowledge-graph-module.md)     | Implemented | RDF/Turtle export/import and SHACL        |
| [SDS-006](./SDS-006-calm-integration.md)           | Implemented | FINOS CALM architecture-as-code           |
| [SDS-007](./SDS-007-sbvr-import.md)                | Implemented | OMG SBVR XMI business vocabulary import   |

## Key Principles

1. **Semantic Primacy**: SEA-DSL remains a semantic language; all schemas are derived, never authored directly.

2. **Declarative Projections**: Projection intent is declared explicitly in the DSL, enabling governance and auditing.

3. **Compatibility by Default**: All projections enforce compatibility rules to prevent breaking downstream consumers.

4. **Traceability**: Every generated artifact links back to its semantic source and projection declaration.

5. **Single Source of Truth**: Rust core (`sea-core`) is the single implementation; all language bindings wrap it.

6. **Standards Alignment**: Built on SBVR semantics, RDF interoperability, and FINOS CALM architecture standards.

## Quick Links

- [Implementation Roadmap](../plans/protobuf_plan.yml)
- [SEA-DSL Reference](../../sea-core/README.md)
- [Python README](../../README_PYTHON.md)
- [TypeScript README](../../README_TYPESCRIPT.md)
- [WASM README](../../README_WASM.md)

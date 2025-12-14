# SEA Projection Framework Specifications

This directory contains the architectural documentation for the SEA Projection Framework, which enables generation of runtime artifacts (Protobuf, GraphQL, etc.) from SEA-DSL semantic models.

## Document Overview

### Architecture Decision Records (ADRs)

| Document                                                   | Status   | Summary                                      |
| ---------------------------------------------------------- | -------- | -------------------------------------------- |
| [ADR-001](./ADR-001-sea-dsl-semantic-source-of-truth.md)   | Accepted | SEA-DSL remains the semantic source of truth |
| [ADR-002](./ADR-002-projection-first-class-construct.md)   | Accepted | Projection as a first-class DSL construct    |
| [ADR-003](./ADR-003-protobuf-projection-target.md)         | Accepted | Protobuf as a projection target              |
| [ADR-004](./ADR-004-projection-compatibility-semantics.md) | Accepted | Compatibility enforcement semantics          |

### Product Requirements

| Document                                         | Status | Summary                               |
| ------------------------------------------------ | ------ | ------------------------------------- |
| [PRD-001](./PRD-001-sea-projection-framework.md) | Draft  | SEA Projection Framework requirements |

### Software Design Specifications

| Document                                           | Status | Summary                           |
| -------------------------------------------------- | ------ | --------------------------------- |
| [SDS-001](./SDS-001-protobuf-projection-engine.md) | Draft  | Protobuf Projection Engine design |

## Key Principles

1. **Semantic Primacy**: SEA-DSL remains a semantic language; all schemas are derived, never authored directly.

2. **Declarative Projections**: Projection intent is declared explicitly in the DSL, enabling governance and auditing.

3. **Compatibility by Default**: All projections enforce compatibility rules to prevent breaking downstream consumers.

4. **Traceability**: Every generated artifact links back to its semantic source and projection declaration.

## Quick Links

- [Implementation Roadmap](../plans/protobuf_plan.yml)
- [SEA-DSL Reference](../../sea-core/README.md)

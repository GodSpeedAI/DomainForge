# ADR-001: Treat SEA-DSL as the Semantic Source of Truth

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

SEA-DSL models enterprise meaning using constructs such as `Entity`, `Resource`, `Flow`, `Policy`, `Metric`, and `ConceptChange`. There is a need to generate schemas, code, and runtime contracts (DDD/Hex, Protobuf, JSON, XML) without collapsing the DSL into a schema-first or implementation-first language.

## Decision

SEA-DSL SHALL remain a **semantic language only**.

All schemas, code artifacts, and runtime contracts SHALL be produced via **projection engines** from the semantic graph.

## Consequences

### Positive

- SEA-DSL remains stable, expressive, and technology-agnostic.
- Multiple projections (Protobuf, DDD manifests, GraphQL, etc.) can coexist.
- Schema evolution is governed by semantic evolution, not implementation drift.

### Negative

- Requires additional tooling (projection engines) to produce usable artifacts.
- Learning curve for users who expect schema-first approaches.

## Related

- [ADR-002: Introduce Projection as a First-Class DSL Construct](./ADR-002-projection-first-class-construct.md)
- [ADR-003: Add Protobuf as a Projection Target](./ADR-003-protobuf-projection-target.md)

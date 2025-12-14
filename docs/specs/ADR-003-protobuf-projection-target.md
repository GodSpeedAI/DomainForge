# ADR-003: Add Protobuf as a Projection Target (Not a Semantic Format)

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

SEA requires high-performance, versioned, cross-language contracts for agents, services, governance events, and telemetry.

## Decision

Add **Protobuf** as a supported projection target.

Protobuf SHALL be generated **only** from the semantic graph via a projection engine.

Protobuf SHALL NOT:

- Replace SEA-DSL
- Encode policy logic
- Encode semantic evolution rules

## Rationale

Protobuf provides:

- Efficient binary serialization
- Strong cross-language support
- Built-in versioning primitives
- Industry-standard tooling

By treating Protobuf as a projection target rather than a source format, we maintain SEA-DSL's semantic integrity while gaining Protobuf's runtime benefits.

## Consequences

### Positive

- Agents receive stable, deterministic contracts.
- Runtime governance and telemetry become efficient and auditable.
- SEA avoids becoming data-centric or schema-centric.

### Negative

- Requires building and maintaining a Protobuf projection engine.
- Some Protobuf features (services, streaming) may require additional mapping work.

## Related

- [ADR-001: Treat SEA-DSL as the Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
- [ADR-002: Introduce Projection as a First-Class DSL Construct](./ADR-002-projection-first-class-construct.md)
- [ADR-004: Enforce Projection Compatibility Semantics](./ADR-004-projection-compatibility-semantics.md)

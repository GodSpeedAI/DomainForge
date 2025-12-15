# ADR-002: Introduce Projection as a First-Class DSL Construct

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

Enterprises need explicit, governable control over _which_ parts of the semantic model are projected _where_, _for whom_, and _under what compatibility guarantees_.

## Decision

Introduce a top-level `Projection` construct in SEA-DSL that:

- Declares **scope** (namespace, tags, profiles)
- Declares **target format** (protobuf, ddd-manifest, etc.)
- Declares **governance metadata** (stability, compatibility, audience)

Projection constructs SHALL NOT contain format-specific details.

### Example Syntax

```sea
Projection PaymentsProto {
  source: "com.example.finance.payments"
  format: protobuf
  compatibility: backward
  stability: stable
  audience: ["agents", "services"]
}
```

## Consequences

### Positive

- Projections are auditable and declarative.
- Compatibility rules can be enforced centrally.
- Multiple consumers (agents, services, pipelines) can rely on explicit contracts.

### Negative

- Adds complexity to the DSL grammar.
- Requires governance tooling to validate projection declarations.

## Related

- [ADR-001: Treat SEA-DSL as the Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
- [ADR-003: Add Protobuf as a Projection Target](./ADR-003-protobuf-projection-target.md)
- [ADR-004: Enforce Projection Compatibility Semantics](./ADR-004-projection-compatibility-semantics.md)

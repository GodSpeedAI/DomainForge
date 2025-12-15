# ADR-004: Enforce Projection Compatibility Semantics

**Status:** Accepted  
**Date:** 2025-12-14  
**Deciders:** DomainForge Architecture Team

## Context

Protobuf and other schema formats require strong backward-compatibility guarantees to avoid breaking agents and services.

## Decision

Each `Projection` SHALL declare a compatibility level:

| Level      | Description                                                          |
| ---------- | -------------------------------------------------------------------- |
| `additive` | Only new fields may be added. No removals or modifications.          |
| `backward` | Backward-compatible changes allowed. Removed fields become reserved. |
| `breaking` | Structural changes allowed with explicit version bump.               |

Projection engines SHALL enforce these rules when generating artifacts.

### Enforcement Rules

1. **Additive Mode**

   - New fields: ✅ Allowed
   - Field removal: ❌ Rejected
   - Field modification: ❌ Rejected

2. **Backward Mode**

   - New fields: ✅ Allowed
   - Field removal: ✅ Allowed (becomes `reserved`)
   - Field type changes: ❌ Rejected

3. **Breaking Mode**
   - All changes: ✅ Allowed
   - Requires: Major version increment

## Consequences

### Positive

- Safe evolution of runtime contracts.
- Breaking changes are explicit and auditable.
- Field numbering and reservations are handled deterministically.

### Negative

- Requires tracking schema history for diff operations.
- May slow down iteration during early development phases.

## Related

- [ADR-002: Introduce Projection as a First-Class DSL Construct](./ADR-002-projection-first-class-construct.md)
- [ADR-003: Add Protobuf as a Projection Target](./ADR-003-protobuf-projection-target.md)

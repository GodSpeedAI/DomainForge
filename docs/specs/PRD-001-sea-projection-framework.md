# PRD-001: SEA Projection Framework

**Product:** SEA Projection Framework  
**Primary Feature:** Protobuf Projection Engine  
**Version:** 1.0  
**Date:** 2025-12-14  
**Status:** Draft

---

## 1. Goals

- Enable SEA-DSL users to declare projection intent explicitly.
- Generate Protobuf schemas from semantic meaning, not implementation details.
- Support agentic systems, governance pipelines, and services.
- Enforce compatibility and stability guarantees automatically.

---

## 2. Non-Goals

- SEA-DSL is **not** a schema language.
- Protobuf is **not** an authoring format.
- Projection engines do **not** infer business meaning.
- No hand-authored `.proto` files as authoritative sources.

---

## 3. User Stories

### 3.1 As a Domain Architect

> I want to declare that a namespace should be exposed to agents as Protobuf so that AI systems can consume enterprise meaning safely.

**Acceptance Criteria:**

- Can define a `Projection` in SEA-DSL targeting a namespace
- Generated `.proto` files contain all entities/resources in that namespace
- Projection metadata is embedded in generated files

### 3.2 As a Platform Engineer

> I want Protobuf schemas to evolve backward-compatibly so agents don't break when semantics evolve.

**Acceptance Criteria:**

- Compatibility level is enforced during generation
- Removed fields become `reserved` automatically
- Build fails if compatibility rules are violated

### 3.3 As a Governance Lead

> I want to trace every runtime message back to a semantic definition and projection declaration.

**Acceptance Criteria:**

- Generated files include source projection name
- Generated files include semantic version
- Comments link back to source namespace

---

## 4. Functional Requirements

### FR-1: Projection Declaration

- DSL supports a `Projection` construct.
- Projection supports:
  - Source namespace(s)
  - Filters (kinds, tags, profiles)
  - Target format
  - Metadata (stability, compatibility, audience)

### FR-2: Semantic Graph Filtering

- Projection engine can select graph nodes by:
  - Namespace
  - Kind (`Entity`, `Resource`, `Flow`, etc.)
  - Tags
  - Profiles

### FR-3: Protobuf Generation

- Generate `.proto` files containing:
  - `message` definitions for Entities and Resources
  - `enum` definitions
  - Governance/event messages
- Deterministic field numbering.
- Namespaces map to Protobuf packages.

### FR-4: Compatibility Enforcement

- Enforce additive/backward/breaking rules.
- Prevent illegal field reuse.
- Emit `reserved` fields when needed.

### FR-5: Traceability

- Generated files include metadata:
  - Projection name
  - Semantic version
  - Source namespace

---

## 5. Quality Attributes

| Attribute         | Description                         |
| ----------------- | ----------------------------------- |
| **Deterministic** | Same input → same output            |
| **Auditable**     | Traceable to DSL and projection     |
| **Safe**          | Backward compatibility enforced     |
| **Extensible**    | New projection targets can be added |

---

## 6. Success Metrics

| Metric                          | Target                            |
| ------------------------------- | --------------------------------- |
| Projection coverage             | 100% of declared namespaces       |
| Compatibility violations caught | 100% at build time                |
| Generation time                 | < 1s for typical domain model     |
| Traceability                    | Every message traceable to source |

---

## 7. Dependencies

- SEA-DSL Parser (existing)
- Semantic Graph Builder (existing)
- Projection Resolver (new)
- Protobuf Projection Engine (new)

---

## 8. Milestones

| Phase | Deliverable               | Status  |
| ----- | ------------------------- | ------- |
| 1     | Projection DSL syntax     | Planned |
| 2     | Projection Resolver       | Planned |
| 3     | Basic Protobuf generation | Planned |
| 4     | Compatibility enforcement | Planned |
| 5     | Traceability metadata     | Planned |

---

## Related Documents

- [ADR-001: SEA-DSL as Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
- [ADR-002: Projection as First-Class Construct](./ADR-002-projection-first-class-construct.md)
- [ADR-003: Protobuf as Projection Target](./ADR-003-protobuf-projection-target.md)
- [ADR-004: Projection Compatibility Semantics](./ADR-004-projection-compatibility-semantics.md)
- [SDS-001: Protobuf Projection Engine](./SDS-001-protobuf-projection-engine.md)

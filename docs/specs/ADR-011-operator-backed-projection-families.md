# ADR-011: Operator-Backed Projection Families

**Status:** Accepted  
**Date:** 2026-07-06  
**Deciders:** DomainForge Architecture Team

## Context

DomainForge projects its canonical semantic graph into many external
ecosystems (Lean, Protobuf, AI-learning datasets today; RDF/OWL, BPMN, CMMN,
ArchiMate, OpenTelemetry, BAML, DSPy, ZenML next). The strategic goal is to
"unlock mature external ecosystems through DomainForge's canonical semantic
model while preserving deterministic identity, semantic traceability, and
projection consistency."

The risk is a bag of one-off exporters: each target inventing its own output
plumbing, its own identity scheme, and its own test scaffolding. The Lean and
AI-learning projections already converged on a shared shape; this ADR *names*
that shape and relocates its cross-cutting machinery into a kernel so reviewers
can reject exporters that do not conform.

Each target corresponds to an **operator family** — a class of external tool
that operates over a projection of the model. The families and their targets:

| Operator family        | Projection target | Format          |
| ---------------------- | ----------------- | --------------- |
| Adaptive work / case   | CMMN              | XML (CMMN 1.1)  |
| Ordered process        | BPMN              | XML (BPMN 2.0)  |
| Enterprise architecture| ArchiMate         | XML (Model Exchange File) |
| Semantic graph         | RDF / OWL         | Turtle + JSON-LD |
| Runtime observation    | OpenTelemetry     | SemConv YAML + constants |
| Learning loop          | ZenML             | Python pipeline |
| Formal verification    | Lean              | Lake package    |
| AI representation       | BAML              | `.baml` files   |
| AI optimization        | DSPy              | Python program  |

## Decision

### 1. Core principle — one projection, one shape

Every projection family MUST be built as:

- **one IR module** — a target-specific intermediate representation built from
  the graph (never rendered straight from the AST);
- **one renderer** — pure IR → bytes, no graph access, no I/O;
- **one `emit(graph, …, &mut ArtifactSink)`** entry point;
- **one `project_<x>_in_memory` binding surface** — string in, relative-path →
  content map out, no filesystem (wasm-safe);
- **one proving fixture** — `fixtures/<family>/basic/domain/model.sea` plus
  expected-output assertions in `domainforge-core/tests/<family>_projection_tests.rs`;
- **one external-toolchain CI `verify-<family>` job** — runs the real ecosystem
  tool (`lake build`, `xmllint`, `weaver`, `baml-cli`, …) against a pinned
  version.

Never render directly from the AST inside the CLI. Never duplicate output logic
in bindings or tests — bindings and tests call `project_<x>_in_memory`; nothing
re-implements rendering. The reference implementation to copy is
`domainforge-core/src/projection/lean/mod.rs` (not `protobuf.rs`, which predates
the pattern).

### 2. Shared kernel

The cross-cutting machinery lives in one place under
`domainforge-core/src/projection/`:

- **`sink.rs` — `ArtifactSink`**: `Dir(&Path)` for the CLI, `Memory { prefix,
  map }` for bindings. Both write through the same type, so CLI and binding
  output stay byte-for-byte identical. (Relocated from `ai_learning`; the old
  path re-exports it.)
- **`ids.rs` — deterministic identity**: `element_id(family, parts)` and
  `content_hash(parts)` hash a U+0001-joined tuple with
  `authority::compute_deterministic_hash` (xxh64 seed 42, 16 hex chars);
  `sanitize_qname(raw)` maps arbitrary strings to safe XML QName / IRI
  fragments (ASCII alphanumerics plus `_ - .`, leading digit escaped). Every
  family MUST mint ids and fragments through this module so identity is uniform
  across targets.
- **`tests/common/projection_harness.rs`**: the fixed `FIXED_TS` timestamp,
  `read_tree`, and `assert_byte_deterministic`, shared via the Rust
  `tests/common/mod.rs` integration-test convention.

### 3. Deterministic ID convention

- Element ids and record ids are content hashes of a U+0001-joined part tuple.
- The separator is illegal in the human-authored names it joins, so it cannot
  be forged inside a part to force a collision.
- `element_id` prepends the family tag as the first hashed field, so equal part
  tuples yield distinct ids under different families.
- Determinism is a gate, not a goal: a fixed `created_at` MUST yield identical
  bytes. No wall-clock, no randomness, no `HashMap` iteration order in any
  renderer — `BTreeMap`/`BTreeSet` and sorted iteration only.

### 4. CLI-format-driven targets; ADR-002 contracts govern scope only

Projection targets stay **CLI-format-driven** (`domainforge project --format
<x>`), following the Lean precedent (`docs/lean-projections.md` Non-goals).
The DSL-level `Projection` / `Mapping` contracts of ADR-002
(`domainforge-core/src/projection/registry.rs`, `contracts.rs`) govern
**scope and compatibility** — which elements are in a projection's remit and
whether a change is breaking — **not** output-format detail. A target does not
need a declared `Projection` contract to exist; the format flag is the surface,
the contract is optional governance metadata.

### 5. No plugin framework

Nine statically-known targets do not justify a trait-object registry or dynamic
loading: one enum variant plus one match arm per format. Revisit only on a
concrete third-party-target requirement.

## Consequences

### Positive

- New families are a mechanical copy of a known shape; reviewers have a
  checklist and can reject one-off exporters.
- Identity is uniform and traceable: runtime telemetry, RDF IRIs, and dataset
  record ids all derive from one hash convention rooted in the model hash.
- CLI and language bindings cannot diverge — single producer per artifact.

### Negative

- The kernel is a shared dependency; a change to `ArtifactSink` or `ids.rs`
  touches every family (mitigated by the determinism test suite).
- Statically-enumerated formats mean third-party targets require a code change
  (accepted deliberate simplification).

## Related

- [ADR-001: SEA-DSL as the Semantic Source of Truth](./ADR-001-sea-dsl-semantic-source-of-truth.md)
- [ADR-002: Projection as a First-Class DSL Construct](./ADR-002-projection-first-class-construct.md)
- [ADR-003: Protobuf as a Projection Target](./ADR-003-protobuf-projection-target.md)
- [ADR-004: Projection Compatibility Semantics](./ADR-004-projection-compatibility-semantics.md)
- [ADR-008: Knowledge Graph Integration](./ADR-008-knowledge-graph-integration.md)
- `docs/lean-projections.md`, `docs/ai-learning-projections.md`

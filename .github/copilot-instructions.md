# DomainForge - SEA DSL Development Guide

## Project Overview

**SEA (Semantic Enterprise Architecture) DSL** is a multi-layered domain modeling framework designed for high-performance, cross-language business rule execution. This is currently a **documentation-first project** with comprehensive architectural specifications but no implementation yet.

**Core Architecture**: Rust core with idiomatic bindings for Python (PyO3), TypeScript (napi-rs), and WebAssembly (wit-bindgen)
**Mathematical Foundation**: Typed directed multigraphs, many-sorted algebra, first-order logic (SBVR-aligned)
**Integration Target**: Bidirectional interoperability with FINOS CALM (Architecture-as-Code)

## Critical Architectural Constraints

### Three-Layer Architecture (ADR-001)

All domain models follow strict layering with **no upward dependencies**:

- **Vocabulary Layer**: Business terms (Entity, Resource, Enums) — no dependencies
- **Fact Model Layer**: Relationships (Flow, Instance) — depends only on Vocabulary
- **Rule Layer**: Constraints (Policy) — references Vocabulary and Fact Model only

### Five Universal Primitives (ADR-005)

Everything models using exactly these primitives:

- **Entity**: Business actors/locations (e.g., "Assembly Line A")
- **Resource**: Quantifiable subjects of value (e.g., "Camera units")
- **Flow**: Transfers between entities (`resource`, `from`, `to`, `quantity`)
- **Instance**: Physical resource instances (`of` resource, `at` entity)
- **Policy**: Constraints with SBVR expressions and deontic modalities

### Cross-Language Semantic Consistency (ADR-007, PRD-015)

All language bindings must produce **identical results** for the same model. The TypeScript Ohm-JS parser is the **reference implementation** — Rust ports must match its semantics exactly.

## Documentation Navigation

This project uses rigorous documentation-as-code with full traceability:

### Specification Hierarchy

1. **`docs/specs/prd.md`**: Product requirements (PRD-001 through PRD-019) with EARS format
2. **`docs/specs/adr.md`**: Architectural decisions (ADR-001 through ADR-008) with YAML metadata
3. **`docs/specs/sds.md`**: Software design specs (SDS-001 through SDS-015)
4. **`docs/specs/technical-specifications.md`**: Integration/performance specs (INT-_, PERF-_, SEC-\*)
5. **`docs/specs/traceability-matrix.md`**: Bidirectional ADR↔PRD↔SDS↔Tech Spec mappings

### Diagram Assets

All diagrams use Mermaid syntax with mandatory traceability front matter:

- **`docs/diagrams/prd/`**: Product requirement visualizations by section (01-overview through 10-acceptance)
- **`docs/diagrams/ard/ADR-00X/`**: C4 models (Context/Container/Component), flows, sequences per ADR
- **`docs/diagrams/sds/`**: System design diagrams (00-traceability through 09-testing)

**Example front matter pattern** (every diagram must include):

```yaml
---
adr: ADR-002
diagram_type: C4Component
implements_adrs: ["ADR-002", "ADR-007"]
satisfies_requirements: ["REQ-005", "REQ-016"]
generated: 2025-11-01
---
```

### Context Documents

- **`context/DSL_instructions.md`**: Complete API specification guidelines with FFI design patterns
- **`context/Rust_details.md`**: Cross-language library architectural patterns whitepaper
- **`context/SBVR_reverse_engineered.md`**: Mathematical foundations and SBVR semantics
- **`context/UBM_mine.md`**: Ohm-JS grammar (reference implementation)

## Development Workflows (Planned)

### When Implementation Begins

**Build System**:

```bash
# Rust core
cargo build --release
cargo test --all-features

# Python bindings
maturin develop  # Development install
maturin build --release  # Production wheels

# TypeScript bindings
npm run build  # Compiles napi-rs bindings
npm test

# WebAssembly
wasm-pack build --target web
```

**Lockstep Versioning (ADR-008)**:

- Single `VERSION` file at repository root drives all packages
- All `Cargo.toml`, `pyproject.toml`, `package.json` read from VERSION
- CI enforces consistency across Rust/Python/TypeScript/WASM releases
- Publish simultaneously to crates.io, PyPI, npm

**Testing Requirements** (from context/DSL_instructions.md):

- [ ] Cross-language equivalence tests (Python ≡ TypeScript ≡ Rust ≡ WASM)
- [ ] Validation of 10K node graph <100ms (PERF-001)
- [ ] FFI operations <1ms overhead amortized (PERF-002)
- [ ] CALM export/import lossless round-trip (INT-004)
- [ ] Grammar porting preserves Ohm-JS semantics exactly

## Code Generation Conventions (When Implementing)

### Rust Core Patterns

```rust
// Graph representation (ADR-003, SDS-005)
struct Graph {
    entities: NodeSet<Entity>,
    resources: NodeSet<Resource>,
    flows: NodeSet<Flow>,

    // Edge maps (adjacency lists)
    moves: EdgeMap<FlowId, ResourceId>,
    from_edges: EdgeMap<FlowId, EntityId>,
    to_edges: EdgeMap<FlowId, EntityId>,
}

// Policy evaluator with SBVR semantics (ADR-004, SDS-006)
enum Expression {
    Quantifier { kind: Quantifier, var: String, range: Box<Expr>, body: Box<Expr> },
    BinaryOp { op: BinOp, left: Box<Expr>, right: Box<Expr> },
    // ...
}
```

### FFI Boundary Design (ADR-007)

**Python must return native Python objects** (not Rust wrappers):

```python
# GOOD: Native Python dataclass
@dataclass
class Entity:
    id: UUID
    name: str
    namespace: Optional[str]
```

**TypeScript must follow idiomatic camelCase**:

```typescript
// GOOD: Idiomatic TypeScript interface
interface Entity {
  id: string;
  name: string;
  namespace?: string;
}
```

### Error Handling (SDS-008, PRD-017)

- Rust errors propagate to native exception types per language
- Include DSL file/line context when available
- Deontic modalities map to severity: `info` | `warn` | `error`

## Adding New Documentation

### When Adding Requirements

1. Update `docs/specs/prd.md` using EARS format
2. Create diagrams in `docs/diagrams/prd/0X-section/` with front matter
3. Update `docs/specs/traceability-matrix.md` mappings
4. Reference in related ADR/SDS documents

### When Making Architectural Decisions

1. Add ADR to `docs/specs/adr.md` with full YAML metadata
2. Create diagram set in `docs/diagrams/ard/ADR-00X/`:
   - Context diagram (C4Context)
   - Flow diagrams (forces, decision rationale)
   - Container/Component diagrams
   - Sequence diagrams for key interactions
3. Update index files (`docs/diagrams/ard/index.md`, `docs/diagrams/ard/ADR-00X/README.md`)
4. Update traceability matrix

### Diagram Naming Convention

Pattern: `{spec-type}-{section}-{diagram-purpose}.md`

Examples:

- `prd-requirements-traceability.md` (PRD Section 7)
- `ADR-002-component-ffi-pipeline.md` (ADR-002 implementation view)
- `sds-architecture-container-overview.md` (SDS Section 3)

## Performance Targets (Non-Negotiable)

From PRD success metrics and technical specs:

- Model validation: <100ms for 10,000 node graphs (PERF-001)
- FFI overhead: <1ms per call amortized (PERF-002)
- WASM bundle: <500KB gzipped (INT-003)
- Memory efficiency: Graph storage <100 bytes per node (PERF-003)

## Common Patterns & Anti-Patterns

### ✅ DO

- Start with documentation updates (this is a docs-first project)
- Maintain traceability metadata in all diagrams
- Preserve Ohm-JS reference semantics when implementing parsers
- Use native types at FFI boundaries (Python dicts, not Rust structs)
- Include acceptance criteria in EARS format for requirements

### ❌ DON'T

- Skip traceability front matter in diagrams
- Create vocabulary items that depend on higher layers
- Expose Rust-specific idioms in language bindings
- Add primitives beyond the five universal types
- Break lockstep versioning across language packages

## AI Agent Reminders

1. **This is documentation-first**: No code exists yet. Focus on specs/diagrams before suggesting implementation.
2. **Traceability is mandatory**: Every diagram needs front matter linking to ADR/PRD/SDS items.
3. **Reference implementation is TypeScript**: When implementing Rust core, match Ohm-JS parser semantics exactly.
4. **Cross-language consistency is critical**: Test suites must verify Python ≡ TypeScript ≡ Rust ≡ WASM.
5. **Diagram limits**: Mermaid diagrams max 20 nodes (prompt constraints), include PlantUML `!include` hints.

## Quick Reference

| Document                            | Purpose                 | Key Sections                                               |
| ----------------------------------- | ----------------------- | ---------------------------------------------------------- |
| `context/DSL_instructions.md`       | API specification guide | Sections 1-10: Rust Core, Python/TS/WASM APIs, Testing, DX |
| `docs/specs/prd.md`                 | Product requirements    | PRD-001 (Layering), PRD-003 (SBVR), PRD-005-008 (FFI)      |
| `docs/specs/adr.md`                 | Architectural decisions | ADR-001 (Layers), ADR-002 (FFI), ADR-007 (Idioms)          |
| `docs/specs/sds.md`                 | Design specifications   | SDS-006 (Policy Engine), SDS-008-011 (Bindings)            |
| `docs/specs/traceability-matrix.md` | Impact analysis         | ADR→PRD→SDS→TechSpec mappings                              |

---

**Last Updated**: 2025-11-01
**Project Phase**: Documentation & Architectural Design
**Next Milestone**: Rust core implementation + PyO3/napi-rs binding scaffolds

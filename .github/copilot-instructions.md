# DomainForge - SEA DSL Development Guide

## Project Overview

**SEA (Semantic Enterprise Architecture) DSL** is a multi-layered domain modeling framework for high-performance, cross-language business rule execution. Currently in the **documentation-first phase** with comprehensive architectural specifications - **no production code exists yet**.

**Core Architecture**: Rust core with idiomatic bindings for Python (PyO3), TypeScript (napi-rs), and WebAssembly (wit-bindgen)
**Mathematical Foundation**: Typed directed multigraphs, many-sorted algebra, first-order logic (SBVR-aligned)
**Integration Target**: Bidirectional interoperability with FINOS CALM (Architecture-as-Code)
**Development Status**: Phase 0 (Foundation Bootstrap) - ready for execution; Phases 1-10 in detailed planning

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

## Development Workflows

### Phase-Based TDD Implementation

The project follows a strict **10-phase roadmap** (`docs/plans/INDEX.md`) with rigorous TDD cycles:

**Phase Execution Pattern** (RED → GREEN → REFACTOR → VALIDATE):

1. **RED**: Write failing tests first (property-based with `proptest` for Rust)
2. **GREEN**: Minimal implementation to pass tests
3. **REFACTOR**: Improve code quality while maintaining green tests
4. **VALIDATE**: Full regression suite + cross-language equivalence checks

**Current Phase**: Phase 0 (Foundation Bootstrap) - establishes Rust workspace with UUID infrastructure
**Next Phases**: Sequential through Phase 6 (Parser), then parallel language bindings (7-9)

### Key Commands (When Implementation Starts)

```bash
# Rust core development
cargo build --release              # Build optimized binary
cargo test --all-features          # Run all tests including property-based
cargo doc --no-deps --open         # Generate and view documentation

# Python bindings (after Phase 7)
maturin develop                    # Development install in current venv
maturin build --release            # Build production wheels

# TypeScript bindings (after Phase 8)
npm run build                      # Compile napi-rs bindings
npm test                           # Run Jest test suite

# WebAssembly (after Phase 9)
wasm-pack build --target web       # Browser-compatible WASM
```

### Traceability Requirements

**Every code change must link to specifications**:

- Commit messages reference ADR/PRD/SDS IDs (e.g., `feat: Entity primitive [ADR-005, SDS-002]`)
- Test names include spec references (e.g., `test_entity_uuid_uniqueness_sds_002()`)
- Diagrams include mandatory YAML front matter with `implements_adrs`, `satisfies_requirements`

### Documentation Updates

When adding features:

1. Update specification first (`docs/specs/prd.md`, `docs/specs/adr.md`, or `docs/specs/sds.md`)
2. Create/update diagrams in `docs/diagrams/{prd|ard|sds}/` with traceability metadata
3. Update `docs/specs/traceability-matrix.md` bidirectional mappings
4. Add implementation plan to `docs/plans/` following PLAN_TEMPLATE.md format

## Project-Specific Conventions

### Naming Patterns

**Rust**:

- Methods: `snake_case` (e.g., `entity_count()`, `flow_count()`) - napi-rs auto-converts to `camelCase` for TypeScript
- Structs: `PascalCase` (e.g., `Entity`, `PolicyEvaluator`)
- Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_NODES`)

**Python** (via PyO3):

- Return native Python objects (dataclasses), not Rust-wrapped types
- Use `Optional[T]` for nullable fields, not `Option<T>`
- Exceptions: `ValidationError` (policy violations), `ValueError` (invalid input)

**TypeScript** (via napi-rs):

- Interfaces follow idiomatic `camelCase` (auto-generated from Rust `snake_case`)
- Use `undefined` for optional fields, not `null`
- Errors: native `Error` subclasses with diagnostic context

### Performance Budgets (Non-Negotiable)

From technical specifications (`docs/specs/technical-specifications.md`):

- Model validation: **<100ms** for 10,000 node graphs (PERF-001)
- FFI overhead: **<1ms** per call amortized (PERF-002)
- WASM bundle: **<500KB** gzipped (INT-003)
- Memory efficiency: Graph storage **<100 bytes** per node (PERF-003)

### Testing Strategy

**Rust Core**:

- Unit tests: One test file per source file (`entity.rs` → `entity_tests.rs`)
- Property-based: Use `proptest` for UUID uniqueness, graph invariants
- Benchmarks: Criterion.rs for performance validation

**Cross-Language Consistency** (Critical):

- Identical test cases across Rust/Python/TypeScript/WASM
- Reference implementation: TypeScript Ohm-JS parser (`context/UBM_mine.md`)
- Rust port must match TypeScript semantics exactly - byte-for-byte output equivalence

### Lockstep Versioning (ADR-008)

- Single `VERSION` file at repository root drives all packages
- All `Cargo.toml`, `pyproject.toml`, `package.json` read from VERSION
- CI enforces consistency - cannot publish if versions drift
- Publish simultaneously to crates.io, PyPI, npm

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

## Code Patterns & Anti-Patterns

### ✅ DO

- **Start with documentation updates** - this is a docs-first project; update specs before implementation
- **Maintain traceability metadata** - every diagram needs YAML front matter linking to ADR/PRD/SDS items
- **Preserve Ohm-JS reference semantics** - when implementing parsers, match TypeScript behavior exactly
- **Use native types at FFI boundaries** - Python dicts/dataclasses, not Rust structs; TypeScript interfaces, not Rust enums
- **Include acceptance criteria** - use EARS format (When/If/Then) in requirements
- **Write tests first** - RED phase before GREEN; use property-based tests for invariants
- **Reference specifications in code** - test names like `test_entity_uuid_uniqueness_sds_002()`

### ❌ DON'T

- **Skip traceability front matter** - all diagrams must have `implements_adrs` and `satisfies_requirements` metadata
- **Create upward dependencies** - Vocabulary layer can't depend on Rules; see three-layer architecture (ADR-001)
- **Expose Rust-specific idioms** - Python users expect Pythonic APIs, not `Result<T, E>`
- **Add primitives beyond the five** - Entity, Resource, Flow, Instance, Policy are universal (ADR-005)
- **Break lockstep versioning** - all language packages must share the same version number
- **Assume grammar semantics** - always validate against TypeScript Ohm-JS reference implementation
- **Optimize prematurely** - meet performance budgets, but prioritize correctness and developer experience first

### Common Mistakes to Avoid

**Diagram Limits**: Mermaid diagrams max 20 nodes due to rendering constraints - use PlantUML `!include` for larger diagrams
**Phase Dependencies**: Phases 0-6 must run sequentially; only phases 7-9 (language bindings) can parallelize
**FFI Complexity**: Don't expose Rust lifetimes to Python/TypeScript - use owned types and clone when necessary
**Test Isolation**: Each phase's tests must be independent - no shared state between test suites

## MCP Tool Integration Workflows

This project uses a **distributed intelligence mesh** of MCP tools for implementation planning and quality assurance. See `.github/chatmodes/tdd.vibepro.chatmode.md` for the complete cognitive architecture.

### Pre-Planning Intelligence Gathering (Mandatory Before Implementation)

Execute this sequence before starting any new phase or feature:

1. **Memory Recall** (`memory`) - Retrieve similar projects, past decisions, known pitfalls
2. **Repository Context** (`github` + `nx`) - Analyze repo structure, active branches, CI patterns
3. **Domain Grounding** (`context7` + `microsoft-docs` + `ref`) - Resolve libraries/frameworks to latest docs
4. **Pattern Research** (`exa` + `github`) - Search for TDD patterns and implementation examples
5. **Metacognitive Checkpoint** (`vibe-check`) - Question assumptions and identify blind spots

Output: Create `docs/plans/<plan_name>/PHASE-000-INTELLIGENCE.md` with all findings.

### Documentation-Grounded Specification Pattern

When authoring specs or architectural decisions:

1. `context7` - Fetch official framework/library docs
2. `microsoft-docs` - Retrieve Azure/Microsoft standards if applicable
3. `ref` - Search for API references and usage patterns
4. `exa` - Find 3-5 real-world implementation examples
5. `vibe-check` - Validate: "Does this specification have hidden complexity?"
6. `memory` - Record specification + rationale for future reference

### Cross-Language Consistency Validation

For FFI boundaries and language bindings:

1. `context7` - Get latest PyO3/napi-rs/wit-bindgen documentation
2. `ref` - Analyze existing FFI patterns in codebase
3. `exa` - Search: "<language> FFI idiomatic patterns"
4. `github` - Review FFI-related issues in similar projects
5. `vibe-check` - Question: "What will break when users misuse this API?"

**Critical**: TypeScript Ohm-JS parser (`context/UBM_mine.md`) is the semantic oracle - Rust must match exactly.

## Autonomous Agent Production Guidelines

### Quality Assurance Checklist (Execute Before Every Commit)

**Pre-Commit Validation** (Non-Negotiable):

```bash
# 1. Specification Traceability
- [ ] Every change links to ADR/PRD/SDS in commit message
- [ ] Test names include spec references (e.g., test_entity_uuid_sds_002)
- [ ] Diagrams include YAML front matter with implements_adrs/satisfies_requirements

# 2. Code Quality Gates
- [ ] cargo fmt --check (Rust formatting)
- [ ] cargo clippy -- -D warnings (no linter warnings)
- [ ] cargo test --all-features (all tests pass)
- [ ] proptest property-based tests included for invariants

# 3. Cross-Language Consistency (if FFI code)
- [ ] Python bindings return native dataclasses, not Rust wrappers
- [ ] TypeScript uses camelCase (auto-converted from snake_case)
- [ ] Identical test case in all language bindings
- [ ] Error messages are native to target language

# 4. Documentation Sync
- [ ] rustdoc comments on all public APIs
- [ ] Update README.md examples if API changed
- [ ] Add entry to CHANGELOG.md (unreleased section)
- [ ] Update traceability matrix if new spec items
```

### Critical Path Invariants (Never Violate)

**Architecture Constraints**:

1. **No Upward Dependencies**: Vocabulary → Fact Model → Rules (ADR-001)

   - Violation Example: `Policy` referencing another `Policy` directly
   - Correct: Policies reference shared `Entity`/`Resource` primitives

2. **Five Primitives Only**: Entity, Resource, Flow, Instance, Policy (ADR-005)

   - If you think you need a 6th primitive, you misunderstood the requirement
   - Extensibility happens through attributes, not new primitive types

3. **Reference Implementation Semantics**: Rust must match TypeScript Ohm-JS exactly
   - Test Strategy: Export same DSL from TypeScript and Rust, diff the ASTs
   - Failure Mode: Different precedence/associativity → silent semantic drift

**Performance Constraints** (Must Be Measurable):

```rust
// Example: Benchmark required for every graph operation
#[bench]
fn bench_validate_10k_entities(b: &mut Bencher) {
    let graph = generate_graph_with_entities(10_000);
    b.iter(|| {
        let start = Instant::now();
        let result = graph.validate();
        assert!(start.elapsed() < Duration::from_millis(100)); // PERF-001
        result
    });
}
```

### Phase Execution Protocol

**Starting a New Phase**:

1. Read `docs/plans/Phase X: <Name>.md` completely
2. Execute MCP intelligence gathering (create PHASE-000-INTELLIGENCE.md)
3. Create feature branch: `feat/phaseX-<description>`
4. Initialize cycle checklist tracking
5. Set up phase-specific test harness

**During Phase Execution**:

```yaml
For Each Cycle (A, B, C, etc.):
  RED Phase:
    - Write failing tests first (TDD)
    - Document expected behavior in test docstrings
    - Link test to SDS/PRD requirement in test name
    - Verify test actually fails for right reason

  GREEN Phase:
    - Minimal implementation to pass tests
    - No premature optimization
    - No "while we're here" scope creep
    - Single responsibility per commit

  REFACTOR Phase:
    - Extract common patterns
    - Improve naming clarity
    - Add performance benchmarks
    - Maintain 100% test pass rate

  VALIDATE Phase:
    - Run full regression suite
    - Check cross-language consistency (if FFI)
    - Update documentation
    - Mark cycle complete with evidence
```

**Completing a Phase**:

1. All cycle checklists marked complete
2. Phase-level integration tests pass
3. Benchmark results meet performance budgets
4. Documentation updated and reviewed
5. Create `PHASE-NNN-RETROSPECTIVE.md` with learnings
6. Update `docs/plans/INDEX.md` status

### Error Recovery Patterns

**When Tests Fail Unexpectedly**:

```yaml
Diagnostic Steps: 1. Check if test name matches actual behavior being tested
  2. Verify test isolation (no shared mutable state)
  3. Review recent changes to dependencies
  4. Use vibe-check to question assumptions
  5. Add regression test for the failure case
  6. Document in PHASE-NNN-RETROSPECTIVE.md
```

**When Performance Budgets Miss**:

```yaml
Response Protocol: 1. Profile with cargo flamegraph or criterion
  2. Identify hot path (don't guess)
  3. Check if algorithm is fundamentally wrong
  4. Consider if test is realistic (10K nodes, not 10M)
  5. Document trade-offs in ADR if budget needs revision
  6. Never silently relax performance constraints
```

**When FFI Bindings Behave Differently**:

```yaml
Cross-Language Debugging: 1. Export AST from both languages as JSON
  2. Use diff tool to find semantic divergence
  3. Check TypeScript Ohm-JS grammar for reference semantics
  4. Verify Rust grammar port is exact (not "close enough")
  5. Add equivalence test to prevent regression
  6. Document in SDS-009/010/011 if pattern emerges
```

### Documentation Debt Prevention

**Real-Time Documentation**:

- Update specs **before** implementation, not after
- Diagrams created **during** design, not retrospectively
- CHANGELOG.md updated **with** each feature commit
- Traceability matrix updated **immediately** when specs change

**Documentation Quality Signals**:

```yaml
Good Documentation:
  - Specific examples from actual codebase
  - Links to related ADR/PRD/SDS items
  - Explains "why" not just "what"
  - Includes failure modes and mitigations
  - Updated date stamp

Poor Documentation:
  - Generic advice ("write good code")
  - No traceability links
  - Outdated examples
  - Missing performance implications
  - No examples of what NOT to do
```

### Continuous Integration Expectations

**CI Must Validate**:

1. All language targets build successfully (Rust, Python, TypeScript, WASM)
2. Test coverage remains above 80% (enforced by tooling)
3. No clippy warnings in Rust code
4. Type checking passes in TypeScript
5. Documentation builds without warnings
6. Benchmark results within performance budgets
7. Cross-language equivalence tests pass

**CI Failure Response**:

- Never commit with "fix CI later" attitude
- Investigate root cause immediately
- Add regression test if CI caught real bug
- Update CI configuration if false positive
- Document in retrospective if systemic issue

### Code Review Protocol (For Human Oversight)

**Self-Review Before Requesting**:

1. Re-read your own changes with fresh eyes
2. Check every TODO/FIXME has a tracking issue
3. Verify no debug print statements left in
4. Confirm no commented-out code blocks
5. Run full test suite locally, not just affected tests

**What Reviewers Will Check**:

- Traceability to specifications
- Test coverage of new code paths
- Performance implications documented
- Error handling is language-idiomatic
- No violation of architectural constraints
- Documentation updated appropriately

## Quick Reference

| Document                            | Purpose                 | Key Sections                                               |
| ----------------------------------- | ----------------------- | ---------------------------------------------------------- |
| `context/DSL_instructions.md`       | API specification guide | Sections 1-10: Rust Core, Python/TS/WASM APIs, Testing, DX |
| `docs/specs/prd.md`                 | Product requirements    | PRD-001 (Layering), PRD-003 (SBVR), PRD-005-008 (FFI)      |
| `docs/specs/adr.md`                 | Architectural decisions | ADR-001 (Layers), ADR-002 (FFI), ADR-007 (Idioms)          |
| `docs/specs/sds.md`                 | Design specifications   | SDS-006 (Policy Engine), SDS-008-011 (Bindings)            |
| `docs/specs/traceability-matrix.md` | Impact analysis         | ADR→PRD→SDS→TechSpec mappings                              |

## Specification Language Patterns

### EARS Format for Requirements (PRD)

All product requirements use **EARS (Easy Approach to Requirements Syntax)**:

```
Pattern: <Trigger> <Action> <Constraint>

Examples:
✅ GOOD: "When a Policy expression is evaluated, the system shall return results within 100ms for graphs with up to 10,000 nodes"
✅ GOOD: "If a Flow references a non-existent Resource, the parser shall raise a ValidationError with the Resource name and line number"

❌ BAD: "The system should be fast" (not measurable)
❌ BAD: "Policies must work correctly" (not specific)
```

**EARS Templates**:

- **Ubiquitous**: "The system shall <action>"
- **Event-driven**: "When <trigger>, the system shall <action>"
- **Unwanted behavior**: "If <condition>, the system shall <action>"
- **State-driven**: "While <state>, the system shall <action>"
- **Optional**: "Where <feature is included>, the system shall <action>"

### ADR YAML Metadata (Required Fields)

Every ADR must include complete YAML front matter:

```yaml
adr_id: ADR-XXX                    # Sequential numbering
title: "Descriptive Title"         # Use Title Case
status: Accepted|Proposed|Deprecated
date: YYYY-MM-DD                   # ISO 8601 format
authors:
  - name: "Author Name"
    role: "Role"
stakeholders: ["Team A", "Team B"] # Who is affected
traceability:
  supersedes: ADR-XXX|null         # What this replaces
  related: ["ADR-YYY", "ADR-ZZZ"]  # Related decisions
mvp_scope: "[MVP]"|"[POST-MVP]"    # Delivery timeline
```

### Diagram Front Matter (Mandatory)

**Every diagram file must start with**:

```yaml
---
adr: ADR-002 # Primary ADR
diagram_type: C4Component|C4Context|Flow|Sequence
implements_adrs: ["ADR-002", "ADR-007"] # All related ADRs
satisfies_requirements: ["PRD-005", "PRD-015"] # Requirements met
generated: 2025-11-07 # Creation date
---
```

**Violation**: Diagrams without front matter will fail CI validation.

## Critical Code Patterns

### Rust Core Patterns

**Graph Structure** (ADR-003, SDS-005):

```rust
// Adjacency list representation for performance
pub struct Graph {
    // Node storage (owned)
    entities: HashMap<Uuid, Entity>,
    resources: HashMap<Uuid, Resource>,
    flows: HashMap<Uuid, Flow>,

    // Edge indexes (borrowed references)
    flow_to_resource: HashMap<Uuid, Uuid>,  // flow_id -> resource_id
    flow_from_entity: HashMap<Uuid, Uuid>,  // flow_id -> from_entity_id
    flow_to_entity: HashMap<Uuid, Uuid>,    // flow_id -> to_entity_id

    // Reverse indexes for traversal
    entity_outgoing_flows: HashMap<Uuid, Vec<Uuid>>,
    entity_incoming_flows: HashMap<Uuid, Vec<Uuid>>,
}

impl Graph {
    // O(1) lookup, not O(n) scan
    pub fn flows_from(&self, entity_id: &Uuid) -> Vec<&Flow> {
        self.entity_outgoing_flows
            .get(entity_id)
            .map(|flow_ids| flow_ids.iter().filter_map(|id| self.flows.get(id)).collect())
            .unwrap_or_default()
    }
}
```

**Policy Evaluator** (ADR-004, SDS-006):

```rust
// SBVR-aligned expression evaluation
pub enum Expression {
    // Quantifiers: forall, exists
    Quantifier {
        kind: Quantifier,
        variable: String,
        range: Box<Expression>,    // Collection to iterate
        body: Box<Expression>,     // Predicate to evaluate
    },

    // Binary operations: AND, OR, >, <, ==, etc.
    BinaryOp {
        op: BinaryOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },

    // Unary operations: NOT, negation
    Unary {
        op: UnaryOp,
        operand: Box<Expression>,  // Note: "operand" not "expr"
    },

    // Literals and variables
    Literal(Value),
    Variable(String),
}

impl Expression {
    pub fn evaluate(&self, context: &Context) -> Result<bool, EvalError> {
        match self {
            Self::Quantifier { kind: Quantifier::ForAll, variable, range, body } => {
                let collection = range.evaluate_to_collection(context)?;
                for item in collection {
                    let mut scoped_context = context.clone();
                    scoped_context.bind(variable, item);
                    if !body.evaluate(&scoped_context)? {
                        return Ok(false); // ForAll fails if any item fails
                    }
                }
                Ok(true)
            }
            // ... other cases
        }
    }
}
```

### Python FFI Patterns (PyO3)

**Native Python Objects** (ADR-007, SDS-009):

```python
# GOOD: Python dataclass with Rust backend
from dataclasses import dataclass
from typing import Optional, Dict, Any
from uuid import UUID

@dataclass
class Entity:
    """Business actor or location in the domain model."""
    id: UUID
    name: str
    namespace: Optional[str] = None
    attributes: Dict[str, Any] = None

    def __post_init__(self):
        if self.attributes is None:
            self.attributes = {}

    @classmethod
    def from_rust(cls, rust_entity):
        """Convert Rust entity to Python dataclass."""
        return cls(
            id=UUID(rust_entity.id),
            name=rust_entity.name,
            namespace=rust_entity.namespace,
            attributes=rust_entity.attributes or {}
        )

# BAD: Exposing Rust wrapper (don't do this)
class Entity:
    def __init__(self, rust_handle):
        self._handle = rust_handle  # User sees Rust internals ❌
```

**Error Handling** (SDS-009, PRD-017):

```python
# GOOD: Native Python exceptions
class ValidationError(ValueError):
    """Raised when policy validation fails."""
    def __init__(self, message: str, violations: list[dict]):
        super().__init__(message)
        self.violations = violations  # Structured diagnostic data

# Usage in Rust FFI:
#[pyfunction]
fn validate_model(py: Python, model: PyObject) -> PyResult<bool> {
    match rust_core::validate(&model) {
        Ok(true) => Ok(true),
        Ok(false) => Ok(false),
        Err(e) => {
            let violations = e.violations.iter()
                .map(|v| v.to_py_dict(py))
                .collect();
            Err(PyErr::new::<ValidationError, _>((e.message, violations)))
        }
    }
}
```

### TypeScript FFI Patterns (napi-rs)

**Idiomatic TypeScript** (ADR-007, SDS-010):

```typescript
// GOOD: TypeScript interface with native types
interface Entity {
  id: string;              // UUID as string
  name: string;
  namespace?: string;      // Optional with undefined
  attributes?: Record<string, unknown>;
}

interface Graph {
  entityCount(): number;           // camelCase (auto-converted)
  flowCount(): number;
  flowsFrom(entityId: string): Flow[];
  validate(): ValidationResult;
}

// Rust implementation uses snake_case
#[napi]
impl Graph {
    #[napi]
    pub fn entity_count(&self) -> u32 {  // → entityCount() in TypeScript
        self.inner.entity_count() as u32
    }

    #[napi]
    pub fn flow_count(&self) -> u32 {    // → flowCount() in TypeScript
        self.inner.flow_count() as u32
    }

    #[napi]
    pub fn flows_from(&self, entity_id: String) -> Result<Vec<Flow>> {
        let uuid = Uuid::parse_str(&entity_id)
            .map_err(|e| napi::Error::from_reason(format!("Invalid UUID: {}", e)))?;
        Ok(self.inner.flows_from(&uuid).into_iter().map(Flow::from).collect())
    }
}
```

### WASM Patterns (wit-bindgen)

**Browser-Compatible Types** (SDS-011, INT-003):

```rust
// WIT interface definition
interface {
    record entity {
        id: string,
        name: string,
        namespace: option<string>,
    }

    validate-model: func(entities: list<entity>) -> result<bool, string>
}

// Rust implementation
#[wit_bindgen::generate!("sea-dsl")]
impl Guest for SeaDsl {
    fn validate_model(entities: Vec<Entity>) -> Result<bool, String> {
        // Keep memory footprint small for WASM
        // Target: <500KB gzipped (INT-003)
        let graph = Graph::from_entities(entities);
        graph.validate()
            .map_err(|e| e.to_string())
    }
}
```

## Anti-Patterns to Avoid

### Architecture Violations

**❌ Upward Dependencies**:

```rust
// BAD: Policy depends on another Policy
pub struct Policy {
    depends_on: Vec<PolicyId>,  // ❌ Violates ADR-001
}

// GOOD: Policies reference shared primitives
pub struct Policy {
    expression: Expression,      // ✅ References Entity/Resource via expressions
    scope: Vec<EntityId>,        // ✅ Declares which entities this constrains
}
```

**❌ Adding Sixth Primitive**:

```rust
// BAD: Inventing new primitive type
pub enum Primitive {
    Entity(Entity),
    Resource(Resource),
    Flow(Flow),
    Instance(Instance),
    Policy(Policy),
    Process(Process),  // ❌ Violates ADR-005 five-primitive constraint
}

// GOOD: Extend through attributes
pub struct Flow {
    // ... standard fields
    attributes: HashMap<String, AttributeValue>,  // ✅ Process is an attribute pattern
}
```

### FFI Anti-Patterns

**❌ Exposing Rust Idioms to Python**:

```python
# BAD: Forcing Python users to handle Result<T, E>
result = graph.validate()  # Returns RustResult
if result.is_ok():
    value = result.unwrap()  # ❌ Not Pythonic

# GOOD: Native Python exceptions
try:
    graph.validate()  # Raises ValidationError on failure
except ValidationError as e:
    print(e.violations)  # ✅ Pythonic error handling
```

**❌ Memory Management in Language Bindings**:

```rust
// BAD: Exposing lifetimes to FFI
#[pyclass]
pub struct Entity<'a> {  // ❌ Python can't handle lifetimes
    graph: &'a Graph,
}

// GOOD: Use owned types
#[pyclass]
pub struct Entity {
    id: Uuid,
    name: String,  // ✅ Owned, no lifetime issues
}
```

### Testing Anti-Patterns

**❌ Flaky Tests**:

```rust
// BAD: Time-dependent test
#[test]
fn test_validation_performance() {
    let graph = create_graph();
    let start = Instant::now();
    graph.validate();
    assert!(start.elapsed() < Duration::from_millis(100));  // ❌ Flaky on slow CI
}

// GOOD: Use criterion benchmarks
#[bench]
fn bench_validation(b: &mut Bencher) {
    let graph = create_graph();
    b.iter(|| graph.validate());  // ✅ Statistical analysis of performance
}
```

**❌ Shared Mutable State**:

```rust
// BAD: Global state between tests
static mut COUNTER: i32 = 0;  // ❌ Tests interfere with each other

#[test]
fn test_a() {
    unsafe { COUNTER += 1; }
}

// GOOD: Isolated test data
#[test]
fn test_a() {
    let graph = Graph::new();  // ✅ Fresh state per test
    // ... test logic
}
```

## Emergency Procedures

### When Blocked on External Dependency

```yaml
Protocol: 1. Document blocker in GitHub issue with "blocked" label
  2. Identify workaround or stub implementation
  3. Add TODO comment with issue reference
  4. Continue with other parallel work
  5. Update phase checklist with blocked status
  6. Set up notification for dependency resolution
```

### When Discovering Architectural Flaw

```yaml
Protocol: 1. STOP implementation immediately
  2. Document flaw in vibe-check session
  3. Propose ADR amendment or new ADR
  4. Get stakeholder review (create RFC issue)
  5. Update affected phase plans
  6. Communicate impact to dependent phases
```

### When Performance Budget Cannot Be Met

```yaml
Protocol: 1. Profile to identify bottleneck (cargo flamegraph)
  2. Verify budget is realistic (not artificial constraint)
  3. Consider algorithmic alternatives (O(n²) → O(n log n))
  4. Document trade-offs in ADR amendment
  5. Get approval to adjust budget if necessary
  6. Add regression test to prevent future degradation
```

---

**Last Updated**: 2025-11-07
**Project Phase**: Documentation & Architectural Design (Phase 0 Ready)
**Next Milestone**: Phase 0 execution - Rust workspace + UUID infrastructure
**Contact**: See `docs/plans/INDEX.md` for phase coordination details

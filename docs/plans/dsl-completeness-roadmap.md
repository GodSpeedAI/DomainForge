# DSL Completeness Roadmap

## Overview

This roadmap defines the path to achieving **semantic completeness** and **production-grade ergonomics** for the **DomainForge DSL** (Semantic Event Architecture DSL). The DSL is the core language for modeling business domains using entities, resources, flows, and policies, which are then projected into various target formats (CALM, Knowledge Graphs, SBVR, code).

### Project Context

**DomainForge** is a semantic modeling system that:

- Uses a custom DSL (parsed via Pest grammar in `sea-core/grammar/sea.pest`)
- Generates a Knowledge Graph Store (KGS) from DSL models
- Projects KGS into multiple formats (CALM, RDF/Turtle, SBVR)
- Integrates with **VibesPro** (code generator) and **GovernedSpeed** (governance engine)
- Supports **Temporal DB** for pattern learning and semantic memory

### Current State

The DSL currently supports:

- **Entities** and **Resources** with domain scoping
- **Flows** (resource movements between entities)
- **Policies** (constraints, obligations, prohibitions)
- **Units** and **Dimensions** (basic support)
- **File-level metadata** (`@namespace`, `@version`, `@owner`)
- **Aggregations** (`sum`, `count`, `avg`, `min`, `max`)
- **Quantifiers** (`forall`, `exists`, `exists_unique`)

### Missing Features

This roadmap addresses gaps identified in `tmp/polish-check.md`, organized into 7 phases:

1. **Foundation** (CRITICAL): Temporal semantics, pattern matching
2. **Core Features** (HIGH): Roles, relations, versioning, instances
3. **Ergonomics** (IMPORTANT): Observability, projections, modules
4. **Polish** (MEDIUM): String operations, indexing
5. **Semantics & Safety** (CRITICAL): Determinism, null handling, Unicode
6. **Ecosystem Integration**: Temporal DB, governance, simulation
7. **Semantic Control Plane**: Boolean-Semantic Framework for generation control

### Implementation Approach

For each feature:

1. **Grammar**: Update `sea-core/grammar/sea.pest`
2. **AST**: Add nodes in `sea-core/src/ast.rs`
3. **Parser**: Update `sea-core/src/parser.rs`
4. **Semantics**: Add validation/evaluation logic
5. **Projections**: Update CALM, KG, SBVR generators
6. **Tests**: Add golden tests in `sea-core/tests/`
7. **Bindings**: Update Python, TypeScript, and WASM bindings

### Language Binding Updates

When adding new DSL features, bindings must be updated to expose functionality:

#### Python Bindings (`sea-core/src/python/`)

- **Entry Point**: `sea-core/src/python/lib.rs` - PyO3 module definition
- **API Wrappers**: `sea-core/src/python/*.rs` - Wrap Rust types for Python
- **Update Steps**:
  1. Add `#[pyclass]` wrapper for new AST nodes (if exposing to Python)
  2. Add `#[pymethods]` for new operations (parse, validate, project)
  3. Update `sea-core/src/python/lib.rs` to register new classes/functions
  4. Add Python tests in `tests/test_*.py`
  5. Update type stubs in `python/sea_core/__init__.pyi` (if exists)

#### TypeScript Bindings (`sea-core/src/typescript/`)

- **Entry Point**: `sea-core/src/typescript/lib.rs` - NAPI-RS module definition
- **API Wrappers**: `sea-core/src/typescript/*.rs` - Wrap Rust types for Node.js
- **Update Steps**:
  1. Add `#[napi]` wrapper for new AST nodes
  2. Add `#[napi]` functions for new operations
  3. Update `sea-core/src/typescript/lib.rs` to export new items
  4. Run `npm run build` to regenerate `index.d.ts` type definitions
  5. Add TypeScript tests in `typescript-tests/*.test.ts`
  6. Update `index.d.ts` manually if auto-generation is incomplete

#### WASM Bindings (`sea-core/src/wasm/`)

- **Entry Point**: `sea-core/src/wasm/lib.rs` - wasm-bindgen module definition
- **API Wrappers**: `sea-core/src/wasm/*.rs` - Wrap Rust types for WebAssembly
- **Update Steps**:
  1. Add `#[wasm_bindgen]` wrapper for new AST nodes
  2. Add `#[wasm_bindgen]` functions for new operations
  3. Use `JsValue` for complex types (serialize via serde-wasm-bindgen)
  4. Update `sea-core/src/wasm/lib.rs` to export new items
  5. Run `just build-wasm` to regenerate WASM artifacts
  6. Add WASM tests in `sea-core/tests/wasm_*.rs`
  7. Update `pkg/package.json` and `pkg/*.d.ts` if needed

#### Binding Update Checklist

For each new feature, ensure:

- [ ] Python: `#[pyclass]` and `#[pymethods]` added
- [ ] TypeScript: `#[napi]` annotations added, `index.d.ts` regenerated
- [ ] WASM: `#[wasm_bindgen]` annotations added, `pkg/` updated
- [ ] All bindings: Error types properly converted (PyErr, napi::Error, JsValue)
- [ ] All bindings: Tests added for new functionality
- [ ] All bindings: Documentation comments (`///`) added for public API

#### CLI Updates (`sea-core/src/cli/` and `sea-core/src/bin/sea.rs`)

- **Entry Point**: `sea-core/src/bin/sea.rs` - Main CLI entry using clap
- **Command Modules**: `sea-core/src/cli/*.rs` - Individual command implementations
- **Update Steps**:
  1. Add new command enum variant in `sea-core/src/cli/mod.rs` (e.g., `Commands::NewFeature`)
  2. Create new command module `sea-core/src/cli/new_feature.rs` with `run()` function
  3. Add command handler in `sea-core/src/bin/sea.rs` match statement
  4. Define clap args struct with `#[derive(Parser)]` in command module
  5. Add CLI tests in `sea-core/tests/cli_*.rs`
  6. Update `justfile` with new recipe if needed (e.g., `just new-feature`)
  7. Update CLI documentation in `README.md` or `docs/`

**Existing CLI Commands**:

- `validate` - Validate DSL files
- `import` - Import from external formats
- `project` - Project to CALM/KG/SBVR
- `format` - Format DSL files
- `test` - Run DSL tests
- `validate-kg` - Validate Knowledge Graph

**CLI Update Checklist**:

- [ ] Command enum variant added to `Commands`
- [ ] Command module created with `run()` function
- [ ] Args struct defined with clap attributes
- [ ] Handler added to `sea.rs` match statement
- [ ] CLI tests added
- [ ] Help text and documentation updated

## Phase 1: Foundation (CRITICAL)

These features are essential for the core expressiveness of the language.

### 1. Temporal Semantics

**Problem**: No way to express time-bound logic or intervals.

**Syntax Example**:

```sea
// Time literals
Policy payment_deadline as:
  forall f in flows where f.resource = "Invoice":
    f.created_at before "2025-12-31T23:59:59Z"

// Intervals
Policy business_hours as:
  forall f in flows where f.resource = "Transaction":
    f.timestamp during interval("09:00", "17:00")

// Relative time
Policy sla_compliance as:
  forall f in flows where f.resource = "SupportTicket":
    f.resolved_at - f.created_at <= 24 "hours"
```

**Implementation**:

- [x] **Grammar** (`sea.pest`): Add `time_literal`, `interval_literal`, `temporal_op` rules
- [x] **AST** (`ast.rs`): Add `TimeLiteral(DateTime)`, `IntervalLiteral(Duration)`, `TemporalExpression`
- [x] **Parser** (`parser.rs`): Parse ISO 8601 timestamps and duration strings
- [x] **Semantics** (`validator.rs`): Validate temporal comparisons, ensure type safety
- [x] **Projection** (`calm.rs`, `kg.rs`): Map to temporal properties in target formats
- [x] **Bindings**: Updated Python and TypeScript bindings for temporal operators

**Testing**:

- [x] Golden test: `tests/temporal_semantics_tests.rs` with various time expressions
- [x] Validation test: Ensure type errors for invalid temporal operations
- [x] All 9 temporal semantics tests passing
- [x] Full test suite passing (86 tests)

### 2. Units & Dimensions (Refinement)

**Problem**: While basic support exists, full arithmetic and validation are needed.

**Syntax Example**:

```sea
// Current: Basic unit support exists
Dimension "Currency"
Unit "USD" of "Currency" factor 1 base "USD"
Unit "EUR" of "Currency" factor 1.1 base "USD"

// Enhancement: Strict validation
Policy budget_limit as:
  sum(f in flows where f.resource = "Money": f.quantity as "USD") <= 10_000 "USD"
  // ERROR if trying to add Currency + Mass
  // ERROR if comparing quantities without unit cast
```

**Implementation**:

- [ ] **Validation** (`validator.rs`): Add `DimensionChecker` to verify all arithmetic operations
- [ ] **Semantics** (`evaluator.rs`): Implement unit conversion using factor/base from `UnitDecl`
- [ ] **Type System** (`types.rs`): Separate `Quantity` (with unit) from `Numeric` (dimensionless)
- [ ] **Error Reporting**: Add `UnitError::MismatchedDimension` with helpful hints

**Testing**:

- Test: `sum(Money) + sum(Mass)` should fail with clear error
- Test: `100 "kg" + 500 "g"` should auto-convert to same unit
- Test: Comparison `100 "USD" < 90 "EUR"` requires explicit cast

### 3. Pattern Semantics

**Problem**: Missing pattern matching capabilities for strings or structures.

**Syntax Example**:

```sea
// Pattern declarations
Pattern "EmailAddress" matches "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
Pattern "PhoneNumber" matches "^\\+?[1-9]\\d{1,14}$"

// Usage in policies
Policy valid_contact as:
  forall e in entities where e.type = "Customer":
    e.email matches "EmailAddress" and
    e.phone matches "PhoneNumber"

// Structural patterns (future)
Pattern "ValidAddress" {
  street: String,
  city: String,
  postal_code: matches "^\\d{5}(-\\d{4})?$"
}
```

**Implementation**:

- [x] **Grammar** (`sea.pest`): Add `pattern_decl`, `matches` operator
- [x] **AST** (`ast.rs`): Add `PatternDecl { name, regex }`, `MatchExpression`
- [x] **Parser** (`parser.rs`): Parse regex strings, validate regex syntax
- [x] **Semantics** (`validator.rs`): Compile regex patterns, cache for performance
- [x] **Runtime** (`evaluator.rs`): Implement pattern matching using `regex` crate
- [x] **Projections**: Patterns exported to CALM, KG (RDF/Turtle), and SBVR
- [x] **Bindings**: Pattern counts exposed through Python, TypeScript, and WASM APIs

**Testing**:

- [x] Test: Valid/invalid email addresses against pattern
- [x] Test: Pattern reuse across multiple policies
- [x] Test: Error on invalid regex syntax
- [x] All tests passing (87 Rust tests, 43 Python tests, 57 TypeScript tests)

## Phase 2: Core Features (HIGH)

These features significantly expand the modeling capabilities.

### 4. Enhanced Aggregations

**Problem**: `group_by` and window functions are missing.

**Syntax Example**:

```sea
// Group by with aggregation
Policy vendor_spending as:
  group_by(f in flows where f.resource = "Money": f.to.name) {
    sum(f.quantity as "USD") <= 50_000 "USD"
  }

// Window functions (sliding window)
Policy throughput_limit as:
  avg(f in flows over last 1 "hour": f.quantity) <= 1000
```

**Implementation**:

- [ ] **Grammar** (`sea.pest`): Add `group_by` clause, `over` window syntax
- [ ] **AST** (`ast.rs`): Extend `AggregationExpression` with `GroupBy { key, aggregation }`
- [ ] **Semantics** (`evaluator.rs`): Implement grouping with deterministic ordering
- [ ] **Validation**: Ensure grouped expressions only reference group key or aggregates
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

**Testing**:

- Test: Group flows by entity, verify sum per group
- Test: Window aggregation over time-series data

### 5. Roles & Relations (Fact Types)

**Problem**: No way to define roles or typed relationships beyond flows.

**Syntax Example**:

```sea
// Role declarations
Role "Payer"
Role "Payee"
Role "Approver"

// Relation types (SBVR-compatible)
Relation "Payment"
  subject: "Payer"
  predicate: "pays"
  object: "Payee"
  via: flow "Money"

// Usage in policies
Policy approval_required as:
  forall r in relations where r.type = "Payment" and r.amount > 10_000 "USD":
    exists a in entities where a has_role "Approver":
      a approves r
```

**Implementation**:

- [ ] **Grammar** (`sea.pest`): Add `role_decl`, `relation_decl`
- [ ] **AST** (`ast.rs`): Add `RoleDecl`, `RelationDecl { subject, predicate, object, via }`
- [ ] **KG Projection** (`kg.rs`): Map to RDF triples with typed predicates
- [ ] **SBVR Projection** (`sbvr.rs`): Generate fact type definitions
- [ ] **Semantics** (`validator.rs`): Validate role assignments and relation integrity
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

**Testing**:

- Test: Define payment relation, query by role
- Golden test: Relation → KG → validate SHACL constraints

### 6. Evolution Semantics

**Problem**: No support for versioning or migration.

**Syntax Example**:

```sea
Entity "Vendor" v2.1.0
  @replaces "Vendor" v2.0.0
  @changes ["added credit_limit field", "removed legacy_id"]
  in "procurement"

// Concept change tracking
ConceptChange "Vendor_v2_migration"
  @from_version "2.0.0"
  @to_version "2.1.0"
  @migration_policy mandatory
  @breaking_change true
```

**Implementation**:

- [ ] **Grammar** (`sea.pest`): Add version suffix `v<semver>`, `@replaces`, `@changes` annotations
- [ ] **AST** (`ast.rs`): Add `ConceptChangeDecl`, version metadata to all declaration nodes
- [ ] **Semantics** (`validator.rs`): Implement version resolution, detect semantic drift
- [ ] **KGS**: Store version history, enable temporal queries
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

**Testing**:

- Test: Load v2.0.0 and v2.1.0 side-by-side, verify migration path
- Test: Detect breaking changes automatically

### 7. Instance Declarations

**Problem**: Cannot define concrete data instances in DSL. (verify the problem is still valid)

**Syntax Example**:

```sea
Instance vendor_123 of "Vendor"
  name: "Acme Corp"
  credit_limit: 50_000 "USD"
  @tags ["preferred", "enterprise"]

// Reference instances in policies
Policy vendor_specific as:
  flows where .to == @vendor_123
```

**Implementation**:

- [ ] **Grammar** (`sea.pest`): Add `instance_decl`, instance reference syntax `@instance_id`
- [ ] **AST** (`ast.rs`): Add `InstanceDecl { id, entity_type, fields }`
- [ ] **Semantics** (`validator.rs`): Validate instances against entity schemas
- [ ] **KGS**: Store instances as nodes with `rdf:type` pointing to entity class
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

**Testing**:

- Test: Create instance, validate field types match entity
- Test: Reference instance in policy expression

## Phase 3: Ergonomics (IMPORTANT)

These features improve the developer experience and safety.

### 8. Observability Semantics

**Problem**: Cannot express metrics or monitoring constraints.
**Implementation**:

- [ ] **Grammar**: Add `Metric` declaration.
- [ ] **AST**: Add `MetricDecl`.
- [ ] **Semantics**: Bind metrics to external signals or internal aggregations.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

### 9. Projection Contracts

**Problem**: No formal mapping specifications.
**Implementation**:

- [ ] **Grammar**: Add syntax for defining mappings (e.g., `mapping <name> for <target>`).
- [ ] **AST**: Add `MappingDecl`.
- [ ] **Semantics**: Implement projection generators based on these contracts.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

### 10. Module System

**Problem**: No `import`/`export` for code organization.
**Implementation**:

- [ ] **Grammar**: Add `import` and `export` keywords.
- [ ] **AST**: Add `ImportDecl`, `ExportDecl`.
- [ ] **Semantics**: Implement module resolution and namespacing.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

### 11. Error Model & Diagnostics

**Problem**: Need structured error reporting.
**Implementation**:

- [x] **Runtime**: Define `SyntaxError`, `TypeError`, `UnitError`, etc. (Implemented via `ValidationError` and `ErrorCode` in `sea-core/src/validation_error.rs`)
- [ ] **Tooling**: Enhance CLI to report errors with source spans and hints.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

## Phase 4: Polish (OPTIONAL)

### 12. String Semantics

**Problem**: Missing case-insensitive operations.
**Implementation**:

- [ ] **Grammar**: Add `icontains`, `istartswith`, `iendswith`.
- [ ] **AST**: Add corresponding operators.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

### 13. Index Semantics

**Problem**: No explicit index queries.
**Implementation**:

- [ ] **Grammar**: Add syntax for querying by tags or metadata.
- [ ] **Semantics**: Optimize queries based on indices.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

## Phase 5: Semantics & Safety (CRITICAL - Ongoing)

These cross-cutting concerns must be addressed throughout all phases.

### 14. Determinism & Iteration Order

**Problem**: Aggregations and iterations must be deterministic.
**Implementation**:

- [x] **Semantics**: Enforce named collection enumeration order (e.g., lexicographical by ID). (Implemented via `IndexMap` in `sea-core/src/graph/mod.rs` and verified in `tests/phase_14_determinism_tests.rs`)
- [ ] **Validation**: Forbid side effects in aggregations.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

### 15. Null Semantics & Type Safety

**Problem**: Underspecified behavior for missing values.
**Implementation**:

- [x] **Design**: Decide between banning nulls (recommended) or strict 3-valued logic. (Implemented strict 3-valued logic in `sea-core/src/policy/three_valued.rs`)
- [x] **Type System**: Separate `Quantity` vs `Numeric`. Enforce explicit unit casts. (Implemented in `sea-core/src/policy/type_inference.rs` with `ExprType` and `TypeError::MixedQuantityNumeric`)
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

### 16. Unicode & Normalization

**Problem**: Identifier consistency across languages.
**Implementation**:

- [ ] **Parser**: Enforce NFC normalization for identifiers.
- [ ] **Semantics**: Define quoting rules for punctuation-heavy names.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

## Phase 6: Ecosystem Integration (Temporal DB)

These features enable deep integration with the Temporal DB and VibesPro ecosystem.

### 17. Governance & Risk Annotations

**Problem**: No way to express risk levels or governance requirements directly in policy.
**Implementation**:

- [ ] **Grammar**: Add `@risk("low"|"medium"|"high"|"critical")` annotation.
- [ ] **Grammar**: Add `@governance_policy` annotation to link to external governance docs.
- [ ] **AST**: Expose these annotations to the projection engine.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

### 18. Architecture & Generator Metadata

**Problem**: Missing context for generator selection (e.g., "hexagonal" vs "script").
**Implementation**:

- [ ] **Grammar**: Add `@architecture_profile` and `@generator_options` to file headers.
- [ ] **Semantics**: Propagate these to the `TemporalContextKey` during generation.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

### 19. Simulation Semantics

**Problem**: Cannot distinguish between "real" and "simulated" logic or define mock data rules.
**Implementation**:

- [ ] **Grammar**: Add `Simulation` block or `@simulated` annotation.
- [ ] **Semantics**: Allow defining probabilistic flows (e.g., `quantity ~ normal(100, 10)`).
- [ ] **Runtime**: Support "simulation mode" execution flag.
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

## Phase 7: Semantic Control Plane (Boolean-Semantic Framework)

These features implement the "Calculus of Semantic Resolution" to control generator behavior and minimize hallucination.

### 20. Semantic Coordinate Annotations

**Problem**: No way to define the "Cognitive State" (Scope/Modality) of a component or prompt.

**Syntax Example**:

```sea
// Annotate entities with semantic coordinates
Entity "PaymentProcessor"
  @scope(0.8)      // Highly specific (0=General, 1=Specific)
  @modality(0.6)   // Mix of abstract + concrete (0=Abstract, 1=Concrete)
  in "payments"

// Maps to Cardinal States:
// @scope(0.0) @modality(0.0) = Axiomatic (Fire)
// @scope(1.0) @modality(0.0) = Schematic (Earth)
// @scope(0.0) @modality(1.0) = Analogical (Air)
// @scope(1.0) @modality(1.0) = Instantiated (Water)
```

**Implementation**:

- [ ] **Grammar** (`sea.pest`): Add `@scope(float)`, `@modality(float)` annotations
- [ ] **AST** (`ast.rs`): Add `SemanticCoordinates { scope: f64, modality: f64 }` to declarations
- [ ] **Validation**: Ensure 0.0 <= scope, modality <= 1.0
- [ ] **Projection**: Include in `TemporalContextKey` for generator queries
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

**Testing**:

- Test: Annotate BC with coordinates, verify propagation to KGS
- Test: Query Temporal DB by semantic distance

### 21. Generation Strategy Definitions

**Problem**: Cannot define the "Orthogonal Path" for generation (Chain of Thought).

**Syntax Example**:

```sea
// Define generation strategy (prevents diagonal jumps)
Strategy "generate_payment_service"
  start: Axiomatic      // (0.0, 0.0) - Define principles
  step: Schematic       // (1.0, 0.0) - Lock in structure (orthogonal: only scope changed)
  step: Instantiated    // (1.0, 1.0) - Generate code (orthogonal: only modality changed)
  @enforce_orthogonal true

// Alternative path (via Analogical)
Strategy "generate_marketing_copy"
  start: Axiomatic      // (0.0, 0.0)
  step: Analogical      // (0.0, 1.0) - Add vibe/persona
  step: Instantiated    // (1.0, 1.0) - Generate specific copy
```

**Implementation**:

- [ ] **Grammar** (`sea.pest`): Add `strategy_decl` with `start`, `step` directives
- [ ] **AST** (`ast.rs`): Add `StrategyDecl { name, steps: Vec<CardinalState> }`
- [ ] **Runtime** (`generator.rs`): Validate transitions are orthogonal (only one coordinate changes per step)
- [ ] **GovernedSpeed Integration**: Reject generation plans with diagonal jumps
- [ ] **Projections**: Ensure patterns are represented in CALM/KG/SBVR
- [ ] **Bindings**: Expose pattern matching to Python and TypeScript APIs

**Testing**:

- Test: Valid orthogonal strategy (Axiomatic → Schematic → Instantiated)
- Test: Invalid diagonal jump (Axiomatic → Instantiated) should error
- Test: Calculate semantic distance between states

**Mathematical Basis**:

- Hallucination risk ∝ ||end - start||² (Euclidean distance)
- Orthogonal transition: Δx=0 OR Δy=0 (one coordinate constant)
- Diagonal transition: Δx≠0 AND Δy≠0 (forbidden)

---

## Implementation Summary

### Recommended Implementation Order

1. **Phase 5 First** (Semantics & Safety): Establish determinism, type safety, and Unicode handling as foundation
2. **Phase 1** (Foundation): Add temporal, pattern, and unit refinements
3. **Phase 2** (Core Features): Implement roles, relations, versioning, instances
4. **Phase 6** (Ecosystem Integration): Add governance and simulation support
5. **Phase 7** (Semantic Control Plane): Implement Boolean-Semantic Framework
6. **Phase 3** (Ergonomics): Add observability, projections, modules
7. **Phase 4** (Polish): String operations and indexing

### Key Files to Modify

| Component               | File Path                        | Purpose                      |
| ----------------------- | -------------------------------- | ---------------------------- |
| **Core DSL**            |                                  |                              |
| Grammar                 | `sea-core/grammar/sea.pest`      | Define syntax rules          |
| AST                     | `sea-core/src/ast.rs`            | Define node types            |
| Parser                  | `sea-core/src/parser.rs`         | Parse tokens to AST          |
| Validator               | `sea-core/src/validator.rs`      | Semantic validation          |
| Evaluator               | `sea-core/src/evaluator.rs`      | Runtime evaluation           |
| Type System             | `sea-core/src/types.rs`          | Type checking                |
| **Projections**         |                                  |                              |
| CALM Projection         | `sea-core/src/calm.rs`           | Generate CALM JSON           |
| KG Projection           | `sea-core/src/kg.rs`             | Generate RDF/Turtle          |
| SBVR Projection         | `sea-core/src/sbvr.rs`           | Generate SBVR text           |
| **Python Bindings**     |                                  |                              |
| Python Entry            | `sea-core/src/python/lib.rs`     | PyO3 module definition       |
| Python API              | `sea-core/src/python/*.rs`       | Python wrappers              |
| Python Tests            | `tests/test_*.py`                | Python integration tests     |
| **TypeScript Bindings** |                                  |                              |
| TS Entry                | `sea-core/src/typescript/lib.rs` | NAPI-RS module definition    |
| TS API                  | `sea-core/src/typescript/*.rs`   | Node.js wrappers             |
| TS Types                | `index.d.ts`                     | TypeScript type definitions  |
| TS Tests                | `typescript-tests/*.test.ts`     | TypeScript integration tests |
| **WASM Bindings**       |                                  |                              |
| WASM Entry              | `sea-core/src/wasm/lib.rs`       | wasm-bindgen module          |
| WASM API                | `sea-core/src/wasm/*.rs`         | WebAssembly wrappers         |
| WASM Package            | `pkg/`                           | Published WASM artifacts     |
| WASM Tests              | `sea-core/tests/wasm_*.rs`       | WASM integration tests       |
| **CLI**                 |                                  |                              |
| CLI Entry               | `sea-core/src/bin/sea.rs`        | Main CLI entry point         |
| CLI Commands            | `sea-core/src/cli/mod.rs`        | Command enum definitions     |
| CLI Modules             | `sea-core/src/cli/*.rs`          | Command implementations      |
| CLI Tests               | `sea-core/tests/cli_*.rs`        | CLI integration tests        |
| Justfile                | `justfile`                       | Task runner recipes          |

### Testing Strategy

For each feature:

1. **Unit Tests**: Test parser, validator, evaluator independently
2. **Golden Tests**: End-to-end DSL → Projection → Validate roundtrip
3. **Error Tests**: Verify helpful error messages for invalid syntax
4. **Integration Tests**: Test interaction with VibesPro, GovernedSpeed, Temporal DB

### Cross-References

- **Current Grammar**: [sea.pest](file:///home/sprime01/projects/domainforge/sea-core/grammar/sea.pest)
- **Missing Items Analysis**: `tmp/polish-check.md`
- **Temporal DB Spec**: `docs/specs/vibespro-temporal-db-integration.md` (to be created)
- **Boolean-Semantic Framework**: See Phase 7 for mathematical proof
- **Example DSL Files**: `sea-core/examples/` and `sea-core/tests/`

### Success Criteria

This roadmap is complete when:

- [ ] All 21 features have passing tests
- [ ] DSL → CALM → DSL roundtrip is lossless
- [ ] DSL → KG → SHACL validation passes
- [ ] VibesPro can query Temporal DB using semantic coordinates
- [ ] GovernedSpeed can enforce orthogonal generation strategies
- [ ] All projections support the full feature set

# DomainForge - SEA DSL Copilot Instructions

## Project Overview

**DomainForge** is a multi-language domain modeling framework implementing the **SEA (Semantic Enterprise Architecture) DSL**. It provides a Rust core with FFI bindings for Python, TypeScript, and WebAssembly.

**Core Philosophy**: Five universal primitives (Entity, Resource, Flow, Instance, Policy) + Graph storage (IndexMap for determinism) + SBVR-aligned policy expressions = complete enterprise modeling capability.

**Recent Major Updates (Nov 2025)**:

- Multiline string support (`"""..."""`) in parser for entity/resource names
- Namespace API change: `namespace()` now returns `&str` (always present, "default" if unspecified) instead of `Option<&str>`
- Constructor patterns: `new()` for default namespace, `new_with_namespace()` for explicit namespace
- Flow API: Uses `ConceptId` parameters (not references) for resource_id, from_id, to_id
- ValidationError convenience constructors: `undefined_entity()`, `undefined_resource()`, `unit_mismatch()`, etc.
- IndexMap ensures deterministic iteration order for graph queries

## Architecture Pattern: Rust Core + FFI Bindings

```
sea-core/              # Rust implementation (source of truth)
├── src/
│   ├── primitives/    # Entity, Resource, Flow, Instance
│   ├── graph/         # Graph storage & queries
│   ├── policy/        # Policy engine (SBVR expressions)
│   ├── parser/        # Pest grammar parser (sea.pest)
│   ├── calm/          # CALM export/import
│   ├── python/        # PyO3 bindings
│   ├── typescript/    # napi-rs bindings
│   └── wasm/          # wasm-bindgen bindings
├── grammar/sea.pest   # DSL grammar definition
└── Cargo.toml         # Feature flags: python, typescript, wasm
```

**Key Principle**: Rust types are canonical. All language bindings wrap `sea_core` structs. Changes to core primitives require updating all three FFI layers.

## Build & Test Commands

### Rust Core

```bash
cargo build                           # Debug build
cargo build --release                 # Production build
cargo test                            # Run all tests
cargo test --test entity_tests        # Specific test file
cargo clippy -- -D warnings           # Linting (zero warnings required)
cargo doc --no-deps --open            # Generate docs
```

### Python Bindings (PyO3 + maturin)

```bash
maturin develop --features python       # Install for local testing
maturin build --release --features python # Build wheel
pytest tests/                           # Run Python tests
python -c "import sea_dsl; print(sea_dsl.__version__)"
```

**Critical**: Python bindings live in `sea-core/src/python/`, but are exposed as `sea_dsl` module. Build with `--features python`.

### TypeScript Bindings (napi-rs)

```bash
npm run build                         # Builds Rust + TypeScript
npm run build:debug                   # Debug build
npm test                              # Runs vitest
node -e "const sea = require('.'); console.log(sea)"
```

**Critical**: TypeScript bindings compile to `.node` native modules. Must rebuild after Rust changes. Build with `--features typescript`.

### WebAssembly

```bash
./scripts/build-wasm.sh               # Full build + optimization
wasm-pack build --target web --features wasm
```

Output goes to `pkg/`. Includes `wasm-opt` optimization if installed.

## Five Core Primitives (Vocabulary + Fact Model Layers)

Every domain model uses exactly these five types:

1. **Entity** (`primitives/entity.rs`) - Business actors/locations

   - Fields: `id: Uuid`, `name: String`, `namespace: Option<String>`, `attributes: HashMap<String, Value>`
   - Example: Warehouse, Customer, Department

2. **Resource** (`primitives/resource.rs`) - Quantifiable subjects of value

   - Fields: `id: Uuid`, `name: String`, `unit: String`, `namespace: Option<String>`
   - Example: Camera (units), Steel (kg), Money (USD)

3. **Flow** (`primitives/flow.rs`) - Resource transfers between entities

   - Fields: `id: Uuid`, `resource_id: Uuid`, `from_id: Uuid`, `to_id: Uuid`, `quantity: Decimal`
   - **Validation**: Must reference existing Entity and Resource UUIDs

4. **Instance** (`primitives/instance.rs`) - Physical resource instances at locations

   - Fields: `id: Uuid`, `resource_id: Uuid`, `entity_id: Uuid`, `serial: Option<String>`
   - Example: Camera #SN12345 at Warehouse

5. **Policy** (`policy/mod.rs`) - Business rules (Rule layer)
   - SBVR-aligned expressions: `forall`, `exists`, comparisons, aggregations
   - Severity levels: `info`, `warn`, `error`

## Graph Storage & Referential Integrity

The `Graph` struct (`graph/mod.rs`) enforces referential integrity:

```rust
// Adding Flow validates Entity + Resource exist
graph.add_flow(flow)?;  // Returns Err if references invalid

// Removing Entity checks for dependent Flows
graph.remove_entity(id)?;  // Fails if Flows reference it
```

**Pattern**: All primitives stored by UUID in IndexMaps. Queries walk relationships via UUID lookups.

### IndexMap for Deterministic Iteration

**Critical Decision**: Graph uses `IndexMap` instead of `HashMap` to guarantee stable iteration order.

**Why This Matters**:

- Policy evaluation iterates over `flows`, `entities`, `resources`, `instances` collections
- Non-deterministic iteration can cause policy results to vary between runs
- Test reproducibility requires consistent ordering
- CALM export/import round-trips must preserve semantics

**Implementation**:

```rust
// graph/mod.rs
pub struct Graph {
    entities: IndexMap<ConceptId, Entity>,
    resources: IndexMap<ConceptId, Resource>,
    flows: IndexMap<ConceptId, Flow>,
    instances: IndexMap<ConceptId, Instance>,
    // ... IndexMap maintains insertion order
}
```

**Usage Pattern**:

```rust
// Iteration order is deterministic (insertion order)
for (id, entity) in graph.entities() {
    // Always same order across runs
}

// Policy evaluation gets consistent results
let result = policy.evaluate(&graph)?;  // Reproducible
```

**Trade-off**: IndexMap has slightly higher memory overhead than HashMap (~40 bytes per entry) but guarantees O(1) access like HashMap. For typical graph sizes (<10K nodes), this is negligible compared to correctness benefits.

## DSL Parser (Pest Grammar)

Grammar lives in `sea-core/grammar/sea.pest`. Parser implementation in `parser/mod.rs`.

**Example DSL syntax**:

```
Entity "Warehouse" in logistics
Resource "Camera" units
Flow "Camera" from "Factory" to "Warehouse" quantity 100
Policy min_quantity as: forall f in flows: f.quantity >= 10
```

**Critical**: Parser produces AST → Graph. When modifying grammar, update both parser logic and AST types.

### Parser Extension Pattern (Adding New Syntax)

When adding new DSL features, follow this workflow:

1. **Update Grammar** (`sea.pest`):

   ```pest
   name = { multiline_string | string_literal }  // Add rule
   entity = { "Entity" ~ name ~ ("in" ~ identifier)? }  // Use in production
   ```

2. **Update AST Parser** (`parser/ast.rs`):

   ```rust
   fn parse_name(pair: Pair<Rule>) -> Result<String, String> {
       match pair.as_rule() {
           Rule::multiline_string => parse_multiline_string(pair),
           Rule::string_literal => Ok(pair.as_str().trim_matches('"').to_string()),
           _ => Err(format!("Expected name, got {:?}", pair.as_rule()))
       }
   }
   ```

3. **Add Unit Tests** (`tests/parser_tests.rs` or new file):

   ```rust
   #[test]
   fn test_multiline_entity_name() {
       let input = r#"Entity """Complex
       Name""" in namespace"#;
       let graph = parse(input).unwrap();
       assert_eq!(graph.entity_count(), 1);
   }
   ```

4. **Add Integration Tests** (`tests/phase_X_*.rs`):

   - Valid cases (accepted syntax)
   - Invalid cases (rejected syntax)
   - Edge cases (empty, unicode, escapes)

5. **Update Documentation**:
   - Grammar comments in `sea.pest`
   - API docs in affected modules
   - Examples in `examples/parser_demo.rs`

**Recent Example**: Multiline string support required changes to grammar (added `name` rule), AST parser (added `parse_multiline_string()` and `parse_name()`), and tests (phase_16_unicode_tests.rs).

## TDD Workflow & Phase Plans

This project follows strict **RED → GREEN → REFACTOR → VALIDATE** cycles documented in `docs/plans/`:

- **Phase 0-6**: Sequential (bootstrap → primitives → graph → policy → parser)
- **Phase 7-9**: Parallel (Python, TypeScript, WASM bindings)
- **Phase 10**: CALM integration (post-MVP)

**Testing Requirements**:

- Unit tests for each primitive (`tests/*_tests.rs`)
- Integration tests (`tests/*_integration_tests.rs`)
- Cross-language parity tests (Rust ≡ Python ≡ TypeScript)
- Property-based tests (proptest) for graph invariants

## FFI Implementation Patterns

### PyO3 Pattern

```rust
#[pyclass]
pub struct Entity {
    inner: sea_core::primitives::Entity,  // Wrap Rust type
}

#[pymethods]
impl Entity {
    #[new]
    pub fn new(name: String) -> Self {
        Self { inner: sea_core::primitives::Entity::new(name) }
    }
}
```

### napi-rs Pattern

```rust
#[napi]
pub struct Entity {
    inner: sea_core::primitives::Entity,
}

#[napi]
impl Entity {
    #[napi(constructor)]
    pub fn new(name: String) -> Self {
        Self { inner: sea_core::primitives::Entity::new(name) }
    }
}
```

**Critical**: All FFI wrappers delegate to `sea_core`. Never duplicate logic in bindings.

## SBVR-Aligned Policy Expressions

Policies use first-order logic inspired by SBVR standard:

- **Quantifiers**: `forall x in flows: ...`, `exists r in resources: ...`
- **Collections**: `flows`, `entities`, `resources`, `instances`
- **Operators**: `=`, `!=`, `<`, `>`, `and`, `or`, `not`, `implies`
- **Aggregations**: `count`, `sum`, `min`, `max`, `avg` (Phase 5)

**Example**: `forall f in flows where f.resource = "Camera": f.quantity >= 500`

## Layered Architecture (ADR-001)

Models follow three-layer separation:

1. **Vocabulary Layer**: Entity, Resource definitions
2. **Fact Model Layer**: Flow, Instance (reference vocabulary)
3. **Rule Layer**: Policy (references vocabulary + facts)

**Dependency Rules**:

- Rules may reference Vocabulary + Fact Model
- Fact Model may reference Vocabulary only
- Vocabulary has no upward dependencies

## Documentation Structure

- `docs/specs/` - PRD, SDS, ADR, API specifications
- `docs/plans/` - Phase-by-phase implementation plans
- `docs/diagrams/` - Mermaid C4 architecture diagrams
- `context/` - SBVR/UBM research and design context

**When adding features**: Update relevant SDS section, trace to PRD requirement, add ADR if architectural.

## Common Pitfalls & Recent API Changes

1. **UUID Version**: Always use `Uuid::new_v4()` (random). Never hardcode UUIDs in tests.
2. **Decimal Types**: Use `rust_decimal::Decimal` for quantities (not f64). Ensures precision in financial models.
3. **Feature Flags**: Building Python? Add `--features python`. TypeScript? Add `--features typescript`. WASM? Add `--features wasm`.
   - ⚠️ **Known Issue**: `uuid` crate in Cargo.toml incorrectly includes non-existent "wasm-bindgen" feature. Use only `["v4", "v5", "v7", "serde"]`.
4. **Namespace API**: `namespace()` returns `&str`, not `Option<&str>`. Check for empty string, not None. Default is `"default"`.
5. **Constructor Patterns**:
   - `Entity::new(name)` → default namespace
   - `Entity::new_with_namespace(name, ns)` → explicit namespace
   - `Flow::new(resource_id, from_id, to_id, quantity)` → takes ConceptId values (clone IDs before passing)
6. **Referential Integrity**: Graph validates all UUIDs. Add primitives in order: Entity/Resource → Flow/Instance → Policy.
7. **IndexMap for Determinism**: Graph uses `IndexMap` (not `HashMap`) to ensure stable iteration order for policies.
8. **Unit Handling**:
   - Import from `sea_core::units::unit_from_string`, NOT from `primitives`
   - Use `Unit::new()` with proper `Dimension` enum (Mass, Count, Currency, etc.)
   - ⚠️ **TODO**: Add zero-check validation for `base_factor` in `Unit::new()` (currently unchecked)
9. **Multiline Strings**: Parser supports `"""..."""` syntax for entity/resource names with newlines
10. **Error Handling**: Use `ValidationError` convenience constructors: `undefined_entity()`, `unit_mismatch()`, `type_mismatch()`
11. **Deprecation Markers**: Always add `#[deprecated(note = "use X instead")]` attribute when marking functions deprecated in docs

## Cross-Language Testing

After changing Rust core, verify all bindings:

```bash
# Rust
cargo test

# Python
maturin develop && pytest tests/

# TypeScript
npm run build && npm test

# Integration (if exists)
bash scripts/test_integration.sh
```

**Parity Requirement**: Same model input must produce identical validation results across all languages.

## Performance Targets (PRD Success Metrics)

- Model validation: <100ms for 10K node graphs
- FFI overhead: <1ms per call
- WASM bundle: <500KB gzipped (see `scripts/build-wasm.sh`)

Profile with: `cargo bench` (Criterion.rs benchmarks)

## CALM Integration (Fully Implemented)

**CALM (Common Architecture Language Model)** is a FINOS standard for architecture-as-code. SEA DSL provides **bidirectional** conversion to/from CALM format.

### Architecture

```
sea-core/src/calm/
├── mod.rs       # Public API
├── models.rs    # CALM data structures
├── export.rs    # SEA → CALM conversion
└── import.rs    # CALM → SEA conversion
```

### Export (SEA → CALM)

Converts SEA primitives to CALM JSON format:

```rust
use sea_core::calm::export;

let calm_json = export(&graph)?;  // Returns serde_json::Value
```

**Mapping:**

- `Entity` → CALM `Node` (type: `actor`)
- `Resource` → CALM `Node` (type: `resource`)
- `Instance` → CALM `Node` (type: `instance`) + ownership relationship
- `Flow` → CALM `Relationship` (type: `flow` with quantity)

**Python:**

```python
calm_json = graph.export_calm()  # Returns JSON string
with open('architecture.json', 'w') as f:
    f.write(calm_json)
```

**TypeScript:**

```typescript
const calmJson = graph.exportCalm(); // Returns JSON string
fs.writeFileSync("architecture.json", calmJson);
```

### Import (CALM → SEA)

Converts CALM JSON to SEA Graph:

```rust
use sea_core::calm::import;

let graph = import(&calm_json)?;  // Returns Result<Graph, String>
```

**Process:**

1. Parse CALM JSON to structured model
2. Create UUID mapping for node references
3. Import nodes → Entity/Resource/Instance primitives
4. Import relationships → Flow primitives
5. Validate referential integrity

**Python:**

```python
with open('architecture.json', 'r') as f:
    calm_json = f.read()
graph = Graph.import_calm(calm_json)
```

**TypeScript:**

```typescript
const calmJson = fs.readFileSync("architecture.json", "utf-8");
const graph = Graph.importCalm(calmJson);
```

### CALM Metadata Preservation

SEA uses `sea:` namespace for metadata:

```json
{
  "version": "2.0",
  "metadata": {
    "sea:exported": true,
    "sea:version": "0.0.1",
    "sea:timestamp": "2025-11-07T12:00:00Z"
  },
  "nodes": [
    {
      "unique-id": "uuid-here",
      "node-type": "actor",
      "name": "Warehouse",
      "namespace": "logistics",
      "metadata": {
        "sea:primitive": "Entity"
      }
    }
  ]
}
```

### Schema Validation

CALM exports are validated against JSON Schema:

**Schema:** `sea-core/schemas/calm-v1.schema.json`

Tests automatically verify schema compliance:

```bash
cargo test calm_schema_validation
```

### Round-Trip Testing

**Critical requirement**: `SEA → CALM → SEA` must preserve semantics.

```rust
// tests/calm_round_trip_tests.rs
let original_graph = create_complex_graph();
let calm_json = export(&original_graph)?;
let reconstructed_graph = import(&calm_json)?;
assert_eq!(original_graph.entity_count(), reconstructed_graph.entity_count());
```

Run round-trip tests:

```bash
cargo test calm_round_trip
```

### CALM Integration Patterns

**1. Export for Architecture Governance:**

```python
# Create SEA model programmatically
graph = Graph()
# ... add primitives ...

# Export to CALM for architecture review
calm_spec = graph.export_calm()
submit_to_architecture_board(calm_spec)
```

**2. Import Existing CALM Architectures:**

```typescript
// Load existing CALM architecture
const existingArch = loadCalmFile('enterprise-arch.json');

// Convert to SEA for domain modeling
const graph = Graph.importCalm(existingArch);

// Add business rules
graph.addPolicy({...});
```

**3. Bi-directional Sync:**

```python
# Load from CALM
graph = Graph.import_calm(calm_source)

# Add domain-specific policies
graph.add_policy(Policy(...))

# Re-export with enrichments
enriched_calm = graph.export_calm()
```

### Testing CALM Integration

```bash
# All CALM tests
cargo test calm

# Schema validation only
cargo test calm_schema_validation

# Round-trip tests only
cargo test calm_round_trip

# With verbose output
cargo test calm -- --nocapture
```

### Common CALM Patterns

**Entity with Namespace:**

```json
{
  "unique-id": "warehouse-123",
  "node-type": "actor",
  "name": "Main Warehouse",
  "namespace": "logistics"
}
```

**Flow Relationship:**

```json
{
  "unique-id": "flow-456",
  "relationship-type": {
    "flow": {
      "resource": "resource-789",
      "quantity": "1000"
    }
  },
  "parties": {
    "source": "factory-001",
    "destination": "warehouse-123"
  }
}
```

**Instance with Ownership:**

```json
{
  "unique-id": "instance-camera-001",
  "node-type": "instance",
  "metadata": {
    "sea:primitive": "Instance",
    "sea:resource_id": "resource-camera",
    "sea:entity_id": "warehouse-123"
  }
}
```

### Troubleshooting CALM

**Import fails with "Missing required field":**

- Verify CALM JSON has `version` and `nodes` fields
- Check schema compliance: `cargo test calm_schema_validation`

**Export missing primitives:**

- Ensure all primitives added to graph before export
- Check referential integrity: `graph.add_flow()` validates references

**Round-trip loses data:**

- Check namespace preservation in CALM metadata
- Verify UUID consistency in import ID mapping

## When to Consult Docs

- **Adding a primitive field**: Check `docs/specs/sds.md` for schema validation
- **Changing parser**: Update `grammar/sea.pest` + `parser/ast.rs` + integration tests
- **New ADR needed**: Architectural decisions that affect multi-language consistency
- **Phase planning**: Follow existing phase structure in `docs/plans/INDEX.md`

## Quick Reference

| Task               | Command                                     |
| ------------------ | ------------------------------------------- |
| Run all tests      | `cargo test && pytest tests/ && npm test`   |
| Format code        | `cargo fmt`                                 |
| Check lints        | `cargo clippy -- -D warnings`               |
| Build Python wheel | `maturin build --release --features python` |
| Build TypeScript   | `npm run build` (runs cargo + napi)         |
| Build WASM         | `./scripts/build-wasm.sh`                   |
| Generate docs      | `cargo doc --no-deps --open`                |

# Technical Specifications

## Technology Stack

### Core Runtime

**Language/Runtime**: Rust 1.75+ (stable channel)
**Justification**: Memory safety, zero-cost abstractions, mature ecosystem
**Traceability**: Based on ADR-002

**Key Dependencies**:

- `uuid` 1.6+: UUID generation and parsing
- `rust_decimal` 1.33+: Arbitrary-precision decimal arithmetic for quantities
- `chrono` 0.4+: Date/time handling with timezone support
- `thiserror` 1.0+: Ergonomic error type derivation
- `serde` 1.0+: Serialization framework
- `serde_json` 1.0+: JSON serialization for CALM export

### Language Bindings

**Python Binding**:

- Framework: PyO3 0.20+
- Build Tool: maturin 1.4+
- Target Python: 3.8+
- Distribution: Pre-built wheels for Linux/macOS/Windows × x86_64/aarch64
- Traceability: Based on ADR-002, ADR-007

**TypeScript/Node Binding**:

- Framework: napi-rs 2.14+
- Build Tool: napi-cli
- Target Node: 16+
- TypeScript: 5.0+
- Distribution: Pre-built `.node` binaries + TypeScript type definitions
- Traceability: Based on ADR-002, ADR-007

**WebAssembly Binding**:

- Framework: wit-bindgen 0.16+
- Target: wasm32-wasi, wasm32-unknown-unknown
- Runtime Compatibility: Browser, Deno, Cloudflare Workers, Wasmer
- Distribution: `.wasm` binary + JavaScript glue code
- Traceability: Based on ADR-002, ADR-007

### Development Tools

**Testing**:

- Unit/Integration: `cargo test` (Rust), pytest (Python), Jest (TypeScript)
- Property-Based: proptest 1.4+ (Rust)
- Benchmarking: Criterion.rs 0.5+
- Coverage: cargo-tarpaulin (Rust), coverage.py (Python)

**CI/CD**:

- Platform: GitHub Actions
- Matrix Testing: Linux/macOS/Windows × x86_64/aarch64 × Python 3.8-3.12 × Node 16/18/20
- Release Automation: Lockstep versioning enforced by scripts

**Documentation**:

- Rust: rustdoc with `cargo doc`
- Python: Sphinx 7.0+ with autodoc
- TypeScript: TypeDoc 0.25+
- Guides: Markdown with MkDocs Material

---

## INT-001: Python FFI Integration

**Type**: FFI Bridge
**Provider/System**: PyO3
**Related Items**: ADR-002, ADR-007, PRD-006, SDS-009

**Authentication**: N/A (in-process)

**Data Format**: Python objects ↔ Rust structs via PyO3 conversion traits

**Interface Specification**:

```python
# Python side: dataclasses
@dataclass
class Entity:
    id: UUID
    name: str
    namespace: Optional[str]
    attributes: Dict[str, Any]

# Rust side: conversion trait
impl IntoPy<PyObject> for Entity {
    fn into_py(self, py: Python) -> PyObject {
        let dict = PyDict::new(py);
        dict.set_item("id", self.id.to_string()).unwrap();
        dict.set_item("name", self.name).unwrap();
        // ... other fields
        dict.into()
    }
}
```

**Error Handling**:

- Rust `Error` → Python `ValueError` or `ValidationError` (for policy violations)
- Include full error message and diagnostic context
- Python tracebacks include original DSL file/line when applicable

**Performance Targets**:

- FFI call overhead: <1ms per call
- Batch operations: Define 100 entities in <10ms
- Validation: Maintain <100ms target for 10K nodes

**Rate Limits**: N/A

**Testing Strategy**:

- Cross-language consistency tests: Identical results for same model in Rust and Python
- Error propagation tests: Verify error messages and types
- Memory leak tests: 24-hour stress test with allocation monitoring

---

## INT-002: TypeScript/Node FFI Integration

**Type**: FFI Bridge
**Provider/System**: napi-rs
**Related Items**: ADR-002, ADR-007, PRD-007, SDS-010

**Authentication**: N/A (in-process)

**Data Format**: JavaScript objects ↔ Rust structs via napi-rs macros

**Interface Specification**:

```typescript
// TypeScript interfaces
interface Entity {
  id: string;
  name: string;
  namespace?: string;
  attributes?: Record<string, unknown>;
}

// Rust napi implementation
#[napi(object)]
pub struct Entity {
  pub id: String,
  pub name: String,
  pub namespace: Option<String>,
  pub attributes: Option<HashMap<String, String>>,
}
```

**Error Handling**:

- Rust `Error` → JavaScript `Error` with descriptive message
- Async operations return `Promise` that rejects with Error
- Stack traces preserved across FFI boundary

**Performance Targets**:

- FFI call overhead: <1ms per call
- Promise resolution: <5ms for validation operations
- Event loop integration: Non-blocking for operations >10ms

**Rate Limits**: N/A

**Testing Strategy**:

- Cross-language consistency: Compare results to Rust baseline
- Async behavior: Verify Promise resolution order
- Memory management: Ensure no leaked Rust objects in Node heap

---

## INT-003: WebAssembly Runtime Integration

**Type**: WebAssembly Module
**Provider/System**: wit-bindgen
**Related Items**: ADR-002, ADR-007, PRD-008, SDS-011

**Authentication**: N/A (client-side)

**Data Format**: WIT types ↔ JavaScript via generated glue code

**Interface Specification**:

```wit
resource model {
  constructor(namespace: option<string>)
  define-entity: func(name: string, attributes: list<tuple<string, string>>) -> result<entity, string>
  validate: func() -> result<validation-result, string>
}
```

**Error Handling**:

- WIT `result<T, E>` → JavaScript try/catch with thrown Error
- Error strings include diagnostic information

**Performance Targets**:

- Binary size: <500KB (gzipped)
- Instantiation: <100ms in browser
- Validation: Same <100ms target as Rust core

**Browser Compatibility**:

- Chrome 90+
- Firefox 89+
- Safari 15+
- Edge 90+

**Edge Runtime Compatibility**:

- Cloudflare Workers
- Deno
- Wasmer/Wasmtime (server-side)

**Testing Strategy**:

- Browser tests: Playwright across Chrome/Firefox/Safari
- Edge runtime tests: Deploy to Cloudflare Workers test environment
- Binary size monitoring: Alert on >10% increase

---

## INT-004: FINOS CALM Export (Post-MVP)

**Type**: JSON Serialization
**Provider/System**: FINOS Architecture-as-Code initiative
**Related Items**: ADR-006, PRD-014, SDS-013

**Authentication**: N/A (file-based export)

**Data Format**: JSON conforming to FINOS CALM schema

**Mapping Specification**:

```json
{
  "nodes": [
    {
      "unique-id": "entity::<uuid>",
      "node-type": "system",
      "name": "<entity.name>",
      "metadata": {
        "namespace": "<entity.namespace>",
        "sea-primitive": "Entity"
      }
    }
  ],
  "relationships": [
    {
      "unique-id": "flow::<uuid>",
      "relationship-type": {
        "connects": {
          "source": {"node": "entity::<from_entity.id>"},
          "destination": {"node": "entity::<to_entity.id>"}
        }
      },
      "metadata": {
        "resource": "<flow.resource>",
        "quantity": "<flow.quantity>",
        "sea-primitive": "Flow"
      }
    }
  ]
}
```

**Error Handling**:

- Invalid CALM: Validation errors with JSON path to problematic element
- Semantic loss warnings: Log SEA features not representable in CALM

**Integration Points**:

- FINOS CALM Viewer: Visual architecture diagrams
- FINOS CALM CLI: Validation and transformation tools

**Testing Strategy**:

- CALM schema validation: Validate exported JSON against FINOS schema
- Round-trip tests: SEA → CALM → SEA preserves semantics
- Integration tests: Import into FINOS CALM Viewer

---

## INT-005: ERP5 UBM Migration

**Type**: Data Migration Adapter
**Provider/System**: ERP5 Python API
**Related Items**: ADR-005, PRD-009, SDS-012

**Authentication**: ERP5 instance credentials (when accessing live instance)

**Data Format**: Python objects (ERP5 API) ↔ SEA primitives

**Mapping Functions**:

```python
# ERP5 to SEA
def migrate_node(erp5_node: Node) -> Entity:
    return Entity(
        name=erp5_node.getTitle(),
        namespace=erp5_node.getPortalType(),
        attributes={
            k: v for k, v in erp5_node.asDict().items()
            if k not in ['id', 'title', 'portal_type']
        }
    )

def migrate_movement(erp5_movement: Movement) -> Flow:
    return Flow(
        name=erp5_movement.getTitle(),
        resource=erp5_movement.getResource(),
        from_entity=erp5_movement.getSource(),
        to_entity=erp5_movement.getDestination(),
        quantity=erp5_movement.getQuantity(),
        status=erp5_movement.getSimulationState(),
        initiated_at=erp5_movement.getStartDate(),
        completed_at=erp5_movement.getStopDate(),
    )
```

**Error Handling**:

- Missing references: Error with suggestion to migrate referenced objects first
- Unmappable concepts: Log warning with manual migration guide reference

**Performance Targets**:

- Migration rate: >100 objects/second for standard UBM patterns
- Batch support: Migrate entire ERP5 modules (1000+ objects)

**Testing Strategy**:

- Sample data: Migrate ERP5 demo datasets, validate results
- Round-trip: SEA → ERP5 → SEA preserves semantics
- Edge cases: Handle custom ERP5 extensions

---

## SEC-001: Memory Safety Guarantees

**Category**: Memory Safety
**Implementation**: Rust ownership system + FFI boundary validation
**Related Items**: ADR-002, PRD-005, SDS-008

**Requirements**:

- No memory leaks across FFI boundaries
- No use-after-free errors
- No data races in concurrent access

**Implementation Details**:

```rust
// Rust: Compile-time guarantees via ownership
pub struct Model {
    graph: Graph, // Owned data
}

// Python FFI: Explicit lifetime management
#[pyfunction]
fn model_new() -> PyResult<usize> {
    let model = Box::new(Model::new(None));
    let handle = Box::into_raw(model) as usize;
    Ok(handle)
}

#[pyfunction]
fn model_drop(handle: usize) {
    unsafe {
        let _ = Box::from_raw(handle as *mut Model);
    } // Drop destroys model
}
```

**Validation**:

- Rust: Compile-time borrow checker
- FFI: Manual handle lifecycle management with drop guards
- Runtime: Valgrind and AddressSanitizer for leak detection

**Testing**:

- 24-hour stress tests with allocation monitoring
- Fuzzing with AFL++ for FFI boundaries
- Cross-language object lifecycle tests

---

## SEC-002: Expression Evaluation Safety

**Category**: Denial of Service Prevention
**Implementation**: Resource limits on policy evaluation
**Related Items**: PRD-012, SDS-006

**Threats**:

- Infinite loops in path traversals (circular graph references)
- Stack overflow from deeply nested expressions
- Excessive memory allocation in quantifier expansion

**Mitigations**:

```rust
pub struct EvaluationLimits {
    pub max_recursion_depth: usize,        // Default: 100
    pub max_quantifier_expansion: usize,   // Default: 10,000
    pub max_evaluation_time_ms: u64,       // Default: 1,000
}

pub fn evaluate_expr(
    expr: &Expr,
    context: &Context,
    graph: &Graph,
    limits: &EvaluationLimits,
    depth: usize,
) -> Result<Value, Error> {
    if depth > limits.max_recursion_depth {
        return Err(Error::RecursionLimitExceeded { depth });
    }

    match expr {
        Expr::ForAll { var, collection, body, .. } => {
            let bindings = expand_collection(collection, graph)?;
            if bindings.len() > limits.max_quantifier_expansion {
                return Err(Error::QuantifierExpansionLimitExceeded {
                    count: bindings.len(),
                    limit: limits.max_quantifier_expansion,
                });
            }
            // Evaluate with increased depth
            evaluate_quantifier(bindings, body, context, graph, limits, depth + 1)
        }
        // ... other cases
    }
}
```

**Configuration**:

- Limits adjustable per-policy
- Default limits protect against accidental DoS

**Testing**:

- Fuzzing: Generate adversarial expressions (deeply nested, large quantifiers)
- Regression: Ensure benign policies complete within limits

---

## SEC-003: Namespace Isolation

**Category**: Access Control
**Implementation**: Namespace-scoped reference resolution
**Related Items**: PRD-010, SDS-001

**Requirements**:

- Namespaces prevent accidental name collisions
- Explicit imports required for cross-namespace references
- No ambient/global scope leakage

**Implementation**:

```rust
pub fn resolve_reference(
    ref_: &Reference,
    registry: &NamespaceRegistry,
    active_namespace: &str,
) -> Result<Uuid, Error> {
    let namespace = ref_.namespace.as_deref().unwrap_or(active_namespace);

    registry.primitives
        .get(&format!("{}::{}", namespace, ref_.name))
        .copied()
        .ok_or_else(|| Error::UnresolvedReference {
            ref_type: "Entity/Resource/Flow",
            name: ref_.name.clone(),
            namespace: namespace.to_string(),
        })
}
```

**Security Properties**:

- Prevents namespace pollution from untrusted models
- Enables sandboxing: Load third-party models in isolated namespaces

**Testing**:

- Isolation tests: Verify models in separate namespaces can't access each other's private primitives
- Import tests: Validate explicit imports grant access

---

## PERF-001: Validation Performance

**Metric**: Validation time for policy evaluation
**Target**: <100ms for graphs with 10,000 nodes
**Related Items**: PRD-005, PRD-012, SDS-006, SDS-008

**Measurement Method**:

```rust
#[bench]
fn bench_validation_10k_nodes(b: &mut Bencher) {
    let model = generate_model_with_nodes(10_000);
    b.iter(|| {
        model.validate()
    });
}
```

**Optimization Strategies**:

- Graph indexing: O(1) lookups, O(k) traversals
- Short-circuit evaluation: And/Or operators return early
- Parallel policy evaluation: Rayon for independent policies (post-MVP)
- Memoization: Cache subexpression results for repeated evaluations

**Profiling**:

- Tool: `cargo flamegraph`
- Hotspots: Quantifier expansion, aggregation computation
- Target: <50% time in graph traversal, <30% in expression evaluation

**Monitoring**:

- CI benchmarks: Alert on >10% regression
- Production metrics: P50/P95/P99 validation latency

---

## PERF-002: FFI Overhead

**Metric**: Time overhead for cross-language calls
**Target**: <1ms per FFI call
**Related Items**: PRD-006, PRD-007, SDS-009, SDS-010

**Measurement Method**:

```python
# Python benchmark
import timeit

setup = "from sea import Model; model = Model()"
stmt = "model.define_entity('Test')"

overhead_ms = timeit.timeit(stmt, setup, number=1000) / 1000
assert overhead_ms < 1.0
```

**Optimization Strategies**:

- Batch operations: `define_entities(list)` instead of individual calls
- Reduce data copying: Pass pointers for large structures
- Avoid unnecessary conversions: Lazy serialization

**Profiling**:

- Tool: Python `cProfile`, Node.js `--prof`
- Breakdown: FFI call time vs. business logic time

**Monitoring**:

- Track FFI overhead percentage in end-to-end benchmarks
- Target: <10% of total operation time

---

## PERF-003: Memory Usage

**Metric**: Memory consumption per graph node
**Target**: <100 bytes per node for typical models
**Related Items**: PRD-004, SDS-005

**Measurement Method**:

```rust
#[test]
fn test_memory_per_node() {
    let initial_mem = current_memory_usage();
    let model = generate_model_with_nodes(1000);
    let final_mem = current_memory_usage();

    let bytes_per_node = (final_mem - initial_mem) / 1000;
    assert!(bytes_per_node < 100);
}
```

**Memory Breakdown**:

- Node storage: ~80 bytes (UUID + struct fields)
- Index overhead: ~20 bytes (hash map entries)
- Attribute storage: Variable (depends on custom attributes)

**Optimization Strategies**:

- String interning: Deduplicate repeated namespace/name strings
- Compact UUIDs: Use fixed-size arrays instead of heap allocation
- Lazy attribute loading: Load attributes on-demand for large models

**Monitoring**:

- Heap profiling: Valgrind/massif for detailed allocation traces
- CI memory tests: Alert on >20% increase

---

## PERF-004: Graph Traversal

**Metric**: Time to traverse N-hop upstream/downstream flows
**Target**: O(k) where k = result set size
**Related Items**: PRD-004, SDS-004, SDS-005

**Measurement Method**:

```rust
#[bench]
fn bench_upstream_traversal(b: &mut Bencher) {
    let model = generate_chain_model(depth = 1000);
    let leaf_flow = model.get_flows().last().unwrap();

    b.iter(|| {
        leaf_flow.upstream(&model.graph, None) // Traverse full chain
    });
}
```

**Performance Characteristics**:

- Single-hop: O(1) via index lookup
- N-hop: O(k) where k = total flows in N-hop neighborhood
- Visited set: O(k) memory to prevent revisiting nodes

**Optimization Strategies**:

- Adjacency lists: Pre-computed indexes for common queries
- Depth limiting: Early termination when hop limit reached
- Caching: Memoize traversal results for repeated queries (post-MVP)

**Monitoring**:

- Benchmark suite: Test on graphs with varying densities (sparse, dense, chain)
- CI regression tests: Detect algorithmic complexity changes

---

## Deployment & Operations

### Deployment Strategy

**Python (PyPI)**:

- **Artifact**: Pre-built wheels (manylinux, macOS, Windows)
- **Process**: `maturin build --release && maturin publish`
- **Versioning**: Lockstep with Rust core (e.g., 1.2.3)
- **Rollback**: PyPI supports yanking broken releases

**TypeScript (npm)**:

- **Artifact**: `.node` binaries + TypeScript type definitions
- **Process**: `npm run build && npm publish`
- **Versioning**: Lockstep (e.g., 1.2.3)
- **Rollback**: `npm unpublish` (within 72 hours) or deprecate

**Rust (crates.io)**:

- **Artifact**: Source crate + documentation
- **Process**: `cargo publish`
- **Versioning**: Lockstep
- **Rollback**: Yank (prevents new downloads, existing users unaffected)

**WebAssembly**:

- **Artifact**: `.wasm` binary + JS glue
- **Distribution**: npm package or CDN (unpkg, jsDelivr)
- **Versioning**: Lockstep
- **Rollback**: npm deprecation or CDN version pinning

### Monitoring

**Performance Metrics** (post-MVP):

- Validation latency: P50/P95/P99
- FFI call overhead: Average time per language binding
- Memory usage: Peak heap size for representative workloads

**Error Metrics**:

- Parse error rate: % of malformed input files
- Validation violation rate: Average violations per model
- FFI error rate: Cross-language exception frequency

**Observability** (post-MVP):

- Structured logging: JSON logs with trace IDs
- Distributed tracing: OpenTelemetry integration for multi-tier validation
- Metrics export: Prometheus format

### Logging

**Rust Core**:

- Framework: `tracing` with configurable levels (ERROR, WARN, INFO, DEBUG, TRACE)
- Format: Structured JSON for machine parsing
- Outputs: stdout (containerized), file (local development)

**Python Binding**:

- Integration: Bridge Rust `tracing` events to Python `logging`
- Configuration: Respect user's `logging.basicConfig` settings

**TypeScript Binding**:

- Integration: Emit Rust log events via Node.js `console` or custom logger
- Configuration: Environment variable `SEA_LOG_LEVEL`

**Example**:

```rust
use tracing::{info, warn, error};

#[tracing::instrument]
pub fn validate(&self) -> ValidationResult {
    info!(primitives = self.graph.node_count(), "Starting validation");

    let violations = self.evaluate_policies();

    if violations.errors > 0 {
        warn!(errors = violations.errors, "Validation found errors");
    }

    ValidationResult { violations, summary }
}
```

### Backup/Recovery

**Model Persistence** (post-MVP):

- Format: JSON or binary (bincode) serialization
- Storage: Local file system, S3, database (via optional adapters)
- Backup Strategy: Versioned snapshots with timestamps

**Disaster Recovery**:

- Models are immutable by default (functional updates)
- Recovery: Reload from last known good snapshot
- Validation: Run validation on restored models to ensure integrity

---

## Traceability Cross-Reference

### Technology Choices → Requirements

| Technology        | Addresses Requirements                                | Supports ADRs |
|-------------------|-------------------------------------------------------|---------------|
| Rust 1.75+        | PRD-005 (performance), PRD-015 (consistency)          | ADR-002       |
| PyO3 0.20+        | PRD-006 (Python bindings)                             | ADR-002, ADR-007 |
| napi-rs 2.14+     | PRD-007 (TypeScript bindings)                         | ADR-002, ADR-007 |
| wit-bindgen 0.16+ | PRD-008 (WASM bindings)                               | ADR-002, ADR-007 |
| Criterion.rs      | PRD-018 (benchmarking)                                | ADR-002       |

### Integration Points → Requirements

| Integration | Addresses Requirements             | Supports ADRs |
|-------------|------------------------------------|---------------|
| INT-001     | PRD-006 (Python FFI)               | ADR-007       |
| INT-002     | PRD-007 (TypeScript FFI)           | ADR-007       |
| INT-003     | PRD-008 (WASM)                     | ADR-007       |
| INT-004     | PRD-014 (CALM export)              | ADR-006       |
| INT-005     | PRD-009 (ERP5 migration)           | ADR-005       |

### Performance Specs → Requirements

| Spec      | Addresses Requirements                      | Supports ADRs |
|-----------|---------------------------------------------|---------------|
| PERF-001  | PRD-012 (validation engine)                 | ADR-003, ADR-004 |
| PERF-002  | PRD-006, PRD-007 (FFI overhead)             | ADR-007       |
| PERF-003  | PRD-004 (graph scalability)                 | ADR-003       |
| PERF-004  | PRD-004 (traversal queries)                 | ADR-003       |

### Security Specs → Requirements

| Spec      | Addresses Requirements         | Supports ADRs |
|-----------|--------------------------------|---------------|
| SEC-001   | PRD-005 (memory safety)        | ADR-002       |
| SEC-002   | PRD-012 (safe evaluation)      | ADR-004       |
| SEC-003   | PRD-010 (namespace isolation)  | ADR-001       |

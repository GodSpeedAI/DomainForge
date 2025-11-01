# Software Design Specification

## System Overview

**Architecture Pattern**: Layered architecture with Rust core and FFI-based language bindings

**Key Components**:

- Parser/Lexer (Rust): DSL grammar parsing and AST construction
- Domain Model (Rust): Graph-based representation with typed primitives
- Validation Engine (Rust): Policy evaluator with quantifier expansion
- Language Bindings: PyO3 (Python), napi-rs (TypeScript), wit-bindgen (WASM)
- Serialization: CALM export/import (post-MVP)

**Integration Points**:

- PyPI (Python package distribution)
- npm (TypeScript package distribution)
- crates.io (Rust crate distribution)
- FINOS CALM (architecture-as-code export)
- ERP5 UBM (legacy model migration)

---

## SDS-001: Namespace and Import Resolution System

**Addresses Requirements**: PRD-010
**Related ADRs**: ADR-001
**MVP Status**: [MVP]

### Component Description

Manages namespace scoping and cross-file reference resolution, enabling modular model organization. Implements lazy reference resolution with namespace-qualified lookups and import path validation.

### Technical Details

**Interfaces**:

- Input: Namespace declarations (`namespace finance;`), import statements (`import manufacturing::Product;`)
- Output: Resolved references mapping qualified names to primitive UUIDs
- Protocols: In-memory resolution; no network calls

**Data Model**:

```rust
pub struct NamespaceRegistry {
    /// Maps namespace name to set of defined primitive names
    namespaces: HashMap<String, HashSet<String>>,
    /// Maps fully-qualified name (namespace::name) to UUID
    primitives: HashMap<String, Uuid>,
    /// Current active namespace for unqualified names
    active_namespace: Option<String>,
}

pub struct Import {
    pub namespace: String,
    pub items: Vec<String>, // Empty = wildcard import
}

pub struct Reference {
    pub namespace: Option<String>,
    pub name: String,
    pub resolved_id: Option<Uuid>, // Lazy resolution
}
```

**API Specification**:

- Method: `register_namespace(name: &str) -> Result<()>`
  - Registers a new namespace
  - Error: Namespace already exists
- Method: `define_primitive(namespace: &str, name: &str, id: Uuid) -> Result<()>`
  - Adds primitive to namespace
  - Error: Duplicate name in namespace
- Method: `resolve_reference(ref: &Reference) -> Result<Uuid>`
  - Resolves reference to UUID using namespace scoping rules
  - Error: Undefined reference, ambiguous reference (multiple imports)

**Dependencies**: None (foundational component)

**Error Handling**:

- Undefined namespace: `Error::UndefinedNamespace { name, location }`
- Duplicate primitive: `Error::DuplicatePrimitive { namespace, name }`
- Unresolved reference: `Error::UnresolvedReference { reference, available_in }`

**Performance Considerations**:

- Metric: Reference resolution time <1ms
- Optimization: Use HashMap for O(1) lookups

**Security Considerations**:

- Concern: Circular imports leading to stack overflow
- Mitigation: Maintain visited set during import graph traversal; detect cycles

### Testing Strategy

- Unit Tests: Namespace registration, reference resolution, import path validation
- Integration Tests: Multi-file models with cross-namespace references
- Performance Tests: Resolution time for 100+ namespaces

---

## SDS-002: Entity Primitive Implementation

**Addresses Requirements**: PRD-002, PRD-011
**Related ADRs**: ADR-005
**MVP Status**: [MVP]

### Component Description

Implements Entity primitive representing business actors, locations, or accounts. Supports extensible attributes and user-defined relations with cardinality constraints.

### Technical Details

**Interfaces**:

- Input: Entity definition with name, attributes, relations
- Output: Entity struct with UUID, stored in graph
- Protocols: In-memory storage; optional persistence via storage adapters

**Data Model**:

```rust
pub struct Entity {
    pub id: Uuid,
    pub name: String,
    pub namespace: Option<String>,
    pub attributes: HashMap<String, AttrValue>,
    pub relations: HashMap<String, Vec<Ref<dyn Primitive>>>,
}

pub enum AttrValue {
    String(String),
    Decimal(Decimal),
    Integer(i64),
    Boolean(bool),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Enum(Ref<Enum>),
}

pub struct Relation {
    pub name: String,
    pub target_type: PrimitiveType,
    pub cardinality: Cardinality,
    pub inverse: Option<String>,
}

pub enum Cardinality {
    ZeroOrOne,      // 0..1
    ExactlyOne,     // 1..1
    ZeroOrMany,     // 0..*
    OneOrMany,      // 1..*
    Range(usize, Option<usize>), // n..m or n..*
}
```

**API Specification**:

- Method: `new(name: &str, namespace: Option<&str>) -> Entity`
  - Creates new Entity with generated UUID
- Method: `set_attribute(&mut self, key: &str, value: AttrValue) -> Result<()>`
  - Sets typed attribute
  - Error: Type mismatch if attribute declared with different type
- Method: `add_relation(&mut self, name: &str, target: Ref<dyn Primitive>) -> Result<()>`
  - Adds relationship to another primitive
  - Error: Cardinality violation

**Dependencies**: Namespace registry (for reference resolution)

**Error Handling**:

- Cardinality violation: `Error::CardinalityViolation { relation, expected, actual }`
- Type mismatch: `Error::AttributeTypeMismatch { key, expected, actual }`

**Performance Considerations**:

- Metric: Attribute access O(1) via HashMap
- Memory: ~200 bytes per Entity + attribute storage

### Testing Strategy

- Unit Tests: Attribute type validation, relation cardinality enforcement
- Integration Tests: Entity creation in graph, reference resolution
- Performance Tests: Attribute access time, relation traversal

---

## SDS-003: Resource Primitive Implementation

**Addresses Requirements**: PRD-002, PRD-011
**Related ADRs**: ADR-005
**MVP Status**: [MVP]

### Component Description

Implements Resource primitive representing quantifiable subjects of value (products, materials, services, currencies). Supports unit of measure and variants via extensible attributes.

### Technical Details

**Interfaces**:

- Input: Resource definition with name, unit, attributes
- Output: Resource struct with UUID, stored in graph
- Protocols: In-memory storage

**Data Model**:

```rust
pub struct Resource {
    pub id: Uuid,
    pub name: String,
    pub namespace: Option<String>,
    pub unit: UnitOfMeasure,
    pub attributes: HashMap<String, AttrValue>,
}

pub enum UnitOfMeasure {
    // Base types
    Count,          // Dimensionless quantity
    Mass(String),   // kg, g, lb, etc.
    Volume(String), // L, mL, gal, etc.
    Length(String), // m, cm, in, etc.
    Time(String),   // s, min, h, etc.
    Currency(String), // USD, EUR, etc.

    // User-defined enum
    CustomEnum(Ref<Enum>),
}
```

**API Specification**:

- Method: `new(name: &str, unit: UnitOfMeasure, namespace: Option<&str>) -> Resource`
  - Creates new Resource with generated UUID
- Method: `flows(&self, graph: &Graph) -> Vec<&Flow>`
  - Returns all flows moving this resource
- Method: `instances(&self, graph: &Graph) -> Vec<&Instance>`
  - Returns all physical instances of this resource

**Dependencies**: Namespace registry, Graph

**Error Handling**:

- Invalid unit: `Error::InvalidUnit { unit, valid_units }`

**Performance Considerations**:

- Metric: Flow/instance lookup O(1) via graph indexes

### Testing Strategy

- Unit Tests: Unit of measure validation, attribute handling
- Integration Tests: Resource referenced by flows and instances
- Performance Tests: Flow/instance query performance

---

## SDS-004: Flow Primitive Implementation

**Addresses Requirements**: PRD-002, PRD-011
**Related ADRs**: ADR-005
**MVP Status**: [MVP]

### Component Description

Implements Flow primitive representing transfers of resources between entities. Captures quantity, source/destination, status, and timestamps. Core primitive for modeling resource movements.

### Technical Details

**Interfaces**:

- Input: Flow definition with resource, from_entity, to_entity, quantity
- Output: Flow struct with UUID, stored in graph with adjacency indexes
- Protocols: In-memory storage with indexed edges

**Data Model**:

```rust
pub struct Flow {
    pub id: Uuid,
    pub name: String,
    pub namespace: Option<String>,

    // Core references (required)
    pub resource: Ref<Resource>,
    pub from_entity: Ref<Entity>,
    pub to_entity: Ref<Entity>,
    pub quantity: Decimal,

    // Optional fields
    pub status: Option<Ref<Enum>>,
    pub initiated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,

    // Extensible attributes
    pub attributes: HashMap<String, AttrValue>,
}
```

**API Specification**:

- Method: `new(name: &str, resource: Ref<Resource>, from: Ref<Entity>, to: Ref<Entity>, quantity: Decimal) -> Flow`
  - Creates new Flow with generated UUID
- Method: `upstream(&self, graph: &Graph, hops: Option<usize>) -> Vec<&Flow>`
  - Returns flows upstream (following from_entity chains)
  - `hops`: Maximum traversal depth (None = unlimited)
- Method: `downstream(&self, graph: &Graph, hops: Option<usize>) -> Vec<&Flow>`
  - Returns flows downstream (following to_entity chains)
- Method: `validate(&self, graph: &Graph) -> Vec<Violation>`
  - Validates quantity >0, references resolve, etc.

**Dependencies**: Namespace registry, Graph, Entity, Resource

**Error Handling**:

- Negative quantity: Violation with severity error
- Unresolved reference: `Error::UnresolvedReference { ref_type, name }`
- Same source/destination: Violation with severity warn (allowed but suspicious)

**Performance Considerations**:

- Metric: Upstream/downstream traversal O(k) where k = result size
- Graph indexes: `from_index`, `to_index`, `moves_index` for O(1) lookups

**Security Considerations**:

- Concern: Infinite loops in traversal (circular flows)
- Mitigation: Track visited flows; enforce maximum traversal depth

### Testing Strategy

- Unit Tests: Flow creation, reference validation
- Integration Tests: Upstream/downstream traversal, graph index correctness
- Performance Tests: Traversal performance on 10K-flow graphs

---

## SDS-005: Graph Data Structure and Indexes

**Addresses Requirements**: PRD-004
**Related ADRs**: ADR-003
**MVP Status**: [MVP]

### Component Description

Core graph data structure storing primitives as nodes with adjacency indexes for efficient traversal. Implements typed multi-graph with reverse indexes for bidirectional queries.

### Technical Details

**Interfaces**:

- Input: Primitive add/remove operations
- Output: Node retrieval, traversal queries
- Protocols: In-memory with thread-safe access via Arc<RwLock<Graph>>

**Data Model**:

```rust
pub struct Graph {
    // Node storage (UUID → Primitive)
    entities: HashMap<Uuid, Entity>,
    resources: HashMap<Uuid, Resource>,
    flows: HashMap<Uuid, Flow>,
    instances: HashMap<Uuid, Instance>,
    policies: HashMap<Uuid, Policy>,

    // Forward indexes (Flow → referenced primitives)
    from_index: HashMap<Uuid, Uuid>,        // flow_id → source_entity_id
    to_index: HashMap<Uuid, Uuid>,          // flow_id → dest_entity_id
    moves_index: HashMap<Uuid, Uuid>,       // flow_id → resource_id
    instance_of_index: HashMap<Uuid, Uuid>, // instance_id → resource_id
    located_at_index: HashMap<Uuid, Uuid>,  // instance_id → entity_id
    applies_to_index: HashMap<Uuid, Vec<Uuid>>, // policy_id → [primitive_ids]

    // Reverse indexes (Entity → incoming/outgoing flows)
    entity_incoming: HashMap<Uuid, Vec<Uuid>>, // entity_id → [flow_ids]
    entity_outgoing: HashMap<Uuid, Vec<Uuid>>, // entity_id → [flow_ids]
    resource_flows: HashMap<Uuid, Vec<Uuid>>,  // resource_id → [flow_ids]
    resource_instances: HashMap<Uuid, Vec<Uuid>>, // resource_id → [instance_ids]
    entity_instances: HashMap<Uuid, Vec<Uuid>>,  // entity_id → [instance_ids]
}
```

**API Specification**:

- Method: `get_entity(&self, id: Uuid) -> Option<&Entity>`
  - Retrieves entity by UUID (O(1))
- Method: `entity_incoming_flows(&self, entity_id: Uuid) -> &[Uuid]`
  - Returns flow IDs with to_entity = entity_id (O(1))
- Method: `flow_upstream(&self, flow_id: Uuid, hops: Option<usize>) -> Vec<Uuid>`
  - Traverses upstream following from_entity chains (O(k))
- Method: `add_flow(&mut self, flow: Flow) -> Result<()>`
  - Adds flow to graph, updates all indexes
  - Error: Reference to non-existent entity/resource
- Method: `validate(&self) -> ValidationResult`
  - Runs all policies, returns violations

**Dependencies**: All primitive types

**Error Handling**:

- Dangling reference: `Error::DanglingReference { primitive_type, id, ref_field }`
- Duplicate ID: `Error::DuplicateId { primitive_type, id }`

**Performance Considerations**:

- Metric: Node retrieval O(1), traversal O(k)
- Memory: ~100 bytes per node + ~50 bytes per edge in indexes
- Target: Support 10,000 nodes in <1GB memory

**Security Considerations**:

- Concern: Graph mutations during traversal (iterator invalidation)
- Mitigation: Use RwLock; readers block writers during traversal

### Testing Strategy

- Unit Tests: Index consistency after add/remove operations
- Integration Tests: Complex traversals, policy evaluation over graph
- Performance Tests: Traversal performance on 1K, 10K, 100K node graphs

---

## SDS-006: Policy Evaluator and Expression Engine

**Addresses Requirements**: PRD-003, PRD-012, PRD-013
**Related ADRs**: ADR-004
**MVP Status**: [MVP]

### Component Description

Evaluates Policy expressions over the graph using first-order modal logic. Implements quantifier expansion, boolean logic with short-circuit evaluation, aggregations, and deontic modality mapping to severity levels.

### Technical Details

**Interfaces**:

- Input: Policy AST, Graph, evaluation context (variable bindings)
- Output: Boolean result + Violation records for failed policies
- Protocols: Recursive AST traversal with memoization

**Data Model**:

```rust
pub struct Policy {
    pub id: Uuid,
    pub name: String,
    pub namespace: Option<String>,

    // Policy metadata
    pub applies_to: Vec<Ref<dyn Primitive>>,
    pub active: bool,
    pub severity: Severity,
    pub message: String,

    // Expressions
    pub when: Option<Expr>, // Activation condition
    pub rule: Expr,          // Main constraint
}

pub enum Severity {
    Info,
    Warn,
    Error,
}

pub enum Expr {
    // Literals
    Literal(Literal),
    Variable(String),

    // Logical
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Implies(Box<Expr>, Box<Expr>),

    // Comparison
    Equals(Box<Expr>, Box<Expr>),
    LessThan(Box<Expr>, Box<Expr>),
    GreaterThan(Box<Expr>, Box<Expr>),
    In(Box<Expr>, Box<Expr>),
    Between(Box<Expr>, Box<Expr>, Box<Expr>),

    // Quantifiers
    ForAll {
        var: String,
        collection: Collection,
        condition: Option<Box<Expr>>,
        body: Box<Expr>,
    },
    Exists {
        var: String,
        collection: Collection,
        condition: Option<Box<Expr>>,
        body: Box<Expr>,
    },

    // Aggregations
    Count(Collection),
    Sum(Collection, Box<Expr>),
    Avg(Collection, Box<Expr>),
    Min(Collection, Box<Expr>),
    Max(Collection, Box<Expr>),

    // Path tests
    Linked {
        from: Box<Expr>,
        target_type: PrimitiveType,
        rel_name: String,
    },
}

pub enum Collection {
    AllEntities,
    AllResources,
    AllFlows,
    AllInstances,
    PathExpression(Box<Expr>, String), // e.g., entity.incoming_flows()
}

pub struct Violation {
    pub policy_id: Uuid,
    pub severity: Severity,
    pub message: String,
    pub affected_primitives: Vec<Uuid>,
    pub rule_expression: String,
}
```

**API Specification**:

- Method: `evaluate(expr: &Expr, context: &Context, graph: &Graph) -> Result<Value>`
  - Recursively evaluates expression
  - Returns: Boolean, Decimal, String, or List
- Method: `expand_quantifier(quantifier: &Expr, context: &Context, graph: &Graph) -> Vec<Context>`
  - Expands forall/exists over graph nodes
  - Returns: List of contexts with variable bindings
- Method: `validate_policies(graph: &Graph) -> ValidationResult`
  - Evaluates all active policies
  - Returns: List of violations

**Dependencies**: Graph, primitive types

**Error Handling**:

- Type error in expression: `Error::TypeError { expr, expected, actual }`
- Undefined variable: `Error::UndefinedVariable { var, available }`
- Division by zero: `Error::DivisionByZero { expr }`

**Performance Considerations**:

- Metric: Policy evaluation <100ms for 10,000 nodes
- Optimization: Short-circuit evaluation for And/Or
- Optimization: Memoize subexpression results (cache-friendly for repeated evaluations)

**Security Considerations**:

- Concern: Infinite recursion in path expressions
- Mitigation: Enforce maximum recursion depth (100 levels)

### Testing Strategy

- Unit Tests: Expression evaluation, quantifier expansion, aggregation correctness
- Integration Tests: Policy evaluation over complex graphs
- Performance Tests: Validation time for 100 policies over 10K nodes

---

## SDS-007: SBVR Grammar Parser

**Addresses Requirements**: PRD-003
**Related ADRs**: ADR-004
**MVP Status**: [MVP]

### Component Description

Parses DSL source files into Abstract Syntax Tree (AST) using grammar aligned with SBVR controlled natural language. Supports primitives, expressions, and policy declarations.

### Technical Details

**Interfaces**:

- Input: Source text (String)
- Output: AST (Program struct) or ParseError with diagnostics
- Protocols: Single-pass parsing with error recovery

**Data Model**:

```rust
pub struct Program {
    pub shebang: Option<String>,
    pub namespace: Option<String>,
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
}

pub enum Declaration {
    Enum(EnumDecl),
    Entity(EntityDecl),
    Resource(ResourceDecl),
    Flow(FlowDecl),
    Instance(InstanceDecl),
    Policy(PolicyDecl),
}

pub struct ParseError {
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub message: String,
    pub snippet: String,
    pub suggestion: Option<String>,
}
```

**Grammar (EBNF Subset)**:

```ebnf
program       ::= shebang? namespace? import* declaration*
namespace     ::= "namespace" IDENT ";"
import        ::= "import" IDENT ("::" IDENT)* ("." "*")? ";"
declaration   ::= enum_decl | entity_decl | resource_decl | flow_decl | instance_decl | policy_decl

policy_decl   ::= "policy" IDENT "{"
                    "appliesTo:" "[" ref_list "]"
                    ("active:" BOOL)?
                    ("severity:" severity)?
                    "message:" STRING
                    ("when:" expr)?
                    "rule:" expr
                  "}"

expr          ::= quantifier | logical | comparison | literal
quantifier    ::= "forall" IDENT "in" collection ("where" expr)? ":" expr
                | "exists" IDENT "in" collection ("where" expr)? ":" expr
logical       ::= expr "and" expr | expr "or" expr | "not" expr | expr "implies" expr
comparison    ::= expr ("=" | "!=" | "<" | ">" | "<=" | ">=") expr
```

**API Specification**:

- Method: `parse(source: &str) -> Result<Program, Vec<ParseError>>`
  - Parses source into AST
  - Returns: Success with Program or list of parse errors

**Dependencies**: None (foundational component)

**Error Handling**:

- Syntax error: Include line/column, snippet, suggestion
- Error recovery: Continue parsing after error to report multiple issues

**Performance Considerations**:

- Metric: Parse time <10ms for 1,000-line files
- Memory: AST size ~10x source size

### Testing Strategy

- Unit Tests: Individual grammar rules, error recovery
- Integration Tests: Complete program parsing, roundtrip (parse → serialize → parse)
- Fuzzing: Random input testing for crash resistance

---

## SDS-008: Rust Core API and Error Handling

**Addresses Requirements**: PRD-005, PRD-017, PRD-018
**Related ADRs**: ADR-002
**MVP Status**: [MVP]

### Component Description

Rust crate (`sea-core`) exposing public API for model creation, validation, and serialization. Implements comprehensive error handling with diagnostics and zero-panic guarantee.

### Technical Details

**Interfaces**:

- Input: Builder-style API calls (define_entity, define_flow, etc.)
- Output: Result<T, Error> with rich diagnostics
- Protocols: Single-threaded or thread-safe (Arc<RwLock<Model>>)

**API Specification**:

```rust
pub struct Model {
    graph: Graph,
    namespace_registry: NamespaceRegistry,
}

impl Model {
    pub fn new(namespace: Option<String>) -> Self;

    pub fn define_entity(&mut self, name: &str, attributes: AttributeMap) -> Result<Entity, Error>;
    pub fn define_resource(&mut self, name: &str, unit: UnitOfMeasure, attributes: AttributeMap) -> Result<Resource, Error>;
    pub fn define_flow(&mut self, name: &str, resource: &str, from: &str, to: &str, quantity: Decimal, attributes: AttributeMap) -> Result<Flow, Error>;
    pub fn define_instance(&mut self, name: &str, of: &str, at: &str, attributes: AttributeMap) -> Result<Instance, Error>;
    pub fn define_policy(&mut self, name: &str, applies_to: Vec<RefAny>, active: bool, severity: Severity, message: String, when: Option<Expr>, rule: Expr) -> Result<Policy, Error>;

    pub fn validate(&self) -> ValidationResult;
    pub async fn validate_streaming(&self, callback: impl Fn(Violation) + Send + 'static) -> Result<(), Error>;

    pub fn get_entity(&self, id: Uuid) -> Option<&Entity>;
    pub fn get_flow(&self, id: Uuid) -> Option<&Flow>;
    // ... other getters
}

pub struct ValidationResult {
    pub violations: Vec<Violation>,
    pub summary: Summary,
}

pub struct Summary {
    pub total_policies: usize,
    pub active_policies: usize,
    pub errors: usize,
    pub warnings: usize,
    pub infos: usize,
}
```

**Error Type**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Parse error at {file}:{line}:{column}: {message}")]
    ParseError {
        file: String,
        line: usize,
        column: usize,
        message: String,
        snippet: String,
    },

    #[error("Undefined reference: {ref_type} '{name}' in namespace '{namespace}'")]
    UnresolvedReference {
        ref_type: String,
        name: String,
        namespace: String,
    },

    #[error("Cardinality violation: relation '{relation}' expects {expected}, found {actual}")]
    CardinalityViolation {
        relation: String,
        expected: String,
        actual: usize,
    },

    // ... additional error variants
}
```

**Dependencies**: All SDS components

**Error Handling**:

- All public functions return Result<T, Error>
- Never panic; use Result for recoverable errors
- Include location information (file, line, column) in errors

**Performance Considerations**:

- Metric: Validation <100ms for 10K nodes
- Benchmarking: Criterion.rs for micro-benchmarks

### Testing Strategy

- Unit Tests: API methods, error handling
- Integration Tests: End-to-end model creation and validation
- Benchmarks: Validation performance, memory usage

---

## SDS-009: Python Bindings (PyO3)

**Addresses Requirements**: PRD-006
**Related ADRs**: ADR-002, ADR-007
**MVP Status**: [MVP]

### Component Description

Python package (`sea-py`) exposing SEA DSL as native Python dataclasses with idiomatic snake_case naming, exception-based error handling, and async generator support.

### Technical Details

**Interfaces**:

- Input: Python function calls matching Rust API
- Output: Python dataclasses (Entity, Flow, etc.)
- Protocols: PyO3 FFI bridge

**Implementation Pattern**:

```python
# Python dataclasses (exposed to users)
from dataclasses import dataclass, field
from typing import Optional, Dict, Any, AsyncIterator
from uuid import UUID, uuid4
from decimal import Decimal
from datetime import datetime

@dataclass
class Entity:
    id: UUID = field(default_factory=uuid4)
    name: str
    namespace: Optional[str] = None
    attributes: Dict[str, Any] = field(default_factory=dict)

    def incoming_flows(self) -> list["Flow"]:
        """Returns flows with to_entity = self"""
        return _sea_core.entity_incoming_flows(str(self.id))

    def outgoing_flows(self) -> list["Flow"]:
        """Returns flows with from_entity = self"""
        return _sea_core.entity_outgoing_flows(str(self.id))

class Model:
    def __init__(self, namespace: Optional[str] = None):
        self._handle = _sea_core.model_new(namespace)

    def define_entity(self, name: str, attributes: Optional[Dict[str, Any]] = None, **kwargs) -> Entity:
        """Define a new entity in the model"""
        try:
            entity_dict = _sea_core.model_define_entity(self._handle, name, attributes or {}, kwargs)
            return Entity(**entity_dict)
        except Exception as e:
            raise ValueError(f"Failed to define entity '{name}': {e}")

    def validate(self) -> ValidationResult:
        """Validate all policies in the model"""
        result_dict = _sea_core.model_validate(self._handle)
        return ValidationResult(**result_dict)

    async def validate_streaming(self) -> AsyncIterator[Violation]:
        """Stream violations as they are detected"""
        async for violation_dict in _sea_core.model_validate_stream(self._handle):
            yield Violation(**violation_dict)
```

**PyO3 Rust Implementation**:

```rust
use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;

#[pyfunction]
fn model_new(namespace: Option<String>) -> PyResult<usize> {
    let model = Model::new(namespace);
    let handle = Box::into_raw(Box::new(model)) as usize;
    Ok(handle)
}

#[pyfunction]
fn model_define_entity(handle: usize, name: &str, attributes: HashMap<String, PyObject>, kwargs: HashMap<String, PyObject>) -> PyResult<PyObject> {
    let model = unsafe { &mut *(handle as *mut Model) };

    let attr_map = convert_py_attributes(attributes)?;
    let entity = model.define_entity(name, attr_map)
        .map_err(|e| PyValueError::new_err(format!("{}", e)))?;

    Python::with_gil(|py| entity_to_dict(entity, py))
}

fn convert_py_attributes(attrs: HashMap<String, PyObject>) -> PyResult<AttributeMap> {
    // Convert Python objects to Rust AttrValue
    // ...
}
```

**Dependencies**: SDS-008 (Rust core API)

**Error Handling**:

- Convert Rust Error → Python ValueError
- Include full error message and context

**Performance Considerations**:

- Metric: FFI overhead <1ms per call
- Optimization: Batch operations where possible

### Testing Strategy

- Unit Tests: Python API methods, error propagation
- Integration Tests: Full workflow (define → validate) in Python
- Cross-Language Tests: Compare Python results to Rust results

---

## SDS-010: TypeScript/Node Bindings (N-API)

**Addresses Requirements**: PRD-007
**Related ADRs**: ADR-002, ADR-007
**MVP Status**: [MVP]

### Component Description

TypeScript package (`sea-ts`) exposing SEA DSL as native TypeScript interfaces with camelCase naming, Promise-based async operations, and thrown Error objects.

### Technical Details

**Interfaces**:

- Input: TypeScript function calls
- Output: TypeScript interfaces (Entity, Flow, etc.)
- Protocols: N-API bridge via napi-rs

**TypeScript Interfaces**:

```typescript
export interface Entity {
  id: string;  // UUID string
  name: string;
  namespace?: string;
  attributes?: Record<string, unknown>;

  incomingFlows(): Promise<Flow[]>;
  outgoingFlows(): Promise<Flow[]>;
}

export interface Flow {
  id: string;
  name: string;
  resource: string;
  fromEntity: string;
  toEntity: string;
  quantity: number;
  status?: string;
  initiatedAt?: Date;
  completedAt?: Date;
  namespace?: string;
  attributes?: Record<string, unknown>;

  upstream(hops?: number): Promise<Flow[]>;
  downstream(hops?: number): Promise<Flow[]>;
}

export class Model {
  constructor(namespace?: string);

  defineEntity(name: string, options?: {
    attributes?: Record<string, unknown>
  }): Promise<Entity>;

  defineFlow(options: {
    name: string;
    resource: string;
    fromEntity: string;
    toEntity: string;
    quantity: number;
    attributes?: Record<string, unknown>;
  }): Promise<Flow>;

  validate(): Promise<ValidationResult>;

  validateStreaming(callback: (violation: Violation) => void): Promise<void>;
}
```

**napi-rs Implementation**:

```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct Model {
    inner: sea_core::Model,
}

#[napi]
impl Model {
    #[napi(constructor)]
    pub fn new(namespace: Option<String>) -> Self {
        Model {
            inner: sea_core::Model::new(namespace),
        }
    }

    #[napi]
    pub async fn define_entity(&mut self, name: String, options: Option<Object>) -> Result<Object> {
        let attributes = parse_options(options)?;
        let entity = self.inner.define_entity(&name, attributes)
            .map_err(|e| Error::from_reason(format!("{}", e)))?;

        entity_to_js_object(entity)
    }

    #[napi]
    pub async fn validate(&self) -> Result<Object> {
        let result = self.inner.validate();
        validation_result_to_js(result)
    }
}
```

**Dependencies**: SDS-008 (Rust core API)

**Error Handling**:

- Convert Rust Error → JavaScript Error
- Preserve stack traces

**Performance Considerations**:

- Metric: FFI overhead <1ms per call
- Use async/await for operations >10ms

### Testing Strategy

- Unit Tests: TypeScript API methods
- Integration Tests: Full workflow in Node.js
- Cross-Language Tests: Compare TypeScript results to Rust results

---

## SDS-011: WebAssembly Bindings (wit-bindgen)

**Addresses Requirements**: PRD-008
**Related ADRs**: ADR-002, ADR-007
**MVP Status**: [MVP]

### Component Description

WebAssembly module (`sea-wasm`) exposing SEA DSL via WIT interface for browser and edge runtime usage. Generates JavaScript glue code automatically.

### Technical Details

**WIT Interface**:

```wit
package sea:core@1.0.0

interface model {
  record entity {
    id: string,
    name: string,
    namespace: option<string>,
  }

  record flow {
    id: string,
    name: string,
    resource: string,
    from-entity: string,
    to-entity: string,
    quantity: float64,
    status: option<string>,
    initiated-at: option<string>,
    completed-at: option<string>,
  }

  resource model {
    constructor(namespace: option<string>)

    define-entity: func(name: string, attributes: list<tuple<string, string>>) -> result<entity, string>
    define-flow: func(decl: flow-declaration) -> result<flow, string>
    validate: func() -> result<validation-result, string>
  }

  record validation-result {
    violations: list<violation>,
    summary: validation-summary,
  }
}
```

**JavaScript Usage**:

```javascript
import { Model } from './sea_wasm.js';

const model = new Model("finance");
const company = await model.defineEntity("Company", {
  attributes: { revenue: "1000000" }
});

const result = await model.validate();
console.log(result.violations);
```

**Dependencies**: SDS-008 (Rust core API)

**Error Handling**:

- WIT Result types map to JavaScript exceptions
- Error messages preserved across boundary

**Performance Considerations**:

- Metric: WASM binary size <500KB gzipped
- Instantiation time <100ms in browser

### Testing Strategy

- Unit Tests: WIT interface generation
- Integration Tests: Browser and Deno runtime usage
- Cross-Language Tests: Compare WASM results to Rust results

---

## SDS-012: ERP5 UBM Migration Adapter

**Addresses Requirements**: PRD-009
**Related ADRs**: ADR-005
**MVP Status**: [MVP]

### Component Description

Adapter providing bidirectional mapping between ERP5 UBM concepts and SEA primitives, enabling migration of existing ERP5 models.

### Technical Details

**Mapping Rules**:

| ERP5 UBM    | SEA Primitive | Mapping Notes                                  |
|-------------|---------------|------------------------------------------------|
| Node        | Entity        | 1:1 mapping; preserve node metadata           |
| Resource    | Resource      | 1:1 mapping; map UoM to UnitOfMeasure enum    |
| Movement    | Flow          | 1:1 mapping; source→from_entity, dest→to_entity|
| Item        | Instance      | 1:1 mapping; resource→of, location→at         |
| Path        | Policy        | Contextual rules → Policy with when clause    |

**API Specification**:

```rust
pub struct Erp5Adapter;

impl Erp5Adapter {
    pub fn migrate_node(node: &Erp5Node) -> Result<Entity, Error> {
        Entity::new(&node.title, Some(&node.portal_type))
            .with_attributes(node.properties.clone())
    }

    pub fn migrate_movement(movement: &Erp5Movement, graph: &Graph) -> Result<Flow, Error> {
        let resource = graph.resolve_resource(&movement.resource)?;
        let source = graph.resolve_entity(&movement.source)?;
        let destination = graph.resolve_entity(&movement.destination)?;

        Flow::new(
            &movement.title,
            resource,
            source,
            destination,
            movement.quantity,
        )
    }

    pub fn export_to_erp5(model: &Model) -> Result<Erp5Model, Error> {
        // Reverse mapping: SEA → ERP5
    }
}
```

**Dependencies**: SDS-008 (Rust core), ERP5 Python API (for Python bindings)

**Error Handling**:

- Unmappable concepts: Log warnings, provide manual migration guide
- Missing references: Error with suggestions

### Testing Strategy

- Integration Tests: Migrate sample ERP5 models, validate results
- Round-Trip Tests: SEA → ERP5 → SEA preserves semantics

---

## SDS-013: CALM Serialization (Post-MVP)

**Addresses Requirements**: PRD-014
**Related ADRs**: ADR-006
**MVP Status**: [POST-MVP]

### Component Description

Implements serialization and deserialization of SEA models to/from FINOS CALM JSON format for architecture-as-code integration.

### Technical Details

**CALM Mapping**:

| SEA Primitive | CALM Concept       | Mapping Strategy                              |
|---------------|--------------------|-----------------------------------------------|
| Entity        | Node               | Map to CALM node with unique ID               |
| Resource      | Resource Type      | Use CALM extension for resource metadata      |
| Flow          | Relationship       | Map to CALM relationship with quantity attr   |
| Policy        | Constraint         | Use CALM constraint extension points          |

**API Specification**:

```rust
impl Model {
    pub fn to_calm(&self) -> serde_json::Value {
        let nodes = self.graph.entities.iter()
            .map(|(id, entity)| calm_node_from_entity(id, entity))
            .collect();

        let relationships = self.graph.flows.iter()
            .map(|(id, flow)| calm_relationship_from_flow(id, flow))
            .collect();

        json!({
            "nodes": nodes,
            "relationships": relationships,
            "metadata": {
                "namespace": self.namespace,
                "version": "1.0",
            }
        })
    }

    pub fn from_calm(calm: serde_json::Value) -> Result<Model, Error> {
        // Deserialize CALM JSON → SEA Model
    }
}
```

**Dependencies**: SDS-008 (Rust core), serde_json

**Error Handling**:

- Invalid CALM: Validation errors with JSON path
- Semantic loss: Warn about unmappable SEA features

### Testing Strategy

- Round-Trip Tests: SEA → CALM → SEA preserves semantics
- Integration Tests: Export to CALM, validate with FINOS tools

---

## SDS-014: Release Automation and CI/CD

**Addresses Requirements**: PRD-016
**Related ADRs**: ADR-008
**MVP Status**: [MVP]

### Component Description

CI/CD pipeline automating lockstep releases across all language packages with consistent versioning, testing, and artifact distribution.

### Technical Details

**GitHub Actions Workflow**:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        target: [x86_64, aarch64]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - name: Run Rust tests
        run: cargo test --all-features
      - name: Run Python tests
        run: |
          pip install -e .
          pytest tests/
      - name: Run TypeScript tests
        run: |
          npm install
          npm test

  publish-rust:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - name: Publish to crates.io
        run: cargo publish --token ${{ secrets.CARGO_TOKEN }}

  publish-python:
    needs: test
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - name: Build wheels
        run: maturin build --release
      - name: Publish to PyPI
        run: maturin publish --token ${{ secrets.PYPI_TOKEN }}

  publish-npm:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - name: Build npm package
        run: npm run build
      - name: Publish to npm
        run: npm publish --access public
        env:
          NODE_AUTH_TOKEN: ${{ secrets.NPM_TOKEN }}
```

**Version Management**:

- Single `VERSION` file at repository root
- All `Cargo.toml`, `pyproject.toml`, `package.json` read from VERSION
- CI enforces consistency

**Dependencies**: GitHub Actions, maturin (Python), napi-rs (TypeScript)

### Testing Strategy

- Pre-Release Tests: Full test suite on all platforms
- Post-Release Smoke Tests: Install from registries, run basic usage

---

## SDS-015: Documentation Generation

**Addresses Requirements**: PRD-019
**Related ADRs**: ADR-007
**MVP Status**: [MVP]

### Component Description

Automated documentation generation from code comments, examples, and domain-specific modeling guides.

### Technical Details

**Documentation Sources**:

- Rust: `cargo doc` generates API docs from `///` comments
- Python: Sphinx with autodoc from docstrings
- TypeScript: TypeDoc from TSDoc comments

**Industry Guides Structure**:

```markdown
# Manufacturing Domain Guide

## Overview
How to model manufacturing processes with SEA DSL.

## Core Concepts
- Work Centers → Entity
- Products → Resource
- Production Orders → Flow

## Example: Assembly Line
\`\`\`python
from sea import Model

model = Model("manufacturing")
work_center = model.define_entity("Assembly Line A")
product = model.define_resource("Camera", unit="Count")
order = model.define_flow(
    name="Production Order 001",
    resource="Camera",
    from_entity="Raw Materials",
    to_entity="Assembly Line A",
    quantity=1000
)
\`\`\`
```

**Dependencies**: None (documentation-only)

### Testing Strategy

- Link Validation: Ensure all cross-references resolve
- Example Testing: Execute all code examples in docs

---

## Summary

This Software Design Specification provides implementation-ready technical details for all SEA DSL components. Each component addresses specific requirements, traces back to architectural decisions, and includes concrete data models, APIs, and testing strategies. The design prioritizes performance (< 100ms validation), developer experience (idiomatic language bindings), and maintainability (comprehensive error handling and testing).

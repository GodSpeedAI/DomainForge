Here is a detailed API specification for the **SEA (Semantic Enterprise Architecture) DSL**.  This specification integrates the core grammar, mathematical model and cross‑language design principles described in the provided sources.  It is organized for clarity, extensibility, and developer ergonomics, and follows best practices for multi‑language tooling.

---

## 1 Overview and Mission

SEA DSL is a domain‑specific language for modelling enterprises and the flows of resources between them.  It formalizes business vocabularies, facts and rules on top of a **first‑order and modal logic** foundation and maps them isomorphically to a **typed, directed multi‑graph**.  The DSL is:

* **Layered**: Separates vocabulary (concepts), fact models (relationships) and rules (constraints).
* **Unified**: Uses five core primitives (Entity, Resource, Flow, Instance and Policy) to model any business domain.
* **Semantic**: Rules are expressed in a controlled language with quantifiers, aggregations and deontic modalities and are evaluated over the graph.
* **Cross‑Language**: Implements a high‑performance Rust core and exposes native APIs for Python (via PyO3), TypeScript/Node (via N‑API), and WebAssembly (via `wit-bindgen`).  All bindings follow idiomatic naming and error‑handling conventions for their host language.

---

## 2 Core Domain Model

The SEA DSL grammar defines a *Program* consisting of optional shebang, namespace declaration, imports and a sequence of declarations.  Declarations introduce enumerations, entities, resources, flows, instances or policies.

### 2.1 Primitive Types

| Primitive    | Purpose / Definition                                                                                                                                                         | Core Fields                                                                                                                                                                             |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Entity**   | Represents a business actor, location or account.  An Entity is always the *source* or *destination* of a flow.  It corresponds to ERP5’s “Node” concept.                    | `id: UUID`, `name: string`, optional `namespace`, key–value `attributes`, and user-defined relations.                                                                                   |
| **Resource** | Quantifiable subject of value—anything that can be measured, exchanged or consumed.  Variants (e.g., product families) are supported through extensible attributes.          | `id: UUID`, `name`, `unit` (primitive type or enum), optional `namespace`, and custom fields.                                                                                           |
| **Flow**     | Represents a transfer of a resource between two entities at a given quantity.  It corresponds to ERP5’s “Movement”.                                                          | `id`, `name`, `resource` (reference), `from_entity` and `to_entity` (references), `quantity` (decimal), optional `status`, `initiated_at`, `completed_at`, and user-defined attributes. |
| **Instance** | Traceable physical instance of a resource (e.g., a serialized asset).  It corresponds to ERP5’s “Item”.                                                                      | `id`, `of` (resource reference), `at` (entity reference), optional `status` and custom fields.                                                                                          |
| **Policy**   | Formalizes constraints and business rules.  Policies can specify the primitives they apply to, activation status, severity level, message, conditions and a rule expression. | `id`, `name`, `appliesTo: list[Ref]`, `active: boolean`, `severity: {"info","warn","error"}`, `message: string`, `when: Expr`, `rule: Expr`.                                            |

### 2.2 Auxiliary Constructs

* **Enums:** Enumerations define sets of symbolic values with optional numeric/string representations.
* **Attributes:** Custom typed fields attached to a primitive (e.g., `height: Decimal`).
* **Relations:** Typed connections between primitives with cardinality constraints, optional roles and inverses.
* **Constraints:** Attachable expressions that must hold for an individual primitive.
* **Namespaces & Imports:** Provide modularity and name scoping across files.

### 2.3 Expression Language

Policies and constraints use an expression language with the following features:

* **Boolean Logic:** `and`, `or`, `not`, and implication (`implies`).
* **Comparisons & Membership:** `=`, `!=`, `<`, `>`, `>=`, `<=`, `in`, `not in`, `between`, `is null`, pattern matching (`like`, `ilike`).
* **Arithmetic:** `+`, `-`, `*`, `/`, `%`, exponentiation `^`, concatenation `||`.
* **Aggregations:** `count`, `sum`, `min`, `max`, `avg` over comprehensions.
* **Quantifiers:** `forall` and `exists` with optional `where` predicates.
* **Collections and Comprehensions:** List/Map literals and set comprehensions.
* **Path tests:** `linked(x, Type, "relName")` to check graph connectivity; `exists` for existential property checks.
* **Tagged Literals:** `date"2025-10-09"`, `datetime"2025-10-09T12:34:56Z"`, durations and currency amounts.

All expressions are evaluated over the graph with a context for variable bindings.  Evaluation semantics follow first‑order logic with quantifiers and deontic modalities; modal operators such as obligation, permission and prohibition map to SBVR rules.

---

## 3 Mathematical Model

SEA represents domain models as a **typed directed multi‑graph** with node sets for each primitive and adjacency indexes for efficient traversal.  Each node stores the corresponding primitive (entity, resource, flow, instance or policy) identified by a UUID.  Edges connect flows to their resource, source and destination entities; and connect instances to their resource and location.  Reverse indexes provide constant‑time lookups (e.g., flows incoming to an entity).

**Graph operations** include retrieving primitives, traversing flows upstream/downstream, accessing incoming/outgoing flows on an entity and performing validation across the entire model.  The graph is the immutable input to the **policy evaluator**, which walks the expression AST to compute boolean results (with support for short‑circuit evaluation and quantifier expansion).

---

## 4 Rust Core API

The Rust crate `sea-core` exposes a stable public API while hiding internal parser and evaluator details.  Modules include:

* **parser/**: Grammar specification (Pest or nom), AST definitions and parsing.
* **model/**: Primitive structs (entity, resource, flow, instance, policy), graph implementation and query operations.
* **validation/**: Constraint checking, policy evaluator and diagnostics.
* **calm/**: Export/import functions for FINOS’s Architecture‑as‑Code (CALM) format.
* **storage/**: Optional adapters (SQL, RDF) for persistence.

### 4.1 Data Structures

#### Primitive Structs

```rust
// Example: Flow primitive (simplified)
pub struct Flow {
    pub id: Uuid,
    pub name: String,
    pub resource: Ref<Resource>,
    pub from: Ref<Entity>,
    pub to: Ref<Entity>,
    pub quantity: Decimal,
    pub status: Option<Ref<Enum>>,
    pub initiated_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub attributes: HashMap<String, AttrValue>,
}
```

Each primitive has a `Ref<T>` type for references (namespace + name + resolved ID).  Reference resolution is lazy and namespace‑aware.

#### Graph

```rust
pub struct Graph {
    entities: HashMap<Uuid, Entity>,
    resources: HashMap<Uuid, Resource>,
    flows: HashMap<Uuid, Flow>,
    instances: HashMap<Uuid, Instance>,
    policies: HashMap<Uuid, Policy>,

    // Adjacency indexes for fast traversal
    from_index: HashMap<FlowId, EntityId>,
    to_index: HashMap<FlowId, EntityId>,
    moves_index: HashMap<FlowId, ResourceId>,
    instance_of_index: HashMap<InstanceId, ResourceId>,
    located_at_index: HashMap<InstanceId, EntityId>,
    applies_to_index: HashMap<PolicyId, Vec<PrimitiveId>>,
    // ... reverse indexes omitted for brevity
}
```

The graph is thread‑safe via `Arc<RwLock<Graph>>`.  Core operations:

* `get_entity(id: Uuid) -> Option<&Entity>`
* `flow_upstream(flow_id: Uuid) -> Vec<&Flow>`
* `entity_incoming_flows(entity_id: Uuid) -> &[FlowId]`
* `validate(&self) -> ValidationResult`

#### Expression AST

Expression nodes include logical (`Or`, `And`, `Not`), comparison, arithmetic and quantifier constructs, as shown in the grammar.  The evaluator uses recursion with a context map for variables and implements short‑circuit semantics and quantifier expansion.

### 4.2 Public API

The main entry point is `Model`, which wraps a `Graph` and provides mutation and validation methods:

```rust
pub struct Model {
    graph: Graph,
}

impl Model {
    pub fn new(namespace: Option<String>) -> Self;
    pub fn define_entity(&mut self, name: &str, attrs: AttributeMap, ...) -> Result<Entity>;
    pub fn define_resource(&mut self, name: &str, unit: TypeOrUoM, ...) -> Result<Resource>;
    pub fn define_flow(&mut self, name: &str, resource: &str, from: &str, to: &str,
                       quantity: Decimal, ...) -> Result<Flow>;
    pub fn define_instance(&mut self, name: &str, of: &str, at: &str, ...) -> Result<Instance>;
    pub fn define_policy(&mut self, name: &str, applies_to: Vec<RefAny>,
                         active: bool, severity: Severity, message: String,
                         when: Option<Expr>, rule: Expr) -> Result<Policy>;

    pub fn validate(&self) -> ValidationResult;
    pub async fn validate_streaming(&self, cb: impl Fn(Violation) + Send + 'static) -> Result<()>;
    pub fn to_calm(&self) -> serde_json::Value;
    pub fn from_calm(calm: serde_json::Value) -> Result<Model>;
}
```

All functions return `Result<T, Error>`, where `Error` carries rich diagnostics; there are no panics.

---

## 5 Python API (PyO3)

### 5.1 Design Principles

* Expose **native Python dataclasses** rather than opaque Rust handles.
* Use **snake_case** names and Python’s type system (e.g., `Decimal`, `datetime`).
* Convert Rust errors into Python exceptions (typically `ValueError` or `pydantic.ValidationError`).
* Use **async generators** and callbacks for streaming rather than leaking Rust’s async runtime.
* Provide picklable, serializable objects to integrate seamlessly with Python tooling.

### 5.2 Classes

#### `Entity`

```python
@dataclass
class Entity:
    id: UUID = field(default_factory=uuid4)
    name: str
    namespace: Optional[str] = None
    attributes: Dict[str, Any] = field(default_factory=dict)

    def incoming_flows(self) -> list[Flow]:
        return _sea_core.entity_incoming_flows(str(self.id))

    def outgoing_flows(self) -> list[Flow]:
        return _sea_core.entity_outgoing_flows(str(self.id))

    def hosted_instances(self) -> list[Instance]:
        return _sea_core.entity_hosted_instances(str(self.id))
```

#### `Flow`

```python
@dataclass
class Flow:
    id: UUID = field(default_factory=uuid4)
    name: str
    resource: str      # Unresolved name; resolved lazily
    from_entity: str
    to_entity: str
    quantity: Decimal
    status: Optional[str] = None
    initiated_at: Optional[datetime] = None
    completed_at: Optional[datetime] = None
    namespace: Optional[str] = None
    attributes: Dict[str, Any] = field(default_factory=dict)

    def upstream(self, hops: Optional[int] = None) -> list[Flow]:
        return _sea_core.flow_upstream(str(self.id), hops)

    def downstream(self, hops: Optional[int] = None) -> list[Flow]:
        return _sea_core.flow_downstream(str(self.id), hops)

    def validate(self) -> ValidationResult:
        return _sea_core.flow_validate(str(self.id))
```

#### `Model`

```python
class Model:
    def __init__(self, namespace: Optional[str] = None):
        self._handle = _sea_core.model_new(namespace)
        self.namespace = namespace

    def define_entity(self, name: str, attributes: Optional[dict[str, Any]] = None, **kwargs) -> Entity:
        entity_dict = _sea_core.model_define_entity(self._handle, name, attributes or {}, kwargs)
        return Entity(**entity_dict)

    def define_flow(self, name: str, resource: str, from_entity: str,
                    to_entity: str, quantity: Decimal, **kwargs) -> Flow:
        flow_dict = _sea_core.model_define_flow(self._handle, name, resource, from_entity,
                                                to_entity, float(quantity), kwargs)
        return Flow(**flow_dict)

    def validate(self) -> ValidationResult:
        result_dict = _sea_core.model_validate(self._handle)
        return ValidationResult(**result_dict)

    async def validate_streaming(self) -> AsyncIterator[Violation]:
        async for violation_dict in _sea_core.model_validate_stream(self._handle):
            yield Violation(**violation_dict)

    def to_calm(self) -> dict[str, Any]:
        return _sea_core.model_to_calm(self._handle)

    @classmethod
    def from_calm(cls, calm_spec: dict[str, Any]) -> "Model":
        handle = _sea_core.model_from_calm(calm_spec)
        model = cls.__new__(cls)
        model._handle = handle
        model.namespace = calm_spec.get('metadata', {}).get('namespace')
        return model
```

Other primitives (`Resource`, `Instance`, `Policy`, `Enum`) follow the same pattern.  Developers can use `pydantic` models for input validation and conversion to/from dictionaries.

---

## 6 TypeScript / Node API (N‑API)

### 6.1 Design Principles

* Expose **native TypeScript interfaces and classes** with **camelCase** property names.
* Use `Promise` for asynchronous operations; avoid exposing Rust `Future` types.
* Convert Rust errors into standard `Error` objects with descriptive messages.
* Avoid retaining Rust objects across calls; instead, pass plain JavaScript objects.

### 6.2 Type Definitions

```ts
interface Entity {
  id: string;          // UUID
  name: string;
  namespace?: string;
  attributes?: Record<string, unknown>;

  incomingFlows(): Promise<Flow[]>;
  outgoingFlows(): Promise<Flow[]>;
  hostedInstances(): Promise<Instance[]>;
}

interface Flow {
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
  validate(): Promise<ValidationResult>;
}

interface Model {
  namespace?: string;

  defineEntity(name: string, attrs?: Record<string, unknown>, options?: object): Promise<Entity>;
  defineResource(name: string, unit: string, attrs?: Record<string, unknown>): Promise<Resource>;
  defineFlow(name: string, resource: string, fromEntity: string, toEntity: string,
             quantity: number, options?: object): Promise<Flow>;
  defineInstance(name: string, of: string, at: string, options?: object): Promise<Instance>;
  definePolicy(opts: PolicyDefinition): Promise<Policy>;

  validate(): Promise<ValidationResult>;
  validateStreaming(cb: (violation: Violation) => void): Promise<void>;
  toCalm(): Promise<unknown>;
  fromCalm(calm: unknown): Promise<Model>;
}
```

The Node binding is implemented via `napi-rs` macros.  All functions marshal plain objects across the FFI boundary; internal identifiers (UUIDs) are strings.  Streaming operations accept a callback; the core spawns an asynchronous Rust task and pushes results to JavaScript via the Node event loop.

---

## 7 WebAssembly API

### 7.1 WIT Interface

SEA exports a `wit` file describing its WebAssembly interface.  An example fragment:

```wit
package sea:core

interface model {
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
    namespace: option<string>,
  }

  resource model {
    constructor(namespace: option<string>)
    define-flow: func(self: &model, decl: flow-declaration) -> result<flow, validation-error>
    validate: func(self: &model) -> result<validation-result, error>
    // additional functions omitted
  }
}
```

Host languages (Web browsers, Cloudflare Workers, Deno) import the generated JS/TS wrapper.  Functions return `Result` types that map to thrown exceptions.  The WASM implementation shares the Rust core, with no cross‑language memory leaks; `wit-bindgen` handles glue code.

---

## 8 Semantic & Logical Semantics

SEA’s rule layer is built on **SBVR** semantics: it separates vocabulary, fact types and rules.  In SEA:

* **Vocabulary** maps to **Entity** and **Resource** declarations.  Noun concepts (e.g., `Company`, `Camera`) are defined as entity/resource names.
* **Fact Types** map to **Flows** and **Relations**.  For example, “Movement has source node” becomes a relation with cardinality 1..1 between a flow and an entity.
* **Rules** map to **Policy** declarations.  A policy uses modal operators like “It is obligatory that” (deontic) and quantifiers (“each”, “at least one”), expressed as `forall` or `exists` in the expression language.  Severity levels (“info”, “warn”, “error”) correspond to SBVR rule types (definitional vs. behavioral).

SEA’s expression syntax supports the full range of SBVR logical constructs—quantifiers, boolean operators, comparisons and aggregates—and therefore can encode necessity, obligation and prohibition.  For example, the SBVR rule “It is obligatory that each movement has exactly one source node” translates to:

```dsl
policy MovementSingleSource {
  appliesTo: [Flow]
  severity: error
  rule: forall m in Flow: count(n in m.from) = 1
}
```

The evaluator grounds variables over the graph, applies quantifiers and aggregates, and produces violations with diagnostic messages.

---

## 9 Interoperability & Integration

* **UBM Alignment:** SEA’s primitives correspond to ERP5’s Unified Business Model: `Resource ↔ Resource`, `Entity ↔ Node`, `Flow ↔ Movement`, `Instance ↔ Item` and `Policy` captures `Path` semantics (contextual conditions).  This mapping enables straightforward migration of existing ERP models into SEA.
* **SBVR Integration:** The vocabulary/fact/rule layers allow round‑trip transformation between SEA models and SBVR XMI/XML.  Use the open‑source Eclipse MDT/SBVR tools to parse or generate SBVR models and align them with SEA declarations.
* **CALM Export/Import:** SEA provides `to_calm()`/`from_calm()` on the `Model` class (Rust, Python, TypeScript) to serialize models to FINOS Architecture‑as‑Code JSON specification.
* **Storage Adapters:** Optional SQL and RDF adapters allow persisting graphs to relational or semantic databases; these are pluggable modules in the Rust core.

---

## 10 Versioning & Packaging

Follow a **lockstep versioning strategy**: all language packages share the same version number to avoid cognitive overhead and to enable clear bug reporting.  The recommended distribution names are:

| Ecosystem | Package Name           | Build Artifacts                                           |
| --------- | ---------------------- | --------------------------------------------------------- |
| Rust      | `sea-core` (crates.io) | `libsea_core.rlib`, `cdylib` for FFI                      |
| Python    | `sea-py` (PyPI)        | Pre‑built wheels for Linux/macOS/Windows (x86_64/aarch64) |
| Node      | `sea-ts` (npm)         | Pre‑built `.node` module; TypeScript typings              |
| WASM      | `sea-wasm`             | `.wasm` binary and JS glue                                |

Continuous integration must build for all target combinations (OS × architecture × Python/Node versions).  Semantic versioning is adopted; breaking changes bump the major version.  Pre‑release versions use `-alpha`/`-beta` suffixes.

---

## 11 Developer Experience & Best Practices

1. **Native Feel** – Use idiomatic naming conventions for each language (snake_case in Python, camelCase in TypeScript) and provide rich error messages native to that language.
2. **Composability** – Prefer pure data structures at API boundaries; return plain dictionaries or objects rather than foreign handles.
3. **Streaming & Async** – Expose asynchronous operations via native primitives (async generators in Python; callbacks or async iterables in TypeScript).  Use callbacks internally to bridge Rust’s async runtime to the host event loop.
4. **Validation** – Validate inputs early (e.g., using Pydantic in Python) and convert errors from Rust to descriptive host‑language exceptions.
5. **Extensibility** – Support user‑defined annotations (`@`), doc comments (`///`), and custom relation/inheritance declarations.  These are no‑ops in the core but can be interpreted by external tools.
6. **Performance** – Avoid chatty FFI calls.  Batch queries (e.g., retrieving multiple flows at once).  Use streaming for large result sets.
7. **Security** – Do not expose internal memory; use safe deserialization.  Enforce namespace scoping to prevent reference collisions.

---

## 12 Testing & Quality Assurance

* **Cross‑Language Consistency Tests:** Maintain a shared test suite of grammar samples and policy evaluations.  All bindings must produce identical results when fed the same input.
* **Property‑Based Testing:** Use Rust’s `proptest` or similar frameworks to generate random models and verify invariants (e.g., flows always have matching source/destination entities).
* **Regression Testing:** Write tests for known bug cases and keep them across languages.
* **Benchmarking:** Measure validation performance on graphs up to 10 000 nodes; ensure validation time <100 ms and FFI overhead <1 ms per call.

---

## 13 Summary

SEA DSL provides a **unified, semantically rigorous and cross‑language** framework for enterprise modelling.  By grounding the core in Rust and exposing idiomatic APIs in Python, TypeScript and WebAssembly, developers gain high performance and type safety without sacrificing usability.  Its design leverages the lessons of UBM and SBVR to ensure that vocabulary, facts and rules remain consistent across domains.  The lockstep versioning and emphasis on developer ergonomics further reduce cognitive overhead and technical debt.  This specification should serve as a blueprint for implementing the SEA DSL and delivering a pleasant, reliable developer experience.

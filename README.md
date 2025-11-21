# 🏛️ DomainForge - SEA DSL

### **Transform Business Logic into Executable Architecture**

> *Build enterprise models that business analysts can read and machines can execute*

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.8%2B-blue.svg)](https://www.python.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)](https://www.typescriptlang.org/)
[![WASM](https://img.shields.io/badge/wasm-ready-purple.svg)](https://webassembly.org/)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-342%20passing-brightgreen.svg)](sea-core/tests/)

---

## 🎯 Why SEA DSL Exists

**The hidden cost of business modeling:**

Your team spends weeks documenting business requirements in Word and Confluence. Product requirements get translated to technical specs. Technical specs get coded in Python. Then someone needs the same logic in TypeScript for the frontend. Six months later, nobody remembers which version matches production.

**Meanwhile:**

- Compliance audits require manual checks across scattered codebases
- New developers take days to understand domain logic
- Business rules drift between systems (Python validation ≠ TypeScript validation)
- "Where did we define that constraint?" becomes a daily question

**What if your business model could validate itself, generate code, and stay in sync across all systems?**

---

## ⚡ What SEA DSL Actually Does

**SEA (Semantic Enterprise Architecture) DSL is executable business modeling.**

Write your domain model **once** in plain language. Get:

- ✅ **Instant validation** — 10,000 entities checked in <100ms
- ✅ **Identical behavior across languages** — Python, TypeScript, Rust, WASM
- ✅ **Architecture-as-Code** — Full FINOS CALM integration
- ✅ **Zero translation overhead** — Business model IS the executable specification
- ✅ **Formal mathematics underneath** — First-order logic + typed graphs
- ✅ **Type-safe by default** — Full TypeScript inference, Python hints, Rust guarantees

**Think: Markdown for Business Models** — Simple enough for analysts, rigorous enough for architects, fast enough for runtime validation.

---

## 🚀 See It Work in 60 Seconds

### Your First Model

```python
from sea import Model

# Create a model
model = Model("camera-factory")

# Define WHO (Entities - the actors)
assembly_line = model.entity("Assembly Line A")
warehouse = model.entity("Warehouse")

# Define WHAT (Resources - things of value)
camera = model.resource("Camera", unit="units")

# Define MOVEMENT (Flows - transfers between entities)
production = model.flow(
    name="Daily Production",
    resource=camera,
    from_entity=assembly_line,
    to_entity=warehouse,
    quantity=1000
)

# Define RULES (Policies - constraints)
model.policy(
    "Production Quality Check",
    expression="forall f in Flow where f.resource = Camera: f.quantity >= 500",
    severity="error"
)

# Validate the model
results = model.validate()
print(f"✅ Model valid: {results.is_valid}")
```

**That's it.** You just created a formal, executable model that:

- Validates production quantities instantly
- Works identically in Python, TypeScript, Rust
- Exports to FINOS CALM for architecture governance
- Provides a single source of truth for your business logic

### Install & Run

```bash
# Python (build from source)
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge
pip install maturin
maturin develop

# Verify
python -c "import sea_dsl; print(sea_dsl.__version__)"

# Run the example
python examples/camera_factory.py
```

> 💡 **Note:** Pre-built packages (PyPI/npm) coming soon. Currently build from source.

---

<details>
<summary><h2>🧩 The Five Building Blocks (Click to expand)</h2></summary>

SEA DSL models everything using just **five universal primitives**:

### 1. 🏢 **Entity** - The Actors

Business locations, departments, or actors in your system.

**Examples**: Assembly Line, Customer, Warehouse, Sales Department

```python
assembly = model.entity("Assembly Line A")
```

### 2. 📦 **Resource** - Things of Value

Quantifiable items that move through your business.

**Examples**: Products, Money, Information, Services, Raw Materials

```python
camera = model.resource("Camera", unit="units")
```

### 3. 🔄 **Flow** - Transfers

Movement of resources between entities.

**Examples**: Purchase Orders, Shipments, Payments, Work Orders

```python
shipment = model.flow(
    resource=camera,
    from_entity=warehouse,
    to_entity=customer,
    quantity=100
)
```

### 4. 🔖 **Instance** - Physical Items

Specific, trackable instances of resources.

**Examples**: Camera #SN12345, Invoice #INV-2024-001, Container #CONT789

```python
camera_123 = model.instance(
    resource=camera,
    identifier="SN12345",
    quantity=1
)
```

### 5. 📜 **Policy** - Business Rules

Constraints and requirements expressed in controlled natural language.

**Examples**: "All shipments must be inspected", "Inventory cannot go negative"

```python
model.policy(
    "Inventory Control",
    expression="forall e in Entity: sum(Instance.quantity where Instance.at = e) >= 0",
    severity="error"
)
```

</details>

---

<details>
<summary><h2>📊 What You Get vs Traditional Approaches</h2></summary>

| Traditional Approach | With SEA DSL |
|---------------------|--------------|
| 📄 Write requirements in Word | 📄 Write executable models that validate themselves |
| 🔀 Business logic scattered across systems | 🎯 Single source of truth for domain rules |
| 🐌 Slow, manual compliance checks | ⚡ Instant validation (<100ms for 10K entities) |
| 🌐 Different rules in Python vs TypeScript | 🔒 Identical behavior across all languages |
| 🤔 "What did we agree on?" | 📖 The model *is* the agreement |
| 🔧 Manual architecture documentation | 📐 Auto-export to FINOS CALM |
| ⏰ Days to update cross-system logic | ⚡ Update once, deploy everywhere |

</details>

---

<details>
<summary><h2>🌟 Key Features & Architecture</h2></summary>

### 🎨 **Simple Yet Rigorous**

- Write models in controlled natural language
- Backed by formal mathematics (first-order logic, typed graphs)
- Business analysts understand it, machines execute it

### ⚡ **Blazing Fast**

- Validates 10,000-node models in <100ms
- Rust core for maximum performance
- Streaming validation for large models

### 🌍 **True Cross-Language**

- Identical semantics in Python, TypeScript, Rust, and WebAssembly
- Native idioms in each language (not a wrapper)
- FFI overhead <1ms per operation

### 📊 **Three-Layer Architecture**

Models separate concerns cleanly:

1. **Vocabulary**: Define business terms
2. **Fact Model**: Structure relationships
3. **Rules**: Specify constraints

### 🔗 **Standards-Based**

- **SBVR-Aligned**: Semantics of Business Vocabulary and Rules (OMG standard)
- **CALM Integration**: Full bidirectional conversion to/from FINOS CALM format
- **UBM Foundation**: Based on ERP5's Unified Business Model primitives
- **JSON Schema Validation**: All CALM exports validated against schema

### 🔒 **Type-Safe Everywhere**

- Full TypeScript type inference
- Python type hints supported
- Rust's compile-time guarantees

### Architecture Diagram

```
┌───────────────────────────────────────────────────────┐
│              Your Application                         │
├───────────────────────────────────────────────────────┤
│  Python API    │  TypeScript    │   WASM             │
│  (PyO3)        │  (napi-rs)     │  (wasm-bindgen)    │
│  maturin       │  .node binary  │  pkg/              │
├───────────────────────────────────────────────────────┤
│              Rust Core Engine (sea-core)              │
│  ┌─────────────┬──────────────┬─────────────────┐   │
│  │ Primitives  │ Graph Store  │ Policy Engine   │   │
│  │ (5 types)   │ (IndexMap)   │ (SBVR logic)    │   │
│  ├─────────────┼──────────────┼─────────────────┤   │
│  │ Parser      │ Validator    │ CALM Integration│   │
│  │ (Pest)      │ (Ref check)  │ (export/import) │   │
│  └─────────────┴──────────────┴─────────────────┘   │
└───────────────────────────────────────────────────────┘
```

### 🎯 Design Principles

1. **🧱 Layered Architecture**: Vocabulary → Facts → Rules (ADR-001)
   - No circular dependencies between layers
   - Clear separation of concerns

2. **⚡ Performance First**: <100ms validation for 10K nodes
   - Rust's zero-cost abstractions
   - IndexMap-based graph storage for O(1) lookups + deterministic iteration

3. **🔄 Deterministic Behavior**: Consistent results across runs
   - IndexMap ensures stable iteration order (not HashMap)
   - Critical for reproducible policy evaluation
   - Test reproducibility guaranteed

4. **🌍 Cross-Language Parity**: 100% identical semantics
   - All bindings wrap Rust core (never duplicate logic)
   - Extensive parity testing ensures equivalence
   - 342+ tests maintaining cross-language consistency

5. **📊 Standards Compliance**:
   - SBVR-aligned policy expressions (OMG standard)
   - FINOS CALM bidirectional conversion
   - JSON Schema validated exports

6. **🔒 Type Safety**: Maximum compile-time guarantees
   - Rust's ownership system prevents invalid states
   - TypeScript full type inference
   - Python type hints for IDE support

</details>

---

<details>
<summary><h2>📚 Real-World Examples</h2></summary>

### 🏭 **Manufacturing: Assembly Line**

```python
model = Model("electronics-manufacturing")

# Entities
supplier = model.entity("Component Supplier")
assembly = model.entity("Assembly Line A")
quality_control = model.entity("QC Department")
finished_goods = model.entity("Finished Goods Warehouse")

# Resources
pcb_board = model.resource("PCB Board", unit="pieces")
camera_module = model.resource("Camera Module", unit="units")

# Flows
component_delivery = model.flow(
    resource=pcb_board,
    from_entity=supplier,
    to_entity=assembly,
    quantity=500
)

# Policy: Just-in-Time inventory
model.policy(
    "JIT Inventory Control",
    expression="""
        forall e in Entity where e.type = 'Assembly':
            sum(Instance.quantity where Instance.at = e) <= 1000
    """,
    severity="warn"
)
```

### 💰 **Finance: Payment Processing**

```typescript
const model = new Model('payment-system');

const customer = model.entity('Customer Account');
const merchant = model.entity('Merchant Account');
const gateway = model.entity('Payment Gateway');

const money = model.resource('USD', { unit: 'dollars' });

const payment = model.flow({
  name: 'Customer Payment',
  resource: money,
  from: customer,
  to: merchant,
  via: gateway,
  quantity: 99.99
});

// Policy: Fraud detection
model.policy({
  name: 'Unusual Activity Detection',
  expression: `
    forall c in Entity where c.type = 'Customer':
      sum(Flow.quantity where Flow.from = c and Flow.timestamp > now() - 1hour) <= 10000
  `,
  severity: 'error'
});
```

### 🚚 **Logistics: Supply Chain**

```python
model = Model("global-supply-chain")

# Multi-hop flows
warehouse_us = model.entity("US Distribution Center", location="USA")
warehouse_eu = model.entity("EU Distribution Center", location="Germany")
retail_store = model.entity("Retail Store", location="France")

product = model.resource("Widget", unit="boxes")

# Cross-border shipment
model.flow(
    resource=product,
    from_entity=warehouse_us,
    to_entity=warehouse_eu,
    quantity=5000,
    attributes={"customs_cleared": True, "shipping_method": "Air"}
)

# Last-mile delivery
model.flow(
    resource=product,
    from_entity=warehouse_eu,
    to_entity=retail_store,
    quantity=100
)

# Policy: Customs compliance
model.policy(
    "Customs Documentation",
    expression="""
        forall f in Flow where f.from.location != f.to.location:
            f.attributes.customs_cleared = true
    """,
    severity="error"
)
```

</details>

---

<details>
<summary><h2>🛠️ Advanced Features</h2></summary>

### 📊 **Graph Queries**

Navigate your business model like a graph database:

```python
# Find all upstream suppliers for a product
suppliers = model.query("""
    from Entity where type = 'Finished Goods'
    traverse upstream via Flow.from
    filter Entity.type = 'Supplier'
""")

# Calculate total inventory across locations
total = model.aggregate("""
    sum(Instance.quantity where Instance.of = Camera)
    group by Instance.at
""")
```

### 🔄 **Streaming Validation**

Validate models incrementally as they're built:

```python
async for violation in model.validate_streaming():
    print(f"⚠️ {violation.policy}: {violation.message}")
    # Act on violations in real-time
```

### 🏛️ **CALM Architecture Export**

Export your domain model to FINOS CALM for architecture governance:

```python
# Build your domain model
graph = Graph()
factory = Entity.new("Camera Factory")
warehouse = Entity.new("Distribution Center")
camera = Resource.new("Camera", "units")
graph.add_entity(factory)
graph.add_entity(warehouse)
graph.add_resource(camera)

flow = Flow.new(
    camera.id().clone(),
    factory.id().clone(),
    warehouse.id().clone(),
    Decimal("1000")
)
graph.add_flow(flow)

# Export to CALM JSON
calm_json = graph.export_calm()
with open('architecture.json', 'w') as f:
    f.write(calm_json)

# Import from CALM
with open('existing-arch.json', 'r') as f:
    calm_data = f.read()
imported_graph = Graph.import_calm(calm_data)
```

**CALM Export Features:**

- ✅ **Bidirectional**: SEA ↔ CALM with semantic preservation
- ✅ **Schema Validated**: All exports validated against CALM JSON Schema v1
- ✅ **Round-Trip Tested**: SEA → CALM → SEA preserves all primitives
- ✅ **Metadata Preserved**: Namespaces, attributes maintained via `sea:` namespace
- ✅ **Standard Compliant**: Follows FINOS CALM specification exactly

### 🔌 **Extensible Attributes**

Add domain-specific metadata:

```python
warehouse = model.entity("Warehouse", attributes={
    "capacity_sqft": 50000,
    "climate_controlled": True,
    "certifications": ["ISO9001", "GMP"]
})
```

### 🗂️ **Workspace Registry (.sea-registry.toml)**

SEA provides a workspace-level registry file `.sea-registry.toml` to map files to logical namespaces via glob patterns. The `sea validate` CLI discovers the nearest registry when invoked from a file and expands directories by resolving files according to the registry rules.

See `docs/reference/sea-registry.md` and `schemas/sea-registry.schema.json` for examples and schema validation.

**Tip**: The CLI supports an optional `--fail-on-ambiguity` flag for `registry resolve` and `registry list` to make resolution fail when two matching namespace rules have an equal literal prefix length. By default, the resolver falls back to an alphabetical tie-breaker.

</details>

---

<details>
<summary><h2>📖 Installation & Setup Guide</h2></summary>

### 📦 Installation

Choose your language:

```bash
# Python (requires Rust toolchain for building from source)
pip install maturin
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge
maturin develop
# or build wheel: maturin build --release

# TypeScript/Node.js
npm install
npm run build
# This produces native .node bindings

# Rust
cargo add sea-core --path ./sea-core

# WebAssembly (browser/edge)
./scripts/build-wasm.sh
# Output in pkg/ directory
```

**Note**: Pre-built packages for PyPI and npm are not yet published. Build from source as shown above.

### ✅ Verify Installation

```bash
# Python
python -c "import sea_dsl; print(sea_dsl.__version__)"

# TypeScript
node -e "const sea = require('.'); console.log(sea)"

# Rust
cargo test --package sea-core

# WASM
# Check pkg/ directory for sea_core_bg.wasm
```

### 🔧 Running Tests

We provide `just` targets that run per-language test suites:

```bash
# per-language
just rust-test
just python-test
just ts-test

# run all tests
just all-tests
```

**VS Code Integration:**

- `Run Rust tests (just)` — runs `just rust-test`
- `Run Python tests (just)` — runs `just python-test`
- `Run TypeScript tests (just)` — runs `just ts-test`
- `Run All Tests (just)` — runs `just all-tests`

If you don't have `just` installed:

```bash
# Using cargo:
cargo install just --locked

# macOS (Homebrew):
brew install just

# Debian/Ubuntu (apt):
sudo apt-get install just

# or use the `just` install recipe:
just install-just
```

**Python Virtual Environment (Linux):**

On some Linux distributions, global pip is blocked by the OS (PEP 668). If you see an "externally-managed-environment" error:

```bash
python3 -m venv .venv
source .venv/bin/activate
python -m pip install --upgrade pip
just setup
```

### 🔍 Debugging Tests in VS Code

We added a `launch.json` with debug profiles for Python (pytest), TypeScript (Vitest), and Rust (CodeLLDB).

**Rust Debugging (Automated):**

```bash
# Using direct script
./scripts/prepare_rust_debug.sh entity_tests

# Or via just:
RUST_TEST_NAME=entity_tests just prepare-rust-debug

# Then run the Debug Rust Test (auto) profile in VS Code
```

</details>

---

<details>
<summary><h2>🤝 Who Uses SEA DSL?</h2></summary>

### 💼 **Business Analysts**

**"I can model our processes without writing code"**

- Visual modeling with text-based DSL
- Immediate validation feedback
- No programming required

### 🏛️ **Enterprise Architects**

**"Single source of truth for business architecture"**

- Export to CALM for architecture governance
- Integration with existing tools
- Formal mathematical foundation

### 💻 **Software Developers**

**"Type-safe domain models in my language"**

- Native APIs in Python, TypeScript, Rust
- Fast enough for runtime validation
- Auto-generated types and documentation

### 📋 **Compliance Officers**

**"Formalize regulations as executable policies"**

- Express complex rules in controlled language
- Automatic compliance checking
- Audit trail of policy changes

</details>

---

<details>
<summary><h2>🚦 Implementation Status & Roadmap</h2></summary>

### ✅ **Completed (v0.1.0)** - November 2025

**Core Framework:**

- [x] Five universal primitives (Entity, Resource, Flow, Instance, Policy)
- [x] Graph storage with **IndexMap** for deterministic iteration
- [x] Referential integrity enforcement (add/remove validation)
- [x] Pest-based DSL parser with full grammar support
- [x] **Multiline string support** (`"""..."""`) for entity/resource names
- [x] SBVR-aligned policy expression engine
- [x] UUID-based primitive identification (v4 + v7 support)
- [x] Namespace support with default namespace ("default")
- [x] ValidationError convenience constructors

**Language Bindings:**

- [x] Rust core library (`sea-core`)
- [x] Python bindings via PyO3 + maturin (`sea-dsl`)
- [x] TypeScript bindings via napi-rs (`@domainforge/sea`)
- [x] WebAssembly bindings via wasm-bindgen (`@domainforge/sea-wasm`)
- [x] Cross-language parity testing (100% semantic equivalence)

**Standards Integration:**

- [x] CALM (Common Architecture Language Model) bidirectional conversion
- [x] CALM JSON Schema v1 validation
- [x] Round-trip testing (SEA → CALM → SEA)
- [x] Metadata preservation with `sea:` namespace

**Testing & Quality:**

- [x] **342 total tests** - All passing, zero failures (as of Nov 2025)
  - 52 unit tests (lib tests)
  - 215+ integration tests
  - 19 doctests
  - 11+ CALM integration tests
- [x] Cross-language parity testing (Python ≡ TypeScript ≡ Rust)
- [x] Property-based tests (proptest for graph invariants)
- [x] CALM round-trip validation (SEA → CALM → SEA)
- [x] Performance benchmarks (<100ms for 10K nodes)
- [x] Deterministic iteration testing (IndexMap verification)

**Documentation:**

- [x] Complete API specifications (Rust, Python, TypeScript)
- [x] Architecture Decision Records (8 ADRs)
- [x] Phase-by-phase implementation plans
- [x] C4 architecture diagrams
- [x] SBVR/UBM design context documentation

### 🔄 **Future Enhancements (v0.2+)**

- [ ] Visual modeling UI (drag-and-drop interface)
- [ ] SQL/PostgreSQL storage adapter
- [ ] Real-time collaborative editing
- [ ] Advanced policy analytics dashboard
- [ ] GraphQL API server
- [ ] CLI tool for model manipulation

### 🔮 **Research & Exploration**

- [ ] Machine learning-assisted model validation
- [ ] Streaming validation for infinite graphs
- [ ] Cloud-hosted SaaS validation service
- [ ] Industry-specific template libraries (finance, logistics, healthcare)
- [ ] Integration with popular ERP systems

</details>

---

<details>
<summary><h2>📖 Documentation & Resources</h2></summary>

| Resource | Description |
|----------|-------------|
| 📘 [**Copilot Instructions**](.github/copilot-instructions.md) | **⭐ Essential guide for AI coding agents** (updated Nov 2025) |
| 🔗 [**API Specification**](docs/specs/api_specification.md) | Complete API reference for all languages |
| 📙 [**Product Requirements**](docs/specs/prd.md) | PRD with success metrics and requirements |
| 📕 [**System Design**](docs/specs/sds.md) | Technical specifications and component design |
| 🏛️ [**Architecture Decisions**](docs/specs/adr.md) | 8 ADRs documenting key architectural choices |
| 📋 [**Implementation Plans**](docs/plans/) | Phase-by-phase TDD implementation guides |
| 🗺️ [**CALM Mapping**](docs/specs/calm-mapping.md) | SEA ↔ CALM conversion specification |
| 🎓 [**Examples**](examples/) | Browser demo and parser examples |
| 🗂️ [**Namespace Registry**](docs/reference/sea-registry.md) | Configure `.sea-registry.toml` and workspace glob patterns |

### 🆕 Recent API Changes (November 2025)

**Breaking Changes:**

- **Namespace API**: `namespace()` now returns `&str` instead of `Option<&str>` (always returns "default" if unspecified)
- **Constructor patterns**: Use `Entity::new_with_namespace(name, ns)` for explicit namespace; when migrating from the old `Entity::new(name)` callsite, pass `"default"` as the namespace to preserve previous behavior.
- **Resource namespace default**: Prefer `Resource::new_with_namespace(name, unit, "default")` when an explicit namespace is required; existing `Resource::new(name, unit)` callsites should be updated to pass an explicit namespace (commonly `"default"`).
- **Flow API**: Constructor takes `ConceptId` values (not references) - clone IDs before passing

**New Features:**

- **Multiline strings**: Parser now supports `"""..."""` syntax for entity/resource names with embedded newlines
- **ValidationError helpers**: Convenience constructors like `undefined_entity()`, `unit_mismatch()`, `type_mismatch()`
- **IndexMap storage**: Graph uses IndexMap for deterministic iteration (guarantees reproducible policy evaluation)

**See**: [Copilot Instructions](.github/copilot-instructions.md) for complete migration guide and patterns.

</details>

---

<details>
<summary><h2>✏️ Contributing</h2></summary>

Please see `docs/CONTRIBUTING.md` for developer notes on building the CLI, TypeScript (N-API), and WASM. If you experience CI failures, attach logs and a minimal reproduction in your PR and we will help diagnose the issue.

</details>

---

## 🚦 Ready to Start?

```bash
# Clone and build
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge
pip install maturin
maturin develop

# Run your first model
python examples/camera_factory.py
```

---

## 📄 License

SEA DSL is open source under the [MIT License](LICENSE).

---

## 🙏 Acknowledgments

Built on the shoulders of giants:

- **ERP5 Unified Business Model** - Foundational primitive design
- **SBVR Standard (OMG)** - Semantic business vocabulary
- **FINOS CALM** - Architecture-as-Code integration
- **Rust Community** - High-performance core runtime

---

**Build enterprise models that business analysts can read and machines can execute**

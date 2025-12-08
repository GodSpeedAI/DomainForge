# 🏛️ DomainForge - SEA DSL

### **Transform Business Logic into Executable Architecture**

> _Build enterprise models that business analysts can read and machines can execute_

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.8%2B-blue.svg)](https://www.python.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)](https://www.typescriptlang.org/)
[![WASM](https://img.shields.io/badge/wasm-ready-purple.svg)](https://webassembly.org/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-544%2B%20passing-brightgreen.svg)](sea-core/tests/)

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

# Unit Conversion (New!)
# Automatically convert units using the 'as' operator
model.policy(
    "Latency Check",
    expression="1000 'ms' as 's' == 1 's'",
    severity="info"
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

### New Features (v0.2.0)

- **Unit Conversion**: Use the `as` operator for type-safe unit conversions (e.g., `1000 'ms' as 's'`).
- **Profiles**: Define DSL dialects with `@profile "cloud"` or `@profile "data"`.
- **Standard Library**: Import built-in types from `std:core`, `std:http`, and `std:aws`.

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

> 💡 **Note:** Pre-built packages are available — install via registries or build from source if you prefer.

### Support Matrix & Tests

- Rust core + CLI: `cargo test -p sea-core --features cli`
- Python bindings: `just python-test` (uses virtualenv if present)
- TypeScript bindings: `just ts-test` (Vitest)
- WASM: `wasm-pack build sea-core --target web --features wasm` (size checks in CI)

### Release Channels

- Source build: supported today for Rust/Python/TypeScript/WASM.
- CI workflows: GitHub Actions (`.github/workflows/ci.yml`, `release.yml`, `publish-python.yml`) build/test/package when secrets (PyPI/npm/crates.io) are configured.
- Tagged releases publish CLI binaries, wheels, npm package, and wasm-pack output per release.

---

## 💻 CLI Usage

The SEA CLI allows you to validate, import, and export models directly from the terminal.

```bash
# Install
cargo install --path sea-core --features cli

# Validate a file
sea validate model.sea

# Export to CALM
sea project --format calm model.sea architecture.json

# Import from Knowledge Graph
sea import --format kg model.ttl
```

[Full CLI Reference](docs/reference/cli.md)

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

### 🏛️ **Bindings & Usage (Python, TypeScript, WASM)**

There are language bindings for Python (PyO3), TypeScript (NAPI), and WASM (wasm-bindgen). Below are small usage snippets showing the `Dimension.parse` and `Unit` constructors.

Python (PyO3):

```python
import sea_dsl

# Parse a Dimension (case-insensitive)
d = sea_dsl.Dimension.parse("currency")
print(str(d))  # 'Currency'

# Create a Unit in Python using the Rust-backed Unit constructor
u = sea_dsl.Unit("USD", "US Dollar", "Currency", 1.0, "USD")
print(u.symbol, u.base_unit)  # USD USD
```

TypeScript (napi-rs):

```typescript
import { Dimension, Unit } from "@domainforge/sea";

const d1 = Dimension.parse("currency");
console.log(d1.name); // 'Currency'

const u = new Unit("USD", "US Dollar", "Currency", 1.0, "USD");
console.log(u.symbol, u.baseUnit);
```

WASM (wasm-bindgen in web context):

```js
import init, { Dimension, Unit } from "./sea_core.js";
await init();

const d = new Dimension("currency");
console.log(d.toString()); // 'Currency'

const u = new Unit("USD", "US Dollar", "Currency", 1.0, "USD");
console.log(u.symbol());
```

These examples demonstrate the cross-language parity for `Dimension` and `Unit` constructs backed by the Rust core.

---

<details>
<summary><h2>📊 What You Get vs Traditional Approaches</h2></summary>

| Traditional Approach                       | With SEA DSL                                        |
| ------------------------------------------ | --------------------------------------------------- |
| 📄 Write requirements in Word              | 📄 Write executable models that validate themselves |
| 🔀 Business logic scattered across systems | 🎯 Single source of truth for domain rules          |
| 🐌 Slow, manual compliance checks          | ⚡ Instant validation (<100ms for 10K entities)     |
| 🌐 Different rules in Python vs TypeScript | 🔒 Identical behavior across all languages          |
| 🤔 "What did we agree on?"                 | 📖 The model _is_ the agreement                     |
| 🔧 Manual architecture documentation       | 📐 Auto-export to FINOS CALM                        |
| ⏰ Days to update cross-system logic       | ⚡ Update once, deploy everywhere                   |

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
   - 544+ Rust tests across 69 test files maintaining cross-language consistency

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
const model = new Model("payment-system");

const customer = model.entity("Customer Account");
const merchant = model.entity("Merchant Account");
const gateway = model.entity("Payment Gateway");

const money = model.resource("USD", { unit: "dollars" });

const payment = model.flow({
  name: "Customer Payment",
  resource: money,
  from: customer,
  to: merchant,
  via: gateway,
  quantity: 99.99,
});

// Policy: Fraud detection
model.policy({
  name: "Unusual Activity Detection",
  expression: `
    forall c in Entity where c.type = 'Customer':
      sum(Flow.quantity where Flow.from = c and Flow.timestamp > now() - 1hour) <= 10000
  `,
  severity: "error",
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

### 🔍 **Enhanced Diagnostics & Error Handling**

SEA provides comprehensive error diagnostics with multiple output formats, fuzzy matching for suggestions, and native error types for all language bindings.

#### Error Codes

All validation errors include structured error codes (E001-E599) organized by category:

- **E001-E099**: Syntax and Parsing Errors
- **E100-E199**: Type System Errors
- **E200-E299**: Unit and Dimension Errors
- **E300-E399**: Scope and Reference Errors
- **E400-E499**: Policy Validation Errors
- **E500-E599**: Namespace and Module Errors

See the complete [Error Code Catalog](docs/specs/error_codes.md) for detailed descriptions and fixes.

#### Diagnostic Formatters

**CLI Output Formats:**

```bash
# Human-readable output with colors and source snippets (default)
sea validate myfile.sea

# JSON output for CI/CD integration
sea validate --format json myfile.sea

# LSP format for IDE integration
sea validate --format lsp myfile.sea

# Disable colors for piping
sea validate --format human --no-color myfile.sea

# Hide source code snippets
sea validate --no-source mydir/
```

**Programmatic Usage:**

```rust
use sea_core::error::diagnostics::{HumanFormatter, JsonFormatter, DiagnosticFormatter};

let error = ValidationError::undefined_entity("Warehouse", "line 10");

// Human-readable with colors
let formatter = HumanFormatter::new(true, true);
println!("{}", formatter.format(&error, Some(source_code)));

// JSON for tools
let json_formatter = JsonFormatter;
println!("{}", json_formatter.format(&error, None));
```

#### Fuzzy Matching Suggestions

Errors automatically suggest similar names when you make typos:

```python
# Your code:
Flow "Materials" from "Warehous" to "Factory"

# Error output:
Error[E001]: Undefined Entity: 'Warehous' at line 1
  --> 1:23
  |
1 | Flow "Materials" from "Warehous" to "Factory"
  |                       ^^^^^^^^^^
  |
  hint: Did you mean 'Warehouse'?
```

The fuzzy matcher uses Levenshtein distance to find similar names within an edit distance of 2.

#### Native Error Types

Each language binding provides native, idiomatic error types:

**Python:**

```python
from sea_dsl import Graph

try:
    graph = Graph.parse(source)
except Exception as e:
    print(f"{e.error_type}[{e.code}]: {e}")
    if hasattr(e, 'suggestion'):
        print(f"Hint: {e.suggestion}")
    if hasattr(e, 'line'):
        print(f"Location: line {e.line}, column {e.column}")
```

**TypeScript:**

```typescript
import { Graph } from "@domainforge/sea";

try {
  const graph = Graph.parse(source);
} catch (e: any) {
  // Parse embedded metadata
  const metadata = JSON.parse(e.message.split("__metadata__: ")[1]);
  console.error(`${metadata.errorType}[${metadata.code}]: ${e.message}`);
  if (metadata.suggestion) {
    console.log(`Hint: ${metadata.suggestion}`);
  }
}
```

**WASM/JavaScript:**

```javascript
import { Graph } from "@domainforge/sea-wasm";

try {
  const graph = Graph.parse(source);
} catch (e) {
  const [msg, meta] = e.split("__SEA_DSL_ERROR_METADATA__: ");
  const metadata = JSON.parse(meta);
  console.error(`${metadata.errorType}[${metadata.code}]: ${msg}`);
  if (metadata.suggestion) {
    console.log(`Hint: ${metadata.suggestion}`);
  }
}
```

**Key Features:**

- ✅ **30+ structured error codes** with detailed descriptions
- ✅ **Fuzzy matching** for "did you mean?" suggestions
- ✅ **Multiple output formats**: JSON, Human-readable, LSP
- ✅ **Source code snippets** with caret indicators
- ✅ **Color-coded output** (can be disabled)
- ✅ **Native error types** for Python, TypeScript, and WASM
- ✅ **Precise source locations** with line and column numbers

</details>

---

<details>
<summary><h2>📖 Installation & Setup Guide</h2></summary>

### 📦 Installation

**From Package Registries (Recommended)**

```bash
# Python
pip install sea-dsl

# Rust
cargo add sea-core

# TypeScript / Node.js
npm install @domainforge/sea

# WebAssembly
npm install @domainforge/sea-wasm
```

**Build from Source**

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

**Note**: Pre-built packages are available from PyPI and npm — see the package registries for the latest installation instructions.

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

### ⚙️ Runtime Evaluation Modes

- **Three-valued logic (default):** missing data yields `NULL`/`None`/`null` and the evaluator records an explanatory violation.
- **Strict boolean logic:** missing or non-boolean data raises evaluation errors; write explicit comparisons such as `COUNT(flows) > 0`.

**Rust**

```rust
use sea_core::Graph;

let mut graph = Graph::new();
graph.set_evaluation_mode(true); // three-valued (default)
let tri_result = policy.evaluate(&graph)?;

graph.set_evaluation_mode(false); // strict boolean
let strict_result = policy.evaluate(&graph)?;
```

**Python**

```python
from sea_dsl import Graph

graph = Graph()
graph.set_evaluation_mode(True)   # three-valued (default)
result = graph.evaluate_policy(policy_json)

graph.set_evaluation_mode(False)  # strict boolean
strict_result = graph.evaluate_policy(policy_json)
```

**TypeScript**

```typescript
import { Graph } from "@domainforge/sea";

const graph = new Graph();
graph.setEvaluationMode(true); // three-valued (default)
const result = graph.evaluatePolicy(policyJson);

graph.setEvaluationMode(false); // strict boolean
const strictResult = graph.evaluatePolicy(policyJson);
```

**WebAssembly (browser/edge)**

```javascript
import init, { Graph } from "./pkg/sea_core.js";

await init();
const graph = new Graph();
graph.setEvaluationMode(false); // toggle strict boolean mode
const result = graph.evaluatePolicy(policyJson);
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
<summary><h2>📖 Documentation & Resources</h2></summary>

| Resource                                                              | Description                                                |
| --------------------------------------------------------------------- | ---------------------------------------------------------- |
| 📘 [**Copilot Instructions**](.github/copilot-instructions.md)        | **⭐ Essential guide for AI coding agents**                |
| 🔗 [**API Specification**](docs/reference/specs/api_specification.md) | Complete API reference for all languages                   |
| 📙 [**Product Requirements**](docs/reference/specs/prd.md)            | PRD with success metrics and requirements                  |
| 📕 [**System Design**](docs/reference/specs/sds.md)                   | Technical specifications and component design              |
| 🏛️ [**Architecture Decisions**](docs/reference/specs/adr.md)          | 8 ADRs documenting key architectural choices               |
| 📋 [**Implementation Plans**](docs/plans/)                            | Phase-by-phase TDD implementation guides                   |
| 🗺️ [**CALM Mapping**](docs/reference/specs/calm-mapping.md)           | SEA ↔ CALM conversion specification                        |
| 🎓 [**Examples**](sea-core/examples/)                                 | DSL examples and parser demos                              |
| 🗂️ [**Namespace Registry**](docs/reference/sea-registry.md)           | Configure `.sea-registry.toml` and workspace glob patterns |
| 📖 [**Error Code Catalog**](docs/specs/error_codes.md)                | Complete list of validation error codes and fixes          |

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
python -c "
import sea_dsl

# Create entities
warehouse = sea_dsl.Entity('Warehouse A', 'logistics')
factory = sea_dsl.Entity('Factory B', 'logistics')

# Create a resource
cameras = sea_dsl.Resource('Cameras', 'units', 'products')

# Create a flow between entities
flow = sea_dsl.Flow(cameras.id(), factory.id(), warehouse.id(), 1000.0)

print(f'Flow: {flow.quantity()} {cameras.name()} from {factory.name()} to {warehouse.name()}')
"
```

---

## 📄 License

SEA DSL is open source under the [Apache-2.0 License](LICENSE).

---

## 🙏 Acknowledgments

Built on the shoulders of giants:

- **[ERP5 Unified Business Model](https://www.erp5.com/)** - Foundational primitive design
- **[SBVR Standard (OMG)](https://www.omg.org/spec/SBVR/)** - Semantic business vocabulary
- **[FINOS CALM](https://github.com/finos/architecture-as-code)** - Architecture-as-Code integration
- **[Rust Community](https://www.rust-lang.org/)** - High-performance core runtime

---

> **Build enterprise models that business analysts can read and machines can execute**

# 🏛️ DomainForge

## Executable domain meaning for humans, agents, and systems

> Business rules should not live in folklore, stale PDFs, and three different implementations. DomainForge turns domain meaning into readable, executable structure that can move across languages, runtimes, and governance surfaces.

[![Rust](https://img.shields.io/badge/rust-1.92%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.11%2B-blue.svg)](https://www.python.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)](https://www.typescriptlang.org/)
[![WASM](https://img.shields.io/badge/wasm-ready-purple.svg)](https://webassembly.org/)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![CI](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml)

---

## The Drift Problem

Every organization has domain truth: the rules, entities, flows, policies, metrics, and relationships that define how the business actually works.

The problem is that this meaning usually gets scattered.

- Analysts describe it in documents.
- Developers reimplement it in services.
- Frontends rebuild parts of it again.
- Architects model it somewhere else.
- Auditors ask where it is enforced.
- Agents receive fragments and improvise.

That is not governance. That is a scavenger hunt.

DomainForge gives teams a semantic source of truth for domain structure: human-readable, machine-executable, versionable, and projectable into the systems that need to use it.

Analysts can read it. Developers can bind to it. Agents can reason over it. Governance systems can inspect it. Architecture tools can project it.

Not documentation hoping to be obeyed. Executable meaning with receipts.

| What You Get | Why It Matters |
| --- | --- |
| **One model, every language** | Python validation and TypeScript validation can use the same declared domain structure. |
| **Fast rule checking** | Current benchmarks include validation of 10,000 entities in under 100 milliseconds. |
| **Business-readable, machine-executable structure** | Analysts can read the rules. Systems can parse, validate, and enforce them. |
| **Architecture-as-Code projection** | Export to FINOS CALM for enterprise architecture and governance workflows. |
| **Structured validation and deterministic projection surfaces** | Not just documentation — rules that can be parsed, checked, projected, and tested. |

---

## See It Working: 60 Seconds

```python
import sea_dsl

graph = sea_dsl.Graph()

# Define WHO does the work
warehouse = sea_dsl.Entity("Warehouse", "logistics")
assembly_line = sea_dsl.Entity("Assembly Line A", "manufacturing")
graph.add_entity(warehouse)
graph.add_entity(assembly_line)

# Define WHAT moves between them
cameras = sea_dsl.Resource("Camera", "units")
graph.add_resource(cameras)

# Define HOW much moves
flow = sea_dsl.Flow(
    cameras.id(),
    assembly_line.id(),
    warehouse.id(),
    1000.0
)
graph.add_flow(flow)

# Validate everything
print(f"Entities: {graph.entity_count()}")
print(f"Flows: {graph.flow_count()}")
```

**That's it.** You just created an executable domain model that:

- Defines entities, resources, and flows as a typed graph
- Uses the same Rust-backed core semantics across Python, TypeScript, Rust, and browser/WASM surfaces
- Exports to FINOS CALM for architecture governance
- Becomes a source of truth teams can inspect, test, version, and govern

---

## Install in 30 Seconds

```bash
# Python
pip install sea-dsl

# TypeScript / Node.js
npm install @domainforge/sea

# Rust
cargo add sea-core

# Verify it works
python -c "import sea_dsl; print('✅ Ready:', sea_dsl.__version__)"
```

> 💡 Pre-built packages for PyPI, npm, and Crates.io. Build from source if pre-built wheels/binaries are not yet available for your platform.

---

## Use DomainForge Anywhere

DomainForge is stack-agnostic.

It does **not** require the GodSpeed AI ecosystem. It does not require SEA-Forge, SWE_SEED, GodSpeed-Agent, a specific agent runtime, a specific cloud, or a specific governance platform.

You can use DomainForge as a standalone Apache 2.0 semantic/domain layer in:

- internal tools
- backend services
- frontend validation
- architecture repositories
- agent runtimes
- policy engines
- knowledge graphs
- compliance workflows
- browser/WASM applications
- Python, TypeScript, and Rust systems

DomainForge's value is self-contained: it makes domain meaning readable, executable, portable, and testable.

Within the broader GodSpeed AI stack, DomainForge can also serve as the semantic layer:

```text
DomainForge defines the domain.
SEA-Forge governs the work.
SWE_SEED proves the change.
GodSpeed-Agent compounds the capability.
```

That stack is optional. DomainForge stands on its own.

---

## Who Benefits?

<table>
<tr>
<td width="50%">

### 💼 Business Analysts

**"I can finally model our processes without waiting for every rule to become custom code."**

- Write rules in controlled natural language
- Get validation feedback
- See the same domain model developers use

</td>
<td width="50%">

### 🏛️ Enterprise Architects

**"We can make business architecture executable and projectable."**

- Export to FINOS CALM
- Preserve semantic consistency across systems
- Integrate with architecture and governance workflows

</td>
</tr>
<tr>
<td width="50%">

### 💻 Software Developers

**"Type-safe domain models that work across our stack."**

- Native APIs for Python, TypeScript, Rust, and WASM
- Runtime validation backed by a Rust core
- Less duplicate business logic across services and frontends

</td>
<td width="50%">

### 📋 Compliance and Governance Teams

**"Policies can become executable, auditable structures."**

- Express complex rules in controlled language
- Check policies against declared domain structure
- Preserve inspectable traces of policy changes and decisions

</td>
</tr>
</table>

---

## How It Works: Six Core Building Blocks

DomainForge starts with **six core concepts**:

| Concept | What It Represents | Example |
| --- | --- | --- |
| **🏢 Entity** | Actors and locations in your system | Assembly Line, Customer, Warehouse |
| **📦 Resource** | Things of value that move | Products, Money, Information |
| **🔄 Flow** | Movement between entities | Shipments, Payments, Work Orders |
| **🔖 Instance** | Specific, trackable items | Camera #SN12345, Invoice #INV-2024 |
| **🔍 Pattern** | Reusable validation patterns | Email format, SKU codes |
| **📜 Policy** | Business rules and constraints | "All shipments must be inspected" |

That is the starting vocabulary. More advanced declarations — roles, relations, dimensions, units, metrics, mappings, projections, and concept changes — extend the model when the domain needs more structure.

No magic syntax. No framework lock-in. Just executable domain meaning.

---

## SEA DSL Syntax

Write models directly in `.sea` files — human-readable, version-controllable, and parseable by all bindings.

### Basic Example

```sea
// Define entities (who/where)
Entity "Warehouse" in logistics
Entity "Factory" in manufacturing

// Define resources (what moves)
Resource "Camera" units in inventory

// Define flows (how things move)
Flow "Camera" from "Warehouse" to "Factory" quantity 100

// Define validation patterns
Pattern "SKU" matches "^[A-Z]{3}-[0-9]{4}$"

// Define business rules
Policy min_shipment as: quantity >= 10
```

### Advanced Features

```sea
// Versioned entities with evolution tracking
Entity "Vendor" v2.0.0
  @replaces "Supplier" v1.0.0
  @changes ["renamed", "added_fields"]

// Roles and relations
Role "Approver" in governance

Relation "Payment"
  subject: "Buyer"
  predicate: "pays"
  object: "Seller"
  via: flow "Money"

// Typed instances
Instance acme_corp of "Vendor" {
  name: "Acme Corporation",
  credit_limit: 50000 "USD"
}

// Quantified policy expressions
Policy all_shipments_inspected as:
  forall s in flows: (s.inspected = true)
```

### All Declaration Types

| Declaration | Syntax |
| --- | --- |
| Entity | `Entity "Name" [vX.Y.Z] [in domain]` |
| Resource | `Resource "Name" [unit] [in domain]` |
| Flow | `Flow "Resource" from "A" to "B" [quantity N]` |
| Pattern | `Pattern "Name" matches "regex"` |
| Role | `Role "Name" [in domain]` |
| Relation | `Relation "Name" subject: ... predicate: ... object: ...` |
| Instance | `Instance id of "Entity" { field: value }` |
| Policy | `Policy name as: expression` |
| Dimension | `Dimension "Name"` |
| Unit | `Unit "Name" of "Dimension" factor N base "Base"` |
| Metric | `Metric "Name" as: expression` |
| Mapping | `Mapping "Name" for format { ... }` |
| Projection | `Projection "Name" for format { ... }` |
| ConceptChange | `ConceptChange "Name" @from_version ... @to_version ...` |

---

## Real-World Impact

<details>
<summary><strong>🏭 Manufacturing: Assembly Line Control</strong> (click to expand)</summary>

```python
import sea_dsl

graph = sea_dsl.Graph()

# Define the supply chain
supplier = sea_dsl.Entity("Component Supplier", "supply")
assembly = sea_dsl.Entity("Assembly Line A", "manufacturing")
quality_control = sea_dsl.Entity("QC Department", "quality")
finished_goods = sea_dsl.Entity("Finished Goods Warehouse", "logistics")
graph.add_entity(supplier)
graph.add_entity(assembly)
graph.add_entity(quality_control)
graph.add_entity(finished_goods)

# Define what's being built
pcb_board = sea_dsl.Resource("PCB Board", "pieces")
camera_module = sea_dsl.Resource("Camera Module", "units")
graph.add_resource(pcb_board)
graph.add_resource(camera_module)

# Define the flow of components
component_delivery = sea_dsl.Flow(
    pcb_board.id(),
    supplier.id(),
    assembly.id(),
    500.0
)
graph.add_flow(component_delivery)
```

**Result:** Inventory constraints can be checked before they turn into production delays.

</details>

<details>
<summary><strong>💰 Finance: Payment Fraud Detection</strong> (click to expand)</summary>

```typescript
import { Graph, Entity, Resource, Flow } from "@domainforge/sea";

const graph = new Graph();

const customer = new Entity("Customer Account");
const merchant = new Entity("Merchant Account");
const gateway = new Entity("Payment Gateway");
graph.addEntity(customer);
graph.addEntity(merchant);
graph.addEntity(gateway);

const money = new Resource("USD", "dollars");
graph.addResource(money);

// Track payment flows
const payment = new Flow(money.id, customer.id, merchant.id, 100);
graph.addFlow(payment);
```

**Result:** The same domain model can support policy checks before downstream action.

</details>

<details>
<summary><strong>🚚 Logistics: Cross-Border Compliance</strong> (click to expand)</summary>

```python
import sea_dsl

graph = sea_dsl.Graph()

# Multi-location warehouses
warehouse_us = sea_dsl.Entity("US Distribution Center", "logistics")
warehouse_us.set_attribute("location", "USA")

warehouse_eu = sea_dsl.Entity("EU Distribution Center", "logistics")
warehouse_eu.set_attribute("location", "Germany")

retail_store = sea_dsl.Entity("Retail Store", "logistics")
retail_store.set_attribute("location", "France")

graph.add_entity(warehouse_us)
graph.add_entity(warehouse_eu)
graph.add_entity(retail_store)

product = sea_dsl.Resource("Widget", "boxes")
graph.add_resource(product)

# Create cross-border flow
shipment = sea_dsl.Flow(
    product.id(),
    warehouse_eu.id(),
    retail_store.id(),
    250.0
)
graph.add_flow(shipment)
```

**Result:** Cross-border requirements can be modeled as enforceable policy instead of scattered checklist logic.

</details>

---

## Technical Foundation

<details>
<summary><strong>⚡ Performance Characteristics</strong></summary>

- **Rust core** for performance-critical validation
- **FFI overhead < 1ms** per operation
- **Streaming validation** for large models

See [benchmarks](docs/explanations/performance-benchmarks.md) for measured results.

</details>

<details>
<summary><strong>🔌 Language Bindings</strong></summary>

**Python (PyO3):**

```python
import sea_dsl

d = sea_dsl.Dimension.parse("currency")
u = sea_dsl.Unit("USD", "US Dollar", "Currency", 1.0, "USD")
```

**TypeScript (napi-rs):**

```typescript
import { Dimension, Unit } from "@domainforge/sea";

const d = Dimension.parse("currency");
const u = new Unit("USD", "US Dollar", "Currency", 1.0, "USD");
```

**WASM (browser):**

```javascript
import init, { Dimension, Unit } from "./sea_core.js";
await init();

const d = new Dimension("currency");
const u = new Unit("USD", "US Dollar", "Currency", 1.0, "USD");

// Programmatic Expression Building & Normalization
const expr = Expression.binary(
  BinaryOp.And,
  Expression.variable("b"),
  Expression.variable("a")
);
console.log(expr.normalize().toStringRepr()); // "(a AND b)"
```

Bindings are designed to preserve the same core semantics across runtimes, with conformance tests guarding against behavioral drift.

</details>

<details>
<summary><strong>🏗️ Architecture</strong></summary>

```text
┌───────────────────────────────────────────────────────┐
│ Your Application                                      │
├───────────────────────────────────────────────────────┤
│ Python API        │ TypeScript        │ WASM           │
│ (PyO3)            │ (napi-rs)         │ (wasm-bindgen) │
├───────────────────────────────────────────────────────┤
│ Rust Core Engine (sea-core)                           │
│ ┌─────────────┬──────────────┬─────────────────┐      │
│ │ Primitives  │ Graph Store  │ Policy Engine   │      │
│ │ Parser      │ Validator    │ CALM Export     │      │
│ │ Authority   │ Fact Store   │ Provenance      │      │
│ └─────────────┴──────────────┴─────────────────┘      │
└───────────────────────────────────────────────────────┘
```

**Design Principles:**

1. **Rust core is canonical.** Bindings wrap core behavior instead of duplicating logic.
2. **Deterministic behavior.** Stable ordering supports repeatable outputs across runs.
3. **Standards-aware projection.** SBVR-aligned expressions and FINOS CALM export help domain models travel into enterprise architecture workflows.

</details>

<details>
<summary><strong>🔧 CLI Usage</strong></summary>

```bash
# Install the CLI
cargo install --path sea-core --features cli

# Validate a model
sea validate model.sea

# Export to FINOS CALM
sea project --format calm model.sea architecture.json

# Import from Knowledge Graph
sea import --format kg model.ttl

# Normalize Expressions
sea normalize "b AND a"  # -> "(a AND b)"

# Evaluate authority decisions
sea authority config.json request.json --facts facts.json --json
```

[Full CLI Reference →](docs/reference/cli-commands.md)

</details>

<details>
<summary><strong>🛡️ Policy Authority</strong></summary>

DomainForge includes a **Policy Authority** system that makes business authority executable, deterministic, fact-grounded, and traceable.

**Key capabilities:**

- **Four policy modalities:** Permission, Prohibition, Obligation, Override
- **Trusted fact provenance:** Caller-supplied facts are untrusted by default
- **Three-valued logic:** True, False, and Unknown with modality-aware unknown handling
- **Deterministic conflict resolution:** Modality precedence → priority → specificity vector dominance
- **Replayable audit traces:** Every decision produces a complete `AuthorityTrace`

```python
from sea_dsl import AuthorityEnvironment

# Create environment from config
env = AuthorityEnvironment(config_json)
env.validate()

# Evaluate an authority request
trace_json, decision_json = env.evaluate(request_json, facts_json)
```

```typescript
import { evaluateAuthority } from '@domainforge/sea';

const result = evaluateAuthority(configJson, requestJson, factsJson);
console.log(result.decisionJson);
```

**Decision outcomes:** `Allow`, `Deny`, `Escalate`, `NotApplicable`, `Reject`

**Safety guarantees:**

- No authority without declared semantics
- No decision without trusted facts
- No trust upgrade through derivation
- No allow from unknown permission
- No weakened prohibition from omitted facts
- No scalar specificity shortcuts

</details>

<details>
<summary><strong>🔍 Error Diagnostics</strong></summary>

Comprehensive error handling with fuzzy matching suggestions:

```text
Error[E001]: Undefined Entity: 'Warehous' at line 1
  --> 1:23
  |
1 | Flow "Materials" from "Warehous" to "Factory"
  |                       ^^^^^^^^^^
  |
  hint: Did you mean 'Warehouse'?
```

- **30+ structured error codes** with detailed descriptions
- **Multiple output formats**: JSON, human-readable, LSP
- **Native error types** for Python, TypeScript, and WASM

[Error Code Catalog →](docs/reference/error-codes.md)

</details>

---

## Getting Started

### Quick Start

```bash
# Clone and set up
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge
pip install maturin
maturin develop

# Run your first model
python examples/camera_factory.py
```

### Running Tests

```bash
# Per-language tests
just rust-test      # Rust core
just python-test    # Python bindings
just ts-test        # TypeScript bindings

# All tests at once
just all-tests
```

### Build from Source

```bash
# Python bindings
pip install maturin && maturin build --release

# TypeScript bindings
npm install && npm run build

# WebAssembly
./scripts/build-wasm.sh
```

---

## Documentation

| Resource | Description |
| --- | --- |
| 📘 [**Copilot Instructions**](.github/copilot-instructions.md) | Essential guide for AI coding agents |
| 📗 [**Product Requirements**](docs/specs/PRD-001-sea-projection-framework.md) | PRD with success metrics |
| 📕 [**System Design**](docs/specs/SDS-002-sea-core-architecture.md) | Technical specifications |
| 🏛️ [**Architecture Decisions**](docs/specs/ADR-001-sea-dsl-semantic-source-of-truth.md) | Key architectural choices |
| 🗺️ [**CALM Mapping**](docs/reference/calm-mapping.md) | SEA ↔ CALM conversion |
| 📖 [**Error Codes**](docs/reference/error-codes.md) | Validation error reference |
| 🔧 [**CLI Reference**](docs/reference/cli-commands.md) | CLI commands and usage |

---

## Contributing

We welcome contributions. Please see [Contributing Guide](CONTRIBUTING.md) for developer notes on building the CLI, TypeScript bindings, and WASM.

---

## License

DomainForge is open source under the [Apache-2.0 License](LICENSE).

---

## Acknowledgments

Built on foundational work from:

- **[ERP5 Unified Business Model](https://www.erp5.com/)** — Foundational primitive design
- **[SBVR Standard (OMG)](https://www.omg.org/spec/SBVR/)** — Semantic business vocabulary
- **[FINOS CALM](https://github.com/finos/architecture-as-code)** — Architecture-as-Code integration
- **[Rust Community](https://www.rust-lang.org/)** — High-performance core runtime

---

<div align="center">

**Make business meaning executable.**

[Get Started →](#install-in-30-seconds) · [Read the Docs →](#documentation) · [See Examples →](#real-world-impact)

</div>

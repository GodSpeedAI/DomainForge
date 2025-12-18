# 🏛️ DomainForge

### **Your Business Rules, Everywhere. Always Correct. Always in Sync.**

> _What if your team never had to ask "which version is right?" again?_

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Python](https://img.shields.io/badge/python-3.8%2B-blue.svg)](https://www.python.org/)
[![TypeScript](https://img.shields.io/badge/typescript-5.0%2B-blue.svg)](https://www.typescriptlang.org/)
[![WASM](https://img.shields.io/badge/wasm-ready-purple.svg)](https://webassembly.org/)
[![License](https://img.shields.io/badge/License-MIT%20OR%20Apache%202.0-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/tests-544%2B%20passing-brightgreen.svg)](sea-core/tests/)

---

## The Hidden Cost You're Already Paying

Every enterprise faces the same invisible tax:

**🔴 The Documentation Drift Problem**
Your team spends weeks documenting business requirements. Then someone codes them in Python. Then someone else needs the same logic in TypeScript. Six months later, three different "source of truth" documents exist—and none of them match production.

**🔴 The Translation Telephone Game**  
_Business requirement → Technical spec → Backend code → Frontend code → Reality_  
At each handoff, fidelity is lost. At the end, you're left wondering: "Is what we built actually what we agreed to?"

**🔴 The Compliance Nightmare**  
Auditors ask: _"Show me where this rule is enforced."_  
You scramble across Confluence pages, Jira tickets, Python services, and TypeScript frontends—hoping you didn't miss anything.

---

## What If You Could Write It Once?

**DomainForge makes your business logic portable, provable, and permanent.**

Write your rules in plain language. Run them _identically_ in Python, TypeScript, Rust, or your browser. Change them in one place—watch the change cascade everywhere, automatically.

| What You Get                              | Why It Matters                                               |
| ----------------------------------------- | ------------------------------------------------------------ |
| **One model, every language**             | Python validation = TypeScript validation. Period. No drift. |
| **Instant rule checking**                 | 10,000 entities validated in under 100 milliseconds          |
| **Business-readable, machine-executable** | Your analysts can read the rules. Your systems enforce them. |
| **Architecture-as-Code**                  | Export directly to FINOS CALM for enterprise governance      |
| **Formal mathematical rigor**             | Not just documentation—provable correctness                  |

---

## See It Working (60 seconds)

```python
from sea import Model

# Define WHO does the work
assembly_line = model.entity("Assembly Line A")
warehouse = model.entity("Warehouse")

# Define WHAT moves between them
camera = model.resource("Camera", unit="units")

# Define HOW much moves
production = model.flow(
    name="Daily Production",
    resource=camera,
    from_entity=assembly_line,
    to_entity=warehouse,
    quantity=1000
)

# Define the RULES that must always be true
model.policy(
    "Minimum Production Threshold",
    expression="forall f in Flow where f.resource = Camera: f.quantity >= 500",
    severity="error"
)

# Validate everything—instantly
results = model.validate()
print(f"✅ Model valid: {results.is_valid}")
```

**That's it.** You just created a formal, executable business rule that:

- Validates production quantities instantly
- Works identically in Python, TypeScript, Rust, or the browser
- Exports to FINOS CALM for architecture governance
- Becomes the single source of truth everyone can trust

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

> 💡 Pre-built packages for PyPI, npm, and Crates.io. No compilation required.

---

## Who Benefits?

<table>
<tr>
<td width="50%">

### 💼 Business Analysts

**"I can finally model our processes without waiting for developers."**

- Write rules in controlled natural language
- Get instant validation feedback
- See the same model developers use

</td>
<td width="50%">

### 🏛️ Enterprise Architects

**"Single source of truth for the entire business architecture."**

- Export to FINOS CALM
- Formal mathematical foundation
- Integrates with existing governance tools

</td>
</tr>
<tr>
<td width="50%">

### 💻 Software Developers

**"Type-safe domain models that work across our whole stack."**

- Native APIs for Python, TypeScript, Rust, WASM
- Fast enough for runtime validation
- No more "which system has the right logic?"

</td>
<td width="50%">

### 📋 Compliance Officers

**"Regulations become executable, auditable policies."**

- Express complex rules in controlled language
- Automatic compliance checking
- Complete audit trail of policy changes

</td>
</tr>
</table>

---

## How It Works: Five Building Blocks

DomainForge models any business domain using just **five universal concepts**:

| Concept         | What It Represents                  | Example                            |
| --------------- | ----------------------------------- | ---------------------------------- |
| **🏢 Entity**   | Actors and locations in your system | Assembly Line, Customer, Warehouse |
| **📦 Resource** | Things of value that move           | Products, Money, Information       |
| **🔄 Flow**     | Movement between entities           | Shipments, Payments, Work Orders   |
| **🔖 Instance** | Specific, trackable items           | Camera #SN12345, Invoice #INV-2024 |
| **📜 Policy**   | Business rules and constraints      | "All shipments must be inspected"  |

That's the entire vocabulary. No magic syntax to learn. No framework lock-in.

---

## Real-World Impact

<details>
<summary><strong>🏭 Manufacturing: Assembly Line Control</strong> (click to expand)</summary>

```python
model = Model("electronics-manufacturing")

# Define the supply chain
supplier = model.entity("Component Supplier")
assembly = model.entity("Assembly Line A")
quality_control = model.entity("QC Department")
finished_goods = model.entity("Finished Goods Warehouse")

# Define what's being built
pcb_board = model.resource("PCB Board", unit="pieces")
camera_module = model.resource("Camera Module", unit="units")

# Define the flow of components
component_delivery = model.flow(
    resource=pcb_board,
    from_entity=supplier,
    to_entity=assembly,
    quantity=500
)

# Enforce Just-in-Time inventory limits
model.policy(
    "JIT Inventory Control",
    expression="""
        forall e in Entity where e.type = 'Assembly':
            sum(Instance.quantity where Instance.at = e) <= 1000
    """,
    severity="warn"
)
```

**Result:** Inventory violations caught before they cause production delays.

</details>

<details>
<summary><strong>💰 Finance: Payment Fraud Detection</strong> (click to expand)</summary>

```typescript
const model = new Model("payment-system");

const customer = model.entity("Customer Account");
const merchant = model.entity("Merchant Account");
const gateway = model.entity("Payment Gateway");

const money = model.resource("USD", { unit: "dollars" });

// Detect unusual spending patterns
model.policy({
  name: "Unusual Activity Detection",
  expression: `
    forall c in Entity where c.type = 'Customer':
      sum(Flow.quantity where Flow.from = c and Flow.timestamp > now() - 1hour) <= 10000
  `,
  severity: "error",
});
```

**Result:** Suspicious transactions flagged automatically—before they clear.

</details>

<details>
<summary><strong>🚚 Logistics: Cross-Border Compliance</strong> (click to expand)</summary>

```python
model = Model("global-supply-chain")

# Multi-location warehouses
warehouse_us = model.entity("US Distribution Center", location="USA")
warehouse_eu = model.entity("EU Distribution Center", location="Germany")
retail_store = model.entity("Retail Store", location="France")

product = model.resource("Widget", unit="boxes")

# Ensure customs documentation for international shipments
model.policy(
    "Customs Documentation Required",
    expression="""
        forall f in Flow where f.from.location != f.to.location:
            f.attributes.customs_cleared = true
    """,
    severity="error"
)
```

**Result:** Cross-border shipments without proper documentation are blocked automatically.

</details>

---

## Technical Foundation

<details>
<summary><strong>⚡ Performance Characteristics</strong></summary>

- **10,000 entities** validated in **< 100ms**
- **Rust core** for maximum performance
- **FFI overhead < 1ms** per operation
- **Streaming validation** for large models

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

All bindings provide **identical semantics**—zero behavioral drift.

</details>

<details>
<summary><strong>🏗️ Architecture</strong></summary>

```

┌───────────────────────────────────────────────────────┐
│ Your Application │
├───────────────────────────────────────────────────────┤
│ Python API │ TypeScript │ WASM │
│ (PyO3) │ (napi-rs) │ (wasm-bindgen) │
├───────────────────────────────────────────────────────┤
│ Rust Core Engine (sea-core) │
│ ┌─────────────┬──────────────┬─────────────────┐ │
│ │ Primitives │ Graph Store │ Policy Engine │ │
│ │ (5 types) │ (IndexMap) │ (SBVR logic) │ │
│ ├─────────────┼──────────────┼─────────────────┤ │
│ │ Parser │ Validator │ CALM Integration│ │
│ │ (Pest) │ (Ref check) │ (export/import) │ │
│ └─────────────┴──────────────┴─────────────────┘ │
└───────────────────────────────────────────────────────┘

```

**Design Principles:**

1. **Rust core is canonical.** Bindings wrap core—never duplicate logic.
2. **Deterministic behavior.** IndexMap ensures stable ordering across runs.
3. **Standards-based.** SBVR-aligned expressions, FINOS CALM export.

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
```

[Full CLI Reference →](docs/reference/cli.md)

</details>

<details>
<summary><strong>🔍 Error Diagnostics</strong></summary>

Comprehensive error handling with fuzzy matching suggestions:

```
Error[E001]: Undefined Entity: 'Warehous' at line 1
  --> 1:23
  |
1 | Flow "Materials" from "Warehous" to "Factory"
  |                       ^^^^^^^^^^
  |
  hint: Did you mean 'Warehouse'?
```

- **30+ structured error codes** with detailed descriptions
- **Multiple output formats**: JSON, Human-readable, LSP
- **Native error types** for Python, TypeScript, and WASM

[Error Code Catalog →](docs/specs/error_codes.md)

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

| Resource                                                              | Description                          |
| --------------------------------------------------------------------- | ------------------------------------ |
| 📘 [**Copilot Instructions**](.github/copilot-instructions.md)        | Essential guide for AI coding agents |
| 🔗 [**API Specification**](docs/reference/specs/api_specification.md) | Complete API reference               |
| 📙 [**Product Requirements**](docs/reference/specs/prd.md)            | PRD with success metrics             |
| 📕 [**System Design**](docs/reference/specs/sds.md)                   | Technical specifications             |
| 🏛️ [**Architecture Decisions**](docs/reference/specs/adr.md)          | Key architectural choices            |
| 🗺️ [**CALM Mapping**](docs/reference/specs/calm-mapping.md)           | SEA ↔ CALM conversion                |
| 📖 [**Error Codes**](docs/specs/error_codes.md)                       | Validation error reference           |

---

## What's New in v0.2.0

- **Unit Conversion**: Type-safe conversions with the `as` operator (`1000 'ms' as 's'`)
- **Profiles**: Define DSL dialects with `@profile "cloud"` or `@profile "data"`
- **Standard Library**: Import built-in types from `std:core`, `std:http`, `std:aws`

---

## Contributing

We welcome contributions! Please see [Contributing Guide](docs/CONTRIBUTING.md) for developer notes on building the CLI, TypeScript bindings, and WASM.

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

**Your business rules, everywhere. Always correct. Always in sync.**

[Get Started →](#install-in-30-seconds) · [Read the Docs →](#documentation) · [See Examples →](#real-world-impact)

</div>

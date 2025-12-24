# SEA Core

[![Crates.io](https://img.shields.io/crates/v/sea-core.svg)](https://crates.io/crates/sea-core)
[![Documentation](https://docs.rs/sea-core/badge.svg)](https://docs.rs/sea-core)
[![License](https://img.shields.io/crates/l/sea-core.svg)](https://github.com/GodSpeedAI/DomainForge/blob/main/LICENSE)
[![CI](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml)
[![MSRV](https://img.shields.io/badge/MSRV-1.77-blue.svg)](https://blog.rust-lang.org/2024/03/21/Rust-1.77.0.html)

Rust core library implementing the **SEA DSL** (Semantic Enterprise Architecture) primitives and validation engine. Part of the [DomainForge](https://github.com/GodSpeedAI/DomainForge) ecosystem.

## Features

- 🏗️ **Domain Primitives** — Entities, Resources, Flows, Roles, Relations
- 📐 **Unit System** — First-class dimensional analysis with 100+ built-in units
- ✅ **Policy Engine** — Constraint validation with three-valued logic
- 🔄 **Round-trip Parsing** — Parse SEA DSL to AST/Graph, format back to source
- 🌐 **Multi-target** — Compile to Rust, Python (PyO3), TypeScript (N-API), WASM

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sea-core = "0.6"
```

Or install the CLI:

```bash
cargo install sea-core --features cli
```

## Quick Start

```rust
use sea_core::parser::parse_to_graph;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        @namespace "finance"

        Entity "Customer"
        Entity "Vendor"

        Resource "Payment" USD in transactions

        Flow "Payment" from "Customer" to "Vendor"
    "#;

    let graph = parse_to_graph(source)?;

    println!("Entities: {:?}", graph.all_entities().len());
    println!("Resources: {:?}", graph.all_resources().len());
    println!("Flows: {:?}", graph.all_flows().len());

    Ok(())
}
```

## Feature Flags

| Feature              | Description                              | Default |
| -------------------- | ---------------------------------------- | ------- |
| `cli`                | Command-line interface (`sea` binary)    | ❌      |
| `python`             | Python bindings via PyO3                 | ❌      |
| `typescript`         | TypeScript/Node.js bindings via N-API    | ❌      |
| `wasm`               | WebAssembly target support               | ❌      |
| `shacl`              | SHACL/RDF knowledge graph export         | ❌      |
| `formatting`         | ICU-based number formatting              | ❌      |
| `three_valued_logic` | Three-valued logic for policy evaluation | ❌      |

### Enable features:

```toml
# CLI binary
sea-core = { version = "0.6", features = ["cli"] }

# Python bindings (for maturin builds)
sea-core = { version = "0.6", features = ["python"] }

# WASM target
sea-core = { version = "0.6", features = ["wasm"] }
```

## CLI Usage

```bash
# Validate a SEA file
sea validate domain.sea

# Format SEA source
sea format domain.sea

# Export to CALM JSON
sea project --format calm domain.sea output.json

# Show version and help
sea --version
sea --help
```

## API Overview

### Parsing

```rust
use sea_core::parser::{parse_to_graph, parse_to_graph_with_options, ParseOptions};

// Simple parsing
let graph = parse_to_graph(source)?;

// With options
let options = ParseOptions {
    default_namespace: Some("my_namespace".into()),
    ..Default::default()
};
let graph = parse_to_graph_with_options(source, &options)?;
```

### Querying the Graph

```rust
// Get all entities
for entity in graph.all_entities() {
    println!("{}: {:?}", entity.name(), entity.namespace());
}

// Get resource by name
if let Some(resource) = graph.resource_by_name("Payment") {
    println!("Unit: {}", resource.unit_symbol());
}

// Get flows for a resource
let flows = graph.flows_for_resource(&resource_id);
```

### Validation

```rust
use sea_core::validation::validate_graph;

let violations = validate_graph(&graph);
for violation in violations {
    eprintln!("[{}] {}", violation.severity, violation.message);
}
```

## Documentation

- 📖 [API Reference](https://docs.rs/sea-core) — Full Rust API documentation
- 📚 [SEA DSL Guide](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/reference/sea-dsl-syntax.md) — Language specification
- 🏗️ [Architecture](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/architecture.md) — Design overview

## Building from Source

```bash
# Clone the repository
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge

# Build the library
cargo build -p sea-core

# Run tests
cargo test -p sea-core

# Build with CLI
cargo build -p sea-core --features cli

# Build documentation
cargo doc -p sea-core --no-deps --open
```

## Minimum Supported Rust Version

This crate requires **Rust 1.77.0** or later.

## Related Crates

| Crate                                                                          | Description                 |
| ------------------------------------------------------------------------------ | --------------------------- |
| [`sea-core`](https://crates.io/crates/sea-core)                                | Core library (this crate)   |
| [`sea_dsl`](https://pypi.org/project/sea-dsl/)                                 | Python bindings             |
| [`@domainforge/sea-core`](https://www.npmjs.com/package/@domainforge/sea-core) | TypeScript/Node.js bindings |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](https://github.com/GodSpeedAI/DomainForge/blob/main/CONTRIBUTING.md) for guidelines.

## License

Licensed under the [Apache License, Version 2.0](https://www.apache.org/licenses/LICENSE-2.0).

---

Part of the [DomainForge](https://github.com/GodSpeedAI/DomainForge) project.

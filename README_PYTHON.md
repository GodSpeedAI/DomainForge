# SEA DSL

[![PyPI](https://img.shields.io/pypi/v/sea-dsl.svg)](https://pypi.org/project/sea-dsl/)
[![Python Versions](https://img.shields.io/pypi/pyversions/sea-dsl.svg)](https://pypi.org/project/sea-dsl/)
[![License](https://img.shields.io/pypi/l/sea-dsl.svg)](https://github.com/GodSpeedAI/DomainForge/blob/main/LICENSE)
[![CI](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml/badge.svg)](https://github.com/GodSpeedAI/DomainForge/actions/workflows/ci.yml)

Python bindings for the **SEA DSL** (Semantic Enterprise Architecture) domain-specific language. Part of the [DomainForge](https://github.com/GodSpeedAI/DomainForge) ecosystem.

## Features

- 🏗️ **Domain Primitives** — Entities, Resources, Flows, Roles, Relations, Instances
- 📐 **Unit System** — First-class dimensional analysis with type-safe quantities
- ✅ **Policy Engine** — Constraint validation with three-valued logic
- 🔄 **DSL Parsing** — Parse SEA source code into queryable graph structures
- 🌐 **CALM Integration** — Export/import FINOS CALM architecture-as-code format
- ⚡ **Native Performance** — Rust-powered core via PyO3

## Installation

```bash
pip install sea-dsl
```

**Requires:** Python 3.9+

## Quick Start

### Parse from DSL

```python
import sea_dsl

source = '''
    @namespace "supply_chain"

    Entity "Warehouse" in logistics
    Entity "Factory" in manufacturing

    Resource "Cameras" units

    Flow "Cameras" from "Warehouse" to "Factory" quantity 100
'''

graph = sea_dsl.Graph.parse(source)

print(f"Entities: {graph.entity_count()}")
print(f"Resources: {graph.resource_count()}")
print(f"Flows: {graph.flow_count()}")
```

### Build Programmatically

```python
import sea_dsl

graph = sea_dsl.Graph()

# Create primitives
warehouse = sea_dsl.Entity("Warehouse", "logistics")
factory = sea_dsl.Entity("Factory", "manufacturing")
cameras = sea_dsl.Resource("Cameras", "units")

graph.add_entity(warehouse)
graph.add_entity(factory)
graph.add_resource(cameras)

# Create flow
flow = sea_dsl.Flow(
    cameras.id(),
    warehouse.id(),
    factory.id(),
    100.0
)
graph.add_flow(flow)

# Query the graph
for entity in graph.all_entities():
    print(f"{entity.name()} in {entity.namespace()}")
```

### Work with Attributes

```python
entity = sea_dsl.Entity("Warehouse")
entity.set_attribute("capacity", 10000)
entity.set_attribute("location", "New York")

print(entity.get_attribute("capacity"))   # 10000
print(entity.get_attribute("location"))   # "New York"
```

### CALM Integration

```python
# Export to CALM JSON
calm_json = graph.export_calm()

# Import from CALM
imported_graph = sea_dsl.Graph.import_calm(calm_json)
```

## API Reference

### Core Classes

| Class              | Description                                            |
| ------------------ | ------------------------------------------------------ |
| `Entity`           | Business actors, locations, organizational units (WHO) |
| `Resource`         | Quantifiable subjects of value (WHAT)                  |
| `Flow`             | Transfers of resources between entities                |
| `Instance`         | Entity type instances with named fields                |
| `ResourceInstance` | Physical instances at entity locations                 |
| `Role`             | Roles that entities can play                           |
| `Relation`         | Relationships between roles                            |
| `Graph`            | Container with validation and query capabilities       |

### Graph Methods

```python
# Add primitives
graph.add_entity(entity)
graph.add_resource(resource)
graph.add_flow(flow)
graph.add_instance(instance)
graph.add_role(role)
graph.add_relation(relation)

# Counts
graph.entity_count()
graph.resource_count()
graph.flow_count()

# Lookup
graph.find_entity_by_name("Warehouse")
graph.find_resource_by_name("Cameras")

# Flow queries
graph.flows_from(entity_id)
graph.flows_to(entity_id)

# Get all
graph.all_entities()
graph.all_resources()
graph.all_flows()

# Policy evaluation
graph.add_policy(policy)
graph.evaluate_policy(policy_json)
graph.set_evaluation_mode(use_three_valued=True)

# CALM integration
graph.export_calm()
Graph.import_calm(json_str)
```

### NamespaceRegistry

```python
# Load workspace registry
reg = sea_dsl.NamespaceRegistry.from_file('.sea-registry.toml')

# Resolve files
for binding in reg.resolve_files():
    print(f"{binding.path} => {binding.namespace}")

# Query namespace for file
ns = reg.namespace_for('/path/to/file.sea')
```

## Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge

# Install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Run tests
pytest tests/
```

### Using Just (Recommended)

```bash
just python-setup    # Create venv and install deps
just python-test     # Run test suite
just python-clean    # Remove venv
```

## Platform Support

Pre-built wheels are available for:

| Platform | Architectures                 |
| -------- | ----------------------------- |
| Linux    | x86_64, aarch64               |
| macOS    | x86_64, arm64 (Apple Silicon) |
| Windows  | x86_64                        |

Build from source for other platforms.

## Related Packages

| Package                                                              | Registry  | Description                    |
| -------------------------------------------------------------------- | --------- | ------------------------------ |
| [`sea-core`](https://crates.io/crates/sea-core)                      | crates.io | Rust core library              |
| [`sea-dsl`](https://pypi.org/project/sea-dsl/)                       | PyPI      | Python bindings (this package) |
| [`@domainforge/sea`](https://www.npmjs.com/package/@domainforge/sea) | npm       | TypeScript/Node.js bindings    |

## Documentation

- 📖 [SEA DSL Guide](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/reference/sea-dsl-syntax.md) — Language specification
- 🏗️ [Architecture](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/architecture.md) — Design overview
- 📚 [API Reference](https://github.com/GodSpeedAI/DomainForge/blob/main/docs/reference/python-api.md) — Full Python API

## License

[Apache-2.0](https://www.apache.org/licenses/LICENSE-2.0)

---

Part of the [DomainForge](https://github.com/GodSpeedAI/DomainForge) project.

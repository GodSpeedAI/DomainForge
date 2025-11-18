# SEA DSL - Python Bindings

Python bindings for the Semantic Enterprise Architecture (SEA) Domain Specific Language.

## Installation

```bash
# From source (PyPI package coming soon)
pip install maturin
git clone https://github.com/GodSpeedAI/DomainForge.git
cd DomainForge
maturin develop

# Or build wheel
maturin build --release
pip install target/wheels/sea_dsl-*.whl
```

## Quick Start

### Creating Primitives

```python
import sea_dsl

# Create entities - use new() for default namespace, new_with_namespace() for explicit
warehouse = sea_dsl.Entity.new("Warehouse")  # Default namespace
factory = sea_dsl.Entity.new_with_namespace("Factory", "manufacturing")

# Namespace is always a string (not None), defaults to "default"
print(warehouse.namespace())  # "default"
print(factory.namespace())    # "manufacturing"

# Create resources
cameras = sea_dsl.Resource.new("Cameras", "units")

# Create flows - pass ConceptId values (clone before passing)
flow = sea_dsl.Flow.new(
    cameras.id().clone(),
    warehouse.id().clone(),
    factory.id().clone(),
    100.0
)
```

### Building a Graph

```python
import sea_dsl
from decimal import Decimal

# Create and populate a graph
graph = sea_dsl.Graph()

# Entities with constructor patterns
warehouse = sea_dsl.Entity.new("Warehouse")
factory = sea_dsl.Entity.new_with_namespace("Factory", "manufacturing")

# Resources with units
cameras = sea_dsl.Resource.new("Cameras", "units")

graph.add_entity(warehouse)
graph.add_entity(factory)
graph.add_resource(cameras)

# Flow with ConceptId clones and Decimal quantity
flow = sea_dsl.Flow.new(
    cameras.id().clone(),
    warehouse.id().clone(),
    factory.id().clone(),
    Decimal("100.0")
)
graph.add_flow(flow)

print(f"Graph has {graph.entity_count()} entities")
print(f"Graph has {graph.flow_count()} flows")
```

### Parsing DSL Source

```python
import sea_dsl

# Supports multiline strings with """ syntax
source = '''
    Entity "Warehouse" in logistics
    Entity """Multi-line
    Factory Name""" in manufacturing
    Resource "Cameras" units
    Flow "Cameras" from "Warehouse" to "Multi-line\nFactory Name" quantity 100
'''

graph = sea_dsl.Graph.parse(source)
print(f"Parsed {graph.entity_count()} entities")
print(f"Parsed {graph.flow_count()} flows")

# Query the graph
warehouse_id = graph.find_entity_by_name("Warehouse")
flows = graph.flows_from(warehouse_id)
for flow in flows:
    print(f"Flow: {flow.quantity()} units")
```

### Working with Attributes

```python
import sea_dsl

# Use new() for default namespace
entity = sea_dsl.Entity.new("Warehouse")
entity.set_attribute("capacity", 10000)
entity.set_attribute("location", "New York")

print(entity.get_attribute("capacity"))  # 10000
print(entity.get_attribute("location"))  # "New York"

# Namespace is always present (not None)
print(entity.namespace())  # "default"
```

## API Reference

### Classes

- `Entity`: Represents business entities (WHO)
- `Resource`: Represents quantifiable resources (WHAT)
- `Flow`: Represents resource movement between entities
- `Instance`: Represents physical instances of resources
- `Graph`: Container with validation and query capabilities (uses IndexMap for deterministic iteration)

### Constructor Patterns (November 2025)

**Entities:**

```python
# Default namespace
entity = Entity.new("Warehouse")  # namespace() returns "default"

# Explicit namespace
entity = Entity.new_with_namespace("Warehouse", "logistics")
```

**Resources:**

```python
resource = Resource.new("Cameras", "units")  # Default namespace
resource = Resource.new_with_namespace("Cameras", "units", "inventory")
```

**Flows:**

```python
# Takes ConceptId values (not references) - clone before passing
flow = Flow.new(
    resource_id.clone(),
    from_id.clone(),
    to_id.clone(),
    Decimal("100.0")
)
```

### Graph Methods

- `add_entity(entity)`: Add an entity to the graph
- `add_resource(resource)`: Add a resource to the graph
- `add_flow(flow)`: Add a flow to the graph (validates references)
- `add_instance(instance)`: Add an instance to the graph
- `entity_count()`: Get number of entities
- `resource_count()`: Get number of resources
- `flow_count()`: Get number of flows
- `instance_count()`: Get number of instances
- `find_entity_by_name(name)`: Find entity ID by name
- `find_resource_by_name(name)`: Find resource ID by name
- `flows_from(entity_id)`: Get all flows from an entity
- `flows_to(entity_id)`: Get all flows to an entity
- `all_entities()`: Get all entities
- `all_resources()`: Get all resources
- `all_flows()`: Get all flows
- `all_instances()`: Get all instances
- `Graph.parse(source)`: Parse DSL source into a graph
- `export_calm()`: Export graph to CALM JSON format
- `Graph.import_calm(json_str)`: Import graph from CALM JSON


### NamespaceRegistry (Workspace)

```python
import sea_dsl

reg = sea_dsl.NamespaceRegistry.from_file('./.sea-registry.toml')
files = reg.resolve_files()
for binding in files:
    print(binding.path, '=>', binding.namespace)

ns = reg.namespace_for('/path/to/file.sea')
print('Namespace:', ns)
```

- `namespace()` now returns `str` instead of `Optional[str]` (always returns "default" if unspecified)
- Constructors split: `new()` for default namespace, `new_with_namespace()` for explicit
- `Resource.new(name, unit)` now routes through `new_with_namespace(..., "default")` so `namespace()` never returns `None` even when a namespace is not supplied
- Flow constructor takes `ConceptId` values (not references) - must clone before passing



- Multiline string support in parser: `Entity """Multi-line\nName"""`
- ValidationError helpers: `undefined_entity()`, `unit_mismatch()`, etc.
- CALM integration: `export_calm()` and `import_calm()` for architecture-as-code
- IndexMap storage ensures deterministic iteration (reproducible results)

## Development

### Building from Source

```bash
# Install maturin
pip install maturin

# Build and install in development mode
maturin develop

# Run tests
pytest
```

### Running Tests

```bash
pytest tests/
```

## License

Apache-2.0

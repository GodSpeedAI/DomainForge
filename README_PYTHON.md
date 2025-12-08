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

# Create entities - use Entity(name, namespace) constructor
warehouse = sea_dsl.Entity("Warehouse")  # Default namespace (None)
factory = sea_dsl.Entity("Factory", "manufacturing")  # Explicit namespace

# Namespace returns Optional[str]
print(warehouse.namespace())  # None
print(factory.namespace())    # "manufacturing"

# Create resources - Resource(name, unit, namespace)
cameras = sea_dsl.Resource("Cameras", "units")

# Create flows - Flow(resource_id, from_id, to_id, quantity)
flow = sea_dsl.Flow(
    cameras.id(),
    warehouse.id(),
    factory.id(),
    100.0
)
```

### Building a Graph

```python
import sea_dsl
from decimal import Decimal

# Create and populate a graph
graph = sea_dsl.Graph()

# Entities with constructor
warehouse = sea_dsl.Entity("Warehouse")
factory = sea_dsl.Entity("Factory", "manufacturing")

# Resources with units
cameras = sea_dsl.Resource("Cameras", "units")

graph.add_entity(warehouse)
graph.add_entity(factory)
graph.add_resource(cameras)

# Flow with positional arguments
flow = sea_dsl.Flow(
    cameras.id(),
    warehouse.id(),
    factory.id(),
    100.0
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

entity = sea_dsl.Entity("Warehouse")
entity.set_attribute("capacity", 10000)
entity.set_attribute("location", "New York")

print(entity.get_attribute("capacity"))  # 10000
print(entity.get_attribute("location"))  # "New York"

# Namespace is None when not specified
print(entity.namespace())  # None
```

## API Reference

### Classes

- `Entity`: Represents business entities (WHO)
- `Resource`: Represents quantifiable resources (WHAT)
- `Flow`: Represents resource movement between entities
- `ResourceInstance`: Represents physical instances of resources at entity locations
- `Instance`: Represents instances of entity types with named fields
- `Role`: Represents roles that entities can play
- `Relation`: Represents relationships between roles
- `Graph`: Container with validation and query capabilities (uses IndexMap for deterministic iteration)

### Constructor Patterns (December 2025)

**Entities:**

```python
# Default namespace (None)
entity = Entity("Warehouse")  # namespace() returns None

# Explicit namespace
entity = Entity("Warehouse", "logistics")  # namespace() returns "logistics"
```

**Resources:**

```python
resource = Resource("Cameras", "units")  # Default namespace
resource = Resource("Cameras", "units", "inventory")  # Explicit namespace
```

**Flows:**

```python
# Takes string IDs and float quantity
flow = Flow(
    resource.id(),  # resource_id
    from_entity.id(),  # from_id
    to_entity.id(),  # to_id
    100.0  # quantity
)
```

**Instances:**

```python
# ResourceInstance - physical instance of a resource at an entity
instance = ResourceInstance(resource.id(), entity.id())
instance = ResourceInstance(resource.id(), entity.id(), "namespace")

# Instance - instance of an entity type with fields
inst = Instance("order_123", "Order")
inst.set_field("status", "pending")
```

### Graph Methods

- `add_entity(entity)`: Add an entity to the graph
- `add_resource(resource)`: Add a resource to the graph
- `add_flow(flow)`: Add a flow to the graph (validates references)
- `add_instance(instance)`: Add an instance to the graph
- `add_role(role)`: Add a role to the graph
- `add_relation(relation)`: Add a relation to the graph
- `entity_count()`: Get number of entities
- `resource_count()`: Get number of resources
- `flow_count()`: Get number of flows
- `instance_count()`: Get number of instances
- `role_count()`: Get number of roles
- `relation_count()`: Get number of relations
- `find_entity_by_name(name)`: Find entity ID by name
- `find_resource_by_name(name)`: Find resource ID by name
- `find_role_by_name(name)`: Find role ID by name
- `flows_from(entity_id)`: Get all flows from an entity
- `flows_to(entity_id)`: Get all flows to an entity
- `all_entities()`: Get all entities
- `all_resources()`: Get all resources
- `all_flows()`: Get all flows
- `all_instances()`: Get all instances
- `all_roles()`: Get all roles
- `all_relations()`: Get all relations
- `Graph.parse(source)`: Parse DSL source into a graph
- `export_calm()`: Export graph to CALM JSON format
- `Graph.import_calm(json_str)`: Import graph from CALM JSON
- `add_policy(policy)`: Add a policy to the graph
- `add_association(owner_id, owned_id, rel_type)`: Add ownership/association relation
- `evaluate_policy(policy_json)`: Evaluate a policy against the graph
- `set_evaluation_mode(use_three_valued)`: Set evaluation mode (three-valued or strict boolean)
- `use_three_valued_logic()`: Get current evaluation mode

### NamespaceRegistry (Workspace)

```python
import sea_dsl

reg = sea_dsl.NamespaceRegistry.from_file('./.sea-registry.toml')
files = reg.resolve_files()
for binding in files:
    print(binding.path, '=>', binding.namespace)

ns = reg.namespace_for('/path/to/file.sea')
print('Namespace:', ns)

# You can also pass `True` as an optional second argument to make resolution fail on ambiguity:
try:
    reg.namespace_for(str('/path/to/file.sea'), True)
except Exception as e:
    print('Ambiguity detected:', e)
```

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

Quick start for tests in development (recommended):

```bash
# Requires just
just python-setup
just python-test
```

If you'd like to remove the local virtual environment and start fresh:

```bash
just python-clean
```

### Manual Python workflow (without just)

```bash
# Create a local virtual environment
python -m venv .venv

# Activate the environment
# Linux/macOS:
source .venv/bin/activate
# Windows (Command Prompt):
.\.venv\Scripts\activate
# Windows (PowerShell):
.\.venv\Scripts\Activate.ps1

# Install development dependencies
pip install -e .  # or `pip install -r requirements-dev.txt`

# Run the Python test suite
pytest tests/

# Clean up the virtual environment when you're done
deactivate
rm -rf .venv
```

## License

Apache-2.0

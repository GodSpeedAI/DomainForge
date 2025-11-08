# SEA DSL - Python Bindings

Python bindings for the Semantic Enterprise Architecture (SEA) Domain Specific Language.

## Installation

```bash
pip install sea-dsl
```

## Quick Start

### Creating Primitives

```python
import sea_dsl

# Create entities
warehouse = sea_dsl.Entity("Warehouse", namespace="logistics")
factory = sea_dsl.Entity("Factory", namespace="logistics")

# Create resources
cameras = sea_dsl.Resource("Cameras", "units")

# Create flows
flow = sea_dsl.Flow(
    resource_id=cameras.id,
    from_id=warehouse.id,
    to_id=factory.id,
    quantity=100.0
)
```

### Building a Graph

```python
import sea_dsl

# Create and populate a graph
graph = sea_dsl.Graph()

warehouse = sea_dsl.Entity("Warehouse")
factory = sea_dsl.Entity("Factory")
cameras = sea_dsl.Resource("Cameras", "units")

graph.add_entity(warehouse)
graph.add_entity(factory)
graph.add_resource(cameras)

flow = sea_dsl.Flow(cameras.id, warehouse.id, factory.id, 100.0)
graph.add_flow(flow)

print(f"Graph has {graph.entity_count()} entities")
print(f"Graph has {graph.flow_count()} flows")
```

### Parsing DSL Source

```python
import sea_dsl

source = '''
    Entity "Warehouse" in logistics
    Entity "Factory" in logistics
    Resource "Cameras" units
    Flow "Cameras" from "Warehouse" to "Factory" quantity 100
'''

graph = sea_dsl.Graph.parse(source)
print(f"Parsed {graph.entity_count()} entities")
print(f"Parsed {graph.flow_count()} flows")

# Query the graph
warehouse_id = graph.find_entity_by_name("Warehouse")
flows = graph.flows_from(warehouse_id)
for flow in flows:
    print(f"Flow: {flow.quantity} units")
```

### Working with Attributes

```python
import sea_dsl

entity = sea_dsl.Entity("Warehouse")
entity.set_attribute("capacity", 10000)
entity.set_attribute("location", "New York")

print(entity.get_attribute("capacity"))  # 10000
print(entity.get_attribute("location"))  # "New York"
```

## API Reference

### Classes

- `Entity`: Represents business entities (WHO)
- `Resource`: Represents quantifiable resources (WHAT)
- `Flow`: Represents resource movement between entities
- `Graph`: Container with validation and query capabilities

### Graph Methods

- `add_entity(entity)`: Add an entity to the graph
- `add_resource(resource)`: Add a resource to the graph
- `add_flow(flow)`: Add a flow to the graph (validates references)
- `entity_count()`: Get number of entities
- `resource_count()`: Get number of resources
- `flow_count()`: Get number of flows
- `find_entity_by_name(name)`: Find entity ID by name
- `find_resource_by_name(name)`: Find resource ID by name
- `flows_from(entity_id)`: Get all flows from an entity
- `flows_to(entity_id)`: Get all flows to an entity
- `all_entities()`: Get all entities
- `all_resources()`: Get all resources
- `all_flows()`: Get all flows
- `Graph.parse(source)`: Parse DSL source into a graph

## Development

### Building from Source

```bash
# Install maturin
pip install maturin

# Build and install in development mode
maturin develop --features python

# Run tests
pytest
```

### Running Tests

```bash
pytest tests/
```

## License

MIT OR Apache-2.0

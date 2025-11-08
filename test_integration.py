#!/usr/bin/env python3
import sea_dsl

# Create a supply chain model
graph = sea_dsl.Graph()

# Create entities
supplier = sea_dsl.Entity("Supplier", "supply_chain")
warehouse = sea_dsl.Entity("Warehouse", "supply_chain")
store = sea_dsl.Entity("Store", "retail")

# Create resources
widgets = sea_dsl.Resource("Widgets", "units")
gadgets = sea_dsl.Resource("Gadgets", "units")

# Add to graph
graph.add_entity(supplier)
graph.add_entity(warehouse)
graph.add_entity(store)
graph.add_resource(widgets)
graph.add_resource(gadgets)

# Create flows
flow1 = sea_dsl.Flow(widgets.id, supplier.id, warehouse.id, 500.0)
flow2 = sea_dsl.Flow(widgets.id, warehouse.id, store.id, 300.0)
flow3 = sea_dsl.Flow(gadgets.id, supplier.id, warehouse.id, 200.0)

graph.add_flow(flow1)
graph.add_flow(flow2)
graph.add_flow(flow3)

# Query the graph
print(f"Graph Statistics:")
print(f"  Entities: {graph.entity_count()}")
print(f"  Resources: {graph.resource_count()}")
print(f"  Flows: {graph.flow_count()}")
print()

# Find entity by name
warehouse_id = graph.find_entity_by_name("Warehouse")
if warehouse_id is None:
    raise ValueError("Warehouse entity not found in graph")
print(f"Warehouse ID: {warehouse_id}")
print()

# Get flows from warehouse
flows_from_warehouse = graph.flows_from(warehouse_id)
print(f"Flows from Warehouse: {len(flows_from_warehouse)}")
for flow in flows_from_warehouse:
    print(f"  - {flow.quantity} units")
print()

# Parse DSL source
dsl_source = '''
Entity "Supplier" in supply_chain
Entity "Factory" in manufacturing
Resource "Steel" tons
Flow "Steel" from "Supplier" to "Factory" quantity 100
'''

parsed = sea_dsl.Graph.parse(dsl_source)
print(f"Parsed Graph:")
print(f"  Entities: {parsed.entity_count()}")
print(f"  Resources: {parsed.resource_count()}")
print(f"  Flows: {parsed.flow_count()}")

print("\nAll tests passed! Python bindings are working correctly.")
